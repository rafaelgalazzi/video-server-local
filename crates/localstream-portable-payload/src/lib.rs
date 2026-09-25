use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{self, Cursor, Read, Seek, SeekFrom, Write},
    path::{Component, Path, PathBuf},
};

use sha2::{Digest, Sha256};
use tar::{Archive, Builder, EntryType, Header};
use thiserror::Error;

const TRAILER_MAGIC: [u8; 8] = *b"LSTRPRT2";
const TRAILER_SIZE: u64 = 8 + 8 + 8 + 32 + 32;
const MANIFEST_PATH: &str = "portable-manifest.sha256";
const MAX_PAYLOAD_SIZE: u64 = 1024 * 1024 * 1024;
const MAX_ENTRY_SIZE: u64 = 512 * 1024 * 1024;
const MAX_ARCHIVE_ENTRIES: usize = 2_048;
const MAX_MANIFEST_SIZE: u64 = 1024 * 1024;
const COPY_BUFFER_SIZE: usize = 64 * 1024;

#[derive(Debug, Clone)]
pub struct PortableResource {
    pub source: PathBuf,
    pub archive_path: String,
}

#[derive(Debug, Error)]
pub enum PortableError {
    #[error("portable I/O failed while {context}")]
    Io {
        context: &'static str,
        #[source]
        source: io::Error,
    },
    #[error("the portable payload is truncated or malformed")]
    MalformedPayload,
    #[error("the portable payload exceeds the configured size limit")]
    PayloadTooLarge,
    #[error("the portable payload contains an unsafe or unsupported path")]
    UnsafeArchivePath,
    #[error("the portable payload contains too many entries")]
    TooManyEntries,
    #[error("the portable payload contains a duplicate entry")]
    DuplicateEntry,
    #[error("the portable payload contains an unsupported entry type")]
    UnsupportedEntry,
    #[error("the portable payload hash does not match its trailer")]
    HashMismatch,
    #[error("the portable payload is not a valid verified extraction")]
    InvalidInstallation,
    #[error("the portable extraction directory is not a normal directory")]
    UnsafeExtractionDirectory,
    #[error("the portable build resource list is empty")]
    EmptyResourceList,
    #[error("a portable resource is too large")]
    ResourceTooLarge,
    #[error("the portable resource set is incomplete")]
    IncompleteResourceSet,
    #[error("the portable build paths are not safe siblings")]
    UnsafeBuildPath,
    #[error("the base executable is already a portable executable")]
    NestedPortableExecutable,
    #[error("the base executable is not a supported Windows x64 PE image")]
    UnsupportedBaseExecutable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortableBuildSummary {
    pub destination: PathBuf,
    pub payload_size: u64,
    pub payload_sha256: String,
    pub file_sha256: String,
}

struct PreparedResource {
    source: File,
    size: u64,
    archive_path: String,
    sha256: [u8; 32],
}

struct PreparedBuild {
    resources: Vec<PreparedResource>,
    manifest: Vec<u8>,
}

pub fn create_portable_executable(
    base_executable: &Path,
    destination: &Path,
    resources: &[PortableResource],
) -> Result<PortableBuildSummary, PortableError> {
    if resources.is_empty() {
        return Err(PortableError::EmptyResourceList);
    }
    if resources.len() > MAX_ARCHIVE_ENTRIES - 1 {
        return Err(PortableError::TooManyEntries);
    }
    let partial = safe_partial_path(destination)?;
    if !base_executable.is_absolute() {
        return Err(PortableError::UnsafeBuildPath);
    }
    if read_payload_trailer(base_executable)?.is_some() {
        return Err(PortableError::NestedPortableExecutable);
    }
    validate_base_executable(base_executable)?;

    let mut resources = resources.to_vec();
    resources.sort_by(|left, right| left.archive_path.cmp(&right.archive_path));
    let mut resource_paths = BTreeSet::new();
    for resource in &resources {
        validate_archive_path(&resource.archive_path)?;
        if !resource_paths.insert(resource.archive_path.clone()) {
            return Err(PortableError::DuplicateEntry);
        }
    }
    require_complete_resource_set(&resources)?;
    let mut prepared = prepare_resources(resources)?;

    let parent = destination.parent().ok_or(PortableError::UnsafeBuildPath)?;
    fs::create_dir_all(parent).map_err(|source| PortableError::Io {
        context: "creating the portable output directory",
        source,
    })?;
    remove_file_if_present(&partial)?;
    fs::copy(base_executable, &partial).map_err(|source| PortableError::Io {
        context: "copying the base executable",
        source,
    })?;

    let build_result =
        append_payload_archive(&partial, &mut prepared.resources, &prepared.manifest);
    if let Err(error) = build_result {
        let _ = fs::remove_file(&partial);
        return Err(error);
    }

    replace_file(&partial, destination)?;
    let trailer = read_payload_trailer(destination)?.ok_or(PortableError::MalformedPayload)?;
    let file_sha256 = hash_file(destination)?;

    Ok(PortableBuildSummary {
        destination: destination.to_owned(),
        payload_size: trailer.payload_size,
        payload_sha256: hex(&trailer.payload_hash),
        file_sha256,
    })
}

fn validate_base_executable(path: &Path) -> Result<(), PortableError> {
    let mut file = File::open(path).map_err(|source| PortableError::Io {
        context: "opening the base executable for format validation",
        source,
    })?;
    let mut dos_header = [0_u8; 64];
    file.read_exact(&mut dos_header)
        .map_err(|_| PortableError::UnsupportedBaseExecutable)?;
    if &dos_header[..2] != b"MZ" {
        return Err(PortableError::UnsupportedBaseExecutable);
    }
    let pe_offset = u32::from_le_bytes(
        dos_header[0x3c..0x40]
            .try_into()
            .map_err(|_| PortableError::UnsupportedBaseExecutable)?,
    );
    let pe_offset = u64::from(pe_offset);
    file.seek(SeekFrom::Start(pe_offset))
        .map_err(|_| PortableError::UnsupportedBaseExecutable)?;
    let mut pe_header = [0_u8; 6];
    file.read_exact(&mut pe_header)
        .map_err(|_| PortableError::UnsupportedBaseExecutable)?;
    if &pe_header[..4] != b"PE\0\0" {
        return Err(PortableError::UnsupportedBaseExecutable);
    }
    let machine = u16::from_le_bytes(pe_header[4..6].try_into().unwrap_or_default());
    let optional_header_offset = pe_offset
        .checked_add(24)
        .ok_or(PortableError::UnsupportedBaseExecutable)?;
    file.seek(SeekFrom::Start(optional_header_offset))
        .map_err(|_| PortableError::UnsupportedBaseExecutable)?;
    let mut optional_magic = [0_u8; 2];
    file.read_exact(&mut optional_magic)
        .map_err(|_| PortableError::UnsupportedBaseExecutable)?;
    let optional = u16::from_le_bytes(optional_magic);
    if machine != 0x8664 || optional != 0x020b {
        return Err(PortableError::UnsupportedBaseExecutable);
    }
    Ok(())
}

fn safe_partial_path(destination: &Path) -> Result<PathBuf, PortableError> {
    if !destination.is_absolute() {
        return Err(PortableError::UnsafeBuildPath);
    }
    let parent = destination.parent().ok_or(PortableError::UnsafeBuildPath)?;
    let name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(PortableError::UnsafeBuildPath)?;
    if name.is_empty() || name == "." || name == ".." {
        return Err(PortableError::UnsafeBuildPath);
    }
    Ok(parent.join(format!(".{name}.portable-partial")))
}

fn require_complete_resource_set(resources: &[PortableResource]) -> Result<(), PortableError> {
    let required = [
        "ffmpeg.exe",
        "ffprobe.exe",
        "web/index.html",
        "licenses/ffmpeg/LICENSE",
        "licenses/ffmpeg/README.txt",
    ];
    let paths = resources
        .iter()
        .map(|resource| resource.archive_path.as_str())
        .collect::<BTreeSet<_>>();
    if required.iter().all(|required| paths.contains(required)) {
        Ok(())
    } else {
        Err(PortableError::IncompleteResourceSet)
    }
}

fn prepare_resources(resources: Vec<PortableResource>) -> Result<PreparedBuild, PortableError> {
    let mut prepared = Vec::with_capacity(resources.len());
    let mut manifest = String::new();
    for resource in resources {
        let metadata =
            fs::symlink_metadata(&resource.source).map_err(|source| PortableError::Io {
                context: "reading portable resource metadata",
                source,
            })?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(PortableError::UnsupportedEntry);
        }
        let size = metadata.len();
        if size > MAX_ENTRY_SIZE {
            return Err(PortableError::ResourceTooLarge);
        }
        let mut source = File::open(&resource.source).map_err(|source| PortableError::Io {
            context: "opening a portable resource",
            source,
        })?;
        let opened_size = source
            .metadata()
            .map_err(|source| PortableError::Io {
                context: "reading an opened portable resource size",
                source,
            })?
            .len();
        if opened_size != size {
            return Err(PortableError::ResourceTooLarge);
        }
        let digest = hash_open_file(&mut source, size)?;
        source
            .seek(SeekFrom::Start(0))
            .map_err(|source| PortableError::Io {
                context: "rewinding a portable resource",
                source,
            })?;
        manifest.push_str(&hex(&digest));
        manifest.push_str("  ");
        manifest.push_str(&resource.archive_path);
        manifest.push('\n');
        prepared.push(PreparedResource {
            source,
            size,
            archive_path: resource.archive_path,
            sha256: digest,
        });
    }
    Ok(PreparedBuild {
        resources: prepared,
        manifest: manifest.into_bytes(),
    })
}

fn append_payload_archive(
    executable: &Path,
    resources: &mut [PreparedResource],
    manifest: &[u8],
) -> Result<(), PortableError> {
    let base_size = metadata(executable, "reading the base executable size")?.len();
    let base_sha256 = hash_open_file(
        &mut OpenOptions::new()
            .read(true)
            .open(executable)
            .map_err(|source| PortableError::Io {
                context: "reading the base executable hash",
                source,
            })?,
        base_size,
    )?;
    let mut file = OpenOptions::new()
        .append(true)
        .open(executable)
        .map_err(|source| PortableError::Io {
            context: "opening the portable payload for writing",
            source,
        })?;
    file.seek(SeekFrom::Start(base_size))
        .map_err(|source| PortableError::Io {
            context: "seeking to the portable payload boundary",
            source,
        })?;

    let mut archive = Builder::new(file);
    append_bytes(&mut archive, MANIFEST_PATH, manifest)?;
    for resource in &mut *resources {
        append_prepared_file(&mut archive, resource)?;
    }
    for resource in &mut *resources {
        if hash_open_file(&mut resource.source, resource.size)? != resource.sha256 {
            return Err(PortableError::MalformedPayload);
        }
    }
    let mut file = archive.into_inner().map_err(|source| PortableError::Io {
        context: "finalizing the portable payload archive",
        source,
    })?;
    file.flush().map_err(|source| PortableError::Io {
        context: "flushing the portable payload archive",
        source,
    })?;
    let payload_end = file
        .seek(SeekFrom::End(0))
        .map_err(|source| PortableError::Io {
            context: "measuring the portable payload archive",
            source,
        })?;
    let payload_size = payload_end
        .checked_sub(base_size)
        .ok_or(PortableError::MalformedPayload)?;
    if payload_size > MAX_PAYLOAD_SIZE {
        return Err(PortableError::PayloadTooLarge);
    }
    let payload_hash = hash_file_range(executable, base_size, payload_size)?;
    file.write_all(&TRAILER_MAGIC)
        .and_then(|()| file.write_all(&base_size.to_le_bytes()))
        .and_then(|()| file.write_all(&payload_size.to_le_bytes()))
        .and_then(|()| file.write_all(&payload_hash))
        .and_then(|()| file.write_all(&base_sha256))
        .map_err(|source| PortableError::Io {
            context: "writing the portable payload trailer",
            source,
        })
}

fn append_bytes<W: Write>(
    archive: &mut Builder<W>,
    path: &str,
    bytes: &[u8],
) -> Result<(), PortableError> {
    let mut header = regular_header(bytes.len() as u64, 0o644);
    archive
        .append_data(&mut header, path, Cursor::new(bytes))
        .map_err(|source| PortableError::Io {
            context: "writing portable manifest data",
            source,
        })
}

fn append_prepared_file<W: Write>(
    archive: &mut Builder<W>,
    resource: &PreparedResource,
) -> Result<(), PortableError> {
    let mode = if resource.archive_path.ends_with(".exe") {
        0o755
    } else {
        0o644
    };
    let mut header = regular_header(resource.size, mode);
    archive
        .append_data(&mut header, &resource.archive_path, &resource.source)
        .map_err(|source| PortableError::Io {
            context: "writing a portable resource",
            source,
        })
}

fn regular_header(size: u64, mode: u32) -> Header {
    let mut header = Header::new_gnu();
    header.set_entry_type(EntryType::Regular);
    header.set_mode(mode);
    header.set_uid(0);
    header.set_gid(0);
    header.set_size(size);
    header.set_mtime(0);
    header.set_cksum();
    header
}

struct PayloadTrailer {
    base_size: u64,
    payload_size: u64,
    payload_hash: [u8; 32],
    base_hash: [u8; 32],
}

pub fn activate(executable: &Path, app_data: &Path) -> Result<Option<PathBuf>, PortableError> {
    let Some(trailer) = read_payload_trailer(executable)? else {
        return Ok(None);
    };
    if trailer.payload_size == 0 || trailer.payload_size > MAX_PAYLOAD_SIZE {
        return Err(PortableError::PayloadTooLarge);
    }
    let actual_payload_hash = hash_file_range(executable, trailer.base_size, trailer.payload_size)?;
    if actual_payload_hash != trailer.payload_hash {
        return Err(PortableError::HashMismatch);
    }
    let actual_base_hash = hash_file_range(executable, 0, trailer.base_size)?;
    if actual_base_hash != trailer.base_hash {
        return Err(PortableError::HashMismatch);
    }
    let expected_manifest = read_payload_manifest(executable, &trailer)?;

    let portable_parent = app_data.join("portable");
    fs::create_dir_all(&portable_parent).map_err(|source| PortableError::Io {
        context: "creating the portable runtime directory",
        source,
    })?;
    let parent_metadata =
        fs::symlink_metadata(&portable_parent).map_err(|source| PortableError::Io {
            context: "inspecting the portable runtime directory",
            source,
        })?;
    if !is_safe_directory_metadata(&parent_metadata) {
        return Err(PortableError::UnsafeExtractionDirectory);
    }

    let digest = hex(&trailer.payload_hash);
    let destination = portable_parent.join(&digest);
    if verify_installation(&destination, &expected_manifest).is_ok() {
        return Ok(Some(destination));
    }
    remove_owned_installation(&destination)?;

    let temporary =
        portable_parent.join(format!(".extract-{}-{}", &digest[..32], std::process::id()));
    remove_owned_installation(&temporary)?;
    fs::create_dir(&temporary).map_err(|source| PortableError::Io {
        context: "creating a temporary portable runtime directory",
        source,
    })?;

    let extraction = extract_payload(
        executable,
        trailer.base_size,
        trailer.payload_size,
        &temporary,
    )
    .and_then(|()| verify_installation(&temporary, &expected_manifest).map(|_| ()));
    if let Err(error) = extraction {
        let _ = remove_owned_installation(&temporary);
        return Err(error);
    }

    match fs::rename(&temporary, &destination) {
        Ok(()) => Ok(Some(destination)),
        Err(_) if verify_installation(&destination, &expected_manifest).is_ok() => {
            let _ = remove_owned_installation(&temporary);
            Ok(Some(destination))
        }
        Err(source) => {
            let _ = remove_owned_installation(&temporary);
            Err(PortableError::Io {
                context: "publishing the extracted portable runtime",
                source,
            })
        }
    }
}

fn read_payload_trailer(executable: &Path) -> Result<Option<PayloadTrailer>, PortableError> {
    let mut file = File::open(executable).map_err(|source| PortableError::Io {
        context: "opening the current executable",
        source,
    })?;
    let size = file
        .metadata()
        .map_err(|source| PortableError::Io {
            context: "reading the current executable size",
            source,
        })?
        .len();
    if size < TRAILER_SIZE {
        return Ok(None);
    }
    file.seek(SeekFrom::End(-(TRAILER_SIZE as i64)))
        .map_err(|source| PortableError::Io {
            context: "seeking to the portable payload trailer",
            source,
        })?;
    let mut trailer = [0_u8; TRAILER_SIZE as usize];
    file.read_exact(&mut trailer)
        .map_err(|source| PortableError::Io {
            context: "reading the portable payload trailer",
            source,
        })?;
    if trailer[..8] != TRAILER_MAGIC {
        return Ok(None);
    }
    let base_size = u64::from_le_bytes(
        trailer[8..16]
            .try_into()
            .map_err(|_| PortableError::MalformedPayload)?,
    );
    let payload_size = u64::from_le_bytes(
        trailer[16..24]
            .try_into()
            .map_err(|_| PortableError::MalformedPayload)?,
    );
    if base_size == 0
        || base_size
            .checked_add(payload_size)
            .and_then(|end| end.checked_add(TRAILER_SIZE))
            != Some(size)
    {
        return Err(PortableError::MalformedPayload);
    }
    let mut payload_hash = [0_u8; 32];
    payload_hash.copy_from_slice(&trailer[24..56]);
    let mut base_hash = [0_u8; 32];
    base_hash.copy_from_slice(&trailer[56..]);
    Ok(Some(PayloadTrailer {
        base_size,
        payload_size,
        payload_hash,
        base_hash,
    }))
}

fn read_payload_manifest(
    executable: &Path,
    trailer: &PayloadTrailer,
) -> Result<BTreeMap<String, String>, PortableError> {
    let mut file = File::open(executable).map_err(|source| PortableError::Io {
        context: "opening the portable payload manifest",
        source,
    })?;
    file.seek(SeekFrom::Start(trailer.base_size))
        .map_err(|source| PortableError::Io {
            context: "seeking to the portable payload manifest",
            source,
        })?;
    let reader = PayloadReader {
        file,
        start: trailer.base_size,
        length: trailer.payload_size,
        position: 0,
    };
    let mut archive = Archive::new(reader);
    let mut manifest = None;
    let mut seen = BTreeSet::new();
    let mut entry_count = 0_usize;
    for entry in archive.entries().map_err(|source| PortableError::Io {
        context: "reading the portable payload manifest archive",
        source,
    })? {
        let mut entry = entry.map_err(|source| PortableError::Io {
            context: "reading a portable payload manifest entry",
            source,
        })?;
        entry_count += 1;
        if entry_count > MAX_ARCHIVE_ENTRIES {
            return Err(PortableError::TooManyEntries);
        }
        if !entry.header().entry_type().is_file() {
            return Err(PortableError::UnsupportedEntry);
        }
        let path = entry
            .path()
            .map_err(|source| PortableError::Io {
                context: "reading a portable payload manifest path",
                source,
            })?
            .into_owned();
        let path = path.to_str().ok_or(PortableError::UnsafeArchivePath)?;
        let relative = validate_archive_path(path)?;
        if !seen.insert(relative.clone()) {
            return Err(PortableError::DuplicateEntry);
        }
        if path == MANIFEST_PATH {
            if entry.size() > MAX_MANIFEST_SIZE {
                return Err(PortableError::PayloadTooLarge);
            }
            let mut bytes = Vec::with_capacity(entry.size() as usize);
            entry
                .by_ref()
                .take(MAX_MANIFEST_SIZE + 1)
                .read_to_end(&mut bytes)
                .map_err(|source| PortableError::Io {
                    context: "reading the portable payload manifest",
                    source,
                })?;
            if bytes.len() as u64 > MAX_MANIFEST_SIZE {
                return Err(PortableError::PayloadTooLarge);
            }
            let text = String::from_utf8(bytes).map_err(|_| PortableError::MalformedPayload)?;
            manifest = Some(parse_manifest(&text)?);
        }
    }
    manifest.ok_or(PortableError::InvalidInstallation)
}

fn extract_payload(
    executable: &Path,
    payload_start: u64,
    payload_size: u64,
    destination: &Path,
) -> Result<(), PortableError> {
    let mut file = File::open(executable).map_err(|source| PortableError::Io {
        context: "opening the portable payload archive",
        source,
    })?;
    file.seek(SeekFrom::Start(payload_start))
        .map_err(|source| PortableError::Io {
            context: "seeking to the portable payload archive",
            source,
        })?;
    let reader = PayloadReader {
        file,
        start: payload_start,
        length: payload_size,
        position: 0,
    };
    let mut archive = Archive::new(reader);
    let mut seen = BTreeSet::new();
    let mut entry_count = 0_usize;
    let mut total_size = 0_u64;
    for entry in archive.entries().map_err(|source| PortableError::Io {
        context: "reading the portable payload archive",
        source,
    })? {
        let entry = entry.map_err(|source| PortableError::Io {
            context: "reading a portable payload entry",
            source,
        })?;
        entry_count += 1;
        if entry_count > MAX_ARCHIVE_ENTRIES {
            return Err(PortableError::TooManyEntries);
        }
        if !entry.header().entry_type().is_file() {
            return Err(PortableError::UnsupportedEntry);
        }
        let path = entry
            .path()
            .map_err(|source| PortableError::Io {
                context: "reading a portable payload path",
                source,
            })?
            .into_owned();
        let path = path.to_str().ok_or(PortableError::UnsafeArchivePath)?;
        let relative = validate_archive_path(path)?;
        let size = entry.size();
        if size > MAX_ENTRY_SIZE {
            return Err(PortableError::ResourceTooLarge);
        }
        total_size = total_size
            .checked_add(size)
            .ok_or(PortableError::PayloadTooLarge)?;
        if total_size > MAX_PAYLOAD_SIZE {
            return Err(PortableError::PayloadTooLarge);
        }
        if !seen.insert(relative.clone()) {
            return Err(PortableError::DuplicateEntry);
        }
        let target = destination.join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|source| PortableError::Io {
                context: "creating a portable payload directory",
                source,
            })?;
        }
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target)
            .map_err(|source| PortableError::Io {
                context: "creating an extracted portable file",
                source,
            })?;
        let copied =
            io::copy(&mut entry.take(MAX_ENTRY_SIZE + 1), &mut output).map_err(|source| {
                PortableError::Io {
                    context: "extracting a portable file",
                    source,
                }
            })?;
        if copied != size {
            return Err(PortableError::MalformedPayload);
        }
        output.flush().map_err(|source| PortableError::Io {
            context: "flushing an extracted portable file",
            source,
        })?;
    }
    if entry_count == 0 {
        return Err(PortableError::MalformedPayload);
    }
    Ok(())
}

fn verify_installation(
    root: &Path,
    expected_manifest: &BTreeMap<String, String>,
) -> Result<(), PortableError> {
    let root_metadata = match root.symlink_metadata() {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            return Err(PortableError::InvalidInstallation)
        }
        Err(source) => {
            return Err(PortableError::Io {
                context: "inspecting an extracted portable runtime",
                source,
            })
        }
    };
    if !is_safe_directory_metadata(&root_metadata) {
        return Err(PortableError::UnsafeExtractionDirectory);
    }
    let expected_paths = manifest_paths(expected_manifest)?;
    let mut actual_paths = BTreeSet::new();
    collect_files(root, root, &mut actual_paths, 0)?;
    if actual_paths != expected_paths {
        return Err(PortableError::InvalidInstallation);
    }

    let manifest_path = root.join(MANIFEST_PATH);
    if read_regular_bounded_utf8(&manifest_path, MAX_MANIFEST_SIZE)?
        != manifest_contents(expected_manifest)
    {
        return Err(PortableError::InvalidInstallation);
    }
    for (relative, expected_hash) in expected_manifest {
        if hex(&hash_regular_file(&root.join(relative))?) != expected_hash.as_str() {
            return Err(PortableError::InvalidInstallation);
        }
    }
    Ok(())
}

fn manifest_contents(manifest: &BTreeMap<String, String>) -> String {
    let mut contents = String::new();
    for (path, hash) in manifest {
        contents.push_str(hash);
        contents.push_str("  ");
        contents.push_str(path);
        contents.push('\n');
    }
    contents
}

fn manifest_paths(manifest: &BTreeMap<String, String>) -> Result<BTreeSet<PathBuf>, PortableError> {
    let mut paths = BTreeSet::new();
    paths.insert(PathBuf::from(MANIFEST_PATH));
    for path in manifest.keys() {
        validate_archive_path(path)?;
        paths.insert(PathBuf::from(path));
    }
    require_complete_resource_set_paths(&paths)?;
    Ok(paths)
}

fn require_complete_resource_set_paths(paths: &BTreeSet<PathBuf>) -> Result<(), PortableError> {
    let required = [
        "ffmpeg.exe",
        "ffprobe.exe",
        "web/index.html",
        "licenses/ffmpeg/LICENSE",
        "licenses/ffmpeg/README.txt",
    ];
    if required
        .iter()
        .all(|required| paths.iter().any(|path| path.to_str() == Some(*required)))
    {
        Ok(())
    } else {
        Err(PortableError::IncompleteResourceSet)
    }
}

fn parse_manifest(manifest: &str) -> Result<BTreeMap<String, String>, PortableError> {
    if manifest.len() as u64 > MAX_MANIFEST_SIZE {
        return Err(PortableError::PayloadTooLarge);
    }
    let mut entries = BTreeMap::new();
    for line in manifest.lines() {
        let (hash, path) = line
            .split_once("  ")
            .ok_or(PortableError::MalformedPayload)?;
        if hash.len() != 64
            || !hash
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(PortableError::MalformedPayload);
        }
        validate_archive_path(path)?;
        if path == MANIFEST_PATH {
            return Err(PortableError::DuplicateEntry);
        }
        if entries.insert(path.to_owned(), hash.to_owned()).is_some() {
            return Err(PortableError::DuplicateEntry);
        }
    }
    let paths = entries.keys().map(PathBuf::from).collect::<BTreeSet<_>>();
    require_complete_resource_set_paths(&paths)?;
    Ok(entries)
}

fn collect_files(
    root: &Path,
    directory: &Path,
    files: &mut BTreeSet<PathBuf>,
    depth: usize,
) -> Result<(), PortableError> {
    let mut tree = BTreeSet::new();
    collect_tree(root, directory, &mut tree, depth)?;
    for path in tree {
        let relative = path.to_string_lossy().replace('\\', "/");
        files.insert(validate_archive_path(&relative)?);
    }
    Ok(())
}

fn collect_tree(
    root: &Path,
    directory: &Path,
    files: &mut BTreeSet<PathBuf>,
    depth: usize,
) -> Result<(), PortableError> {
    if depth > 32 {
        return Err(PortableError::InvalidInstallation);
    }
    let entries = fs::read_dir(directory).map_err(|source| PortableError::Io {
        context: "enumerating an extracted portable directory",
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| PortableError::Io {
            context: "reading an extracted portable directory entry",
            source,
        })?;
        let file_type = entry.file_type().map_err(|source| PortableError::Io {
            context: "reading an extracted portable file type",
            source,
        })?;
        if file_type.is_symlink() {
            return Err(PortableError::UnsafeExtractionDirectory);
        }
        if file_type.is_dir() {
            collect_tree(root, &entry.path(), files, depth + 1)?;
        } else if file_type.is_file() {
            files.insert(
                entry
                    .path()
                    .strip_prefix(root)
                    .map_err(|_| PortableError::InvalidInstallation)?
                    .to_owned(),
            );
        } else {
            return Err(PortableError::UnsupportedEntry);
        }
    }
    Ok(())
}

fn validate_archive_path(path: &str) -> Result<PathBuf, PortableError> {
    if path.is_empty()
        || path.len() > 512
        || path
            .chars()
            .any(|character| matches!(character, '\\' | '\0' | ':'))
    {
        return Err(PortableError::UnsafeArchivePath);
    }
    let mut safe = PathBuf::new();
    for component in Path::new(path).components() {
        let Component::Normal(value) = component else {
            return Err(PortableError::UnsafeArchivePath);
        };
        if value.to_str().is_none() {
            return Err(PortableError::UnsafeArchivePath);
        }
        safe.push(value);
    }
    let allowed = safe == Path::new("ffmpeg.exe")
        || safe == Path::new("ffprobe.exe")
        || safe == Path::new(MANIFEST_PATH)
        || safe
            .strip_prefix(Path::new("web"))
            .is_ok_and(|remainder| !remainder.as_os_str().is_empty())
        || safe
            .strip_prefix(Path::new("licenses/ffmpeg"))
            .is_ok_and(|remainder| !remainder.as_os_str().is_empty());
    if !allowed {
        return Err(PortableError::UnsafeArchivePath);
    }
    Ok(safe)
}

fn hash_file(path: &Path) -> Result<String, PortableError> {
    Ok(hex(&hash_regular_file(path)?))
}

fn hash_regular_file(path: &Path) -> Result<[u8; 32], PortableError> {
    let file_metadata = fs::symlink_metadata(path).map_err(|source| PortableError::Io {
        context: "inspecting a file for portable hashing",
        source,
    })?;
    if file_metadata.file_type().is_symlink() || !file_metadata.is_file() {
        return Err(PortableError::UnsupportedEntry);
    }
    let mut file = File::open(path).map_err(|source| PortableError::Io {
        context: "opening a file for portable hashing",
        source,
    })?;
    let size = file
        .metadata()
        .map_err(|source| PortableError::Io {
            context: "reading a file size for portable hashing",
            source,
        })?
        .len();
    if size != file_metadata.len() {
        return Err(PortableError::MalformedPayload);
    }
    hash_open_file(&mut file, size)
}

fn hash_open_file(file: &mut File, length: u64) -> Result<[u8; 32], PortableError> {
    file.seek(SeekFrom::Start(0))
        .map_err(|source| PortableError::Io {
            context: "rewinding a file for portable hashing",
            source,
        })?;
    hash_reader(&mut *file, length)
}

fn read_regular_bounded_utf8(path: &Path, maximum_size: u64) -> Result<String, PortableError> {
    let file_metadata = fs::symlink_metadata(path).map_err(|source| PortableError::Io {
        context: "inspecting a bounded portable file",
        source,
    })?;
    if file_metadata.file_type().is_symlink() || !file_metadata.is_file() {
        return Err(PortableError::UnsupportedEntry);
    }
    let file = File::open(path).map_err(|source| PortableError::Io {
        context: "opening a bounded portable file",
        source,
    })?;
    let size = file
        .metadata()
        .map_err(|source| PortableError::Io {
            context: "reading a bounded portable file size",
            source,
        })?
        .len();
    if size != file_metadata.len() || size > maximum_size {
        return Err(PortableError::PayloadTooLarge);
    }
    let mut bytes = Vec::with_capacity(size as usize);
    file.take(maximum_size + 1)
        .read_to_end(&mut bytes)
        .map_err(|source| PortableError::Io {
            context: "reading a bounded portable file",
            source,
        })?;
    if bytes.len() as u64 > maximum_size {
        return Err(PortableError::PayloadTooLarge);
    }
    String::from_utf8(bytes).map_err(|_| PortableError::MalformedPayload)
}

fn hash_file_range(path: &Path, start: u64, length: u64) -> Result<[u8; 32], PortableError> {
    let mut file = File::open(path).map_err(|source| PortableError::Io {
        context: "opening a portable payload for hashing",
        source,
    })?;
    let size = file
        .metadata()
        .map_err(|source| PortableError::Io {
            context: "reading a portable payload size",
            source,
        })?
        .len();
    if start.checked_add(length).map_or(true, |end| end > size) {
        return Err(PortableError::MalformedPayload);
    }
    file.seek(SeekFrom::Start(start))
        .map_err(|source| PortableError::Io {
            context: "seeking to the portable payload hash range",
            source,
        })?;
    hash_reader(&mut file, length)
}

fn hash_reader(mut reader: impl Read, length: u64) -> Result<[u8; 32], PortableError> {
    let mut hasher = Sha256::new();
    let mut remaining = length;
    let mut buffer = vec![0_u8; COPY_BUFFER_SIZE];
    while remaining > 0 {
        let requested = usize::try_from(remaining.min(buffer.len() as u64))
            .map_err(|_| PortableError::PayloadTooLarge)?;
        let count = reader
            .read(&mut buffer[..requested])
            .map_err(|source| PortableError::Io {
                context: "hashing portable data",
                source,
            })?;
        if count == 0 {
            return Err(PortableError::MalformedPayload);
        }
        hasher.update(&buffer[..count]);
        remaining -= count as u64;
    }
    Ok(hasher.finalize().into())
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
    }
    output
}

fn metadata(path: &Path, context: &'static str) -> Result<fs::Metadata, PortableError> {
    path.metadata()
        .map_err(|source| PortableError::Io { context, source })
}

fn is_safe_directory_metadata(metadata: &fs::Metadata) -> bool {
    metadata.is_dir() && !metadata.file_type().is_symlink()
}

fn replace_file(source: &Path, destination: &Path) -> Result<(), PortableError> {
    match fs::rename(source, destination) {
        Ok(()) => Ok(()),
        Err(first) => {
            if !destination.exists() {
                let _ = fs::remove_file(source);
                return Err(PortableError::Io {
                    context: "publishing the portable executable",
                    source: first,
                });
            }
            let Some(name) = destination.file_name().and_then(|name| name.to_str()) else {
                let _ = fs::remove_file(source);
                return Err(PortableError::UnsafeBuildPath);
            };
            let Some(parent) = destination.parent() else {
                let _ = fs::remove_file(source);
                return Err(PortableError::UnsafeBuildPath);
            };
            let backup = parent.join(format!(".{name}.previous"));
            remove_file_if_present(&backup)?;
            fs::rename(destination, &backup).map_err(|source| PortableError::Io {
                context: "preserving the previous portable executable",
                source,
            })?;
            match fs::rename(source, destination) {
                Ok(()) => {
                    let _ = fs::remove_file(&backup);
                    Ok(())
                }
                Err(publish_error) => {
                    let _ = fs::rename(&backup, destination);
                    let _ = fs::remove_file(source);
                    Err(PortableError::Io {
                        context: "publishing the portable executable",
                        source: publish_error,
                    })
                }
            }
        }
    }
}

fn remove_file_if_present(path: &Path) -> Result<(), PortableError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(PortableError::Io {
            context: "removing an existing portable build file",
            source,
        }),
    }
}

fn remove_owned_installation(path: &Path) -> Result<(), PortableError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(source) => {
            return Err(PortableError::Io {
                context: "inspecting an existing portable runtime directory",
                source,
            })
        }
    };
    if !is_safe_directory_metadata(&metadata) {
        return Err(PortableError::UnsafeExtractionDirectory);
    }
    let mut files = BTreeSet::new();
    collect_tree(path, path, &mut files, 0)?;
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(PortableError::Io {
            context: "removing an invalid portable runtime directory",
            source,
        }),
    }
}

struct PayloadReader {
    file: File,
    start: u64,
    length: u64,
    position: u64,
}

impl Read for PayloadReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let remaining = self.length - self.position;
        let count = remaining.min(buffer.len() as u64) as usize;
        if count == 0 {
            return Ok(0);
        }
        let count = self.file.read(&mut buffer[..count])?;
        self.position += count as u64;
        Ok(count)
    }
}

impl Seek for PayloadReader {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        let target = match position {
            SeekFrom::Start(value) => Some(value),
            SeekFrom::End(value) => self.length.checked_add_signed(value),
            SeekFrom::Current(value) => self.position.checked_add_signed(value),
        }
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "portable seek overflow"))?;
        if target > self.length {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "portable seek outside payload",
            ));
        }
        self.file.seek(SeekFrom::Start(self.start + target))?;
        self.position = target;
        Ok(target)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        io::{Seek, Write},
    };

    use tempfile::tempdir;

    use super::{
        activate, create_portable_executable, hash_file, PortableError, PortableResource,
        TRAILER_SIZE,
    };

    fn base_image() -> Vec<u8> {
        let mut image = vec![0_u8; 128];
        image[..2].copy_from_slice(b"MZ");
        image[0x3c..0x40].copy_from_slice(&64_u32.to_le_bytes());
        image[64..68].copy_from_slice(b"PE\0\0");
        image[68..70].copy_from_slice(&0x8664_u16.to_le_bytes());
        image[88..90].copy_from_slice(&0x020b_u16.to_le_bytes());
        image
    }

    fn write_base_executable(path: &std::path::Path) {
        fs::write(path, base_image()).expect("base executable should be written");
    }

    fn resources(root: &std::path::Path) -> Vec<PortableResource> {
        let files = [
            ("ffmpeg.exe", b"ffmpeg-test".as_slice()),
            ("ffprobe.exe", b"ffprobe-test".as_slice()),
            ("web/index.html", b"<!doctype html>".as_slice()),
            ("licenses/ffmpeg/LICENSE", b"license-test".as_slice()),
            ("licenses/ffmpeg/README.txt", b"build-test".as_slice()),
        ];
        files
            .into_iter()
            .map(|(archive_path, contents)| {
                let source = root.join(format!("source-{archive_path}").replace('/', "_"));
                fs::write(&source, contents).expect("portable fixture should be written");
                PortableResource {
                    source,
                    archive_path: archive_path.to_owned(),
                }
            })
            .collect()
    }

    #[test]
    fn extracts_a_verified_payload_once_and_reuses_it() {
        let workspace = tempdir().expect("temporary workspace should exist");
        let base = workspace.path().join("base.exe");
        write_base_executable(&base);
        let portable = workspace.path().join("LocalStream.exe");
        let summary = create_portable_executable(&base, &portable, &resources(workspace.path()))
            .expect("portable executable should be built");
        assert!(summary.payload_size > 0);
        assert_eq!(summary.payload_sha256.len(), 64);
        assert_eq!(summary.file_sha256.len(), 64);

        let app_data = workspace.path().join("app-data");
        let extracted = activate(&portable, &app_data)
            .expect("payload should activate")
            .expect("portable payload should exist");
        assert_eq!(
            fs::read(extracted.join("web/index.html")).expect("UI should be extracted"),
            b"<!doctype html>"
        );
        assert_eq!(
            hash_file(&extracted.join("ffmpeg.exe")).expect("sidecar should hash"),
            hash_file(&workspace.path().join("source-ffmpeg.exe")).expect("source should hash")
        );
        assert_eq!(
            activate(&portable, &app_data).expect("second activation should succeed"),
            Some(extracted)
        );
    }

    #[test]
    fn rejects_a_tampered_payload_hash_before_extraction() {
        let workspace = tempdir().expect("temporary workspace should exist");
        let base = workspace.path().join("base.exe");
        write_base_executable(&base);
        let portable = workspace.path().join("LocalStream.exe");
        create_portable_executable(&base, &portable, &resources(workspace.path()))
            .expect("portable executable should be built");
        let mut file = fs::OpenOptions::new()
            .write(true)
            .open(&portable)
            .expect("portable executable should reopen");
        let size = file.metadata().expect("metadata should exist").len();
        use std::io::{Seek, SeekFrom, Write};
        file.seek(SeekFrom::Start(size - TRAILER_SIZE + 24))
            .expect("hash should be seekable");
        file.write_all(&[0_u8]).expect("hash should be corruptible");
        drop(file);

        assert!(matches!(
            activate(&portable, &workspace.path().join("app-data")),
            Err(PortableError::HashMismatch)
        ));
    }

    #[test]
    fn rejects_a_modified_base_executable() {
        let workspace = tempdir().expect("temporary workspace should exist");
        let base = workspace.path().join("base.exe");
        write_base_executable(&base);
        let portable = workspace.path().join("LocalStream.exe");
        create_portable_executable(&base, &portable, &resources(workspace.path()))
            .expect("portable executable should be built");
        let mut file = fs::OpenOptions::new()
            .write(true)
            .open(&portable)
            .expect("portable executable should reopen");
        let size = file.metadata().expect("metadata should exist").len();
        file.seek(std::io::SeekFrom::Start(size - TRAILER_SIZE + 56))
            .expect("base hash should be seekable");
        file.write_all(&[0_u8])
            .expect("base hash should be corruptible");
        drop(file);

        assert!(matches!(
            activate(&portable, &workspace.path().join("app-data")),
            Err(PortableError::HashMismatch)
        ));
    }

    #[test]
    fn repairs_an_extraction_even_when_its_local_manifest_is_rewritten() {
        let workspace = tempdir().expect("temporary workspace should exist");
        let base = workspace.path().join("base.exe");
        write_base_executable(&base);
        let portable = workspace.path().join("LocalStream.exe");
        create_portable_executable(&base, &portable, &resources(workspace.path()))
            .expect("portable executable should be built");
        let app_data = workspace.path().join("app-data");
        let extracted = activate(&portable, &app_data)
            .expect("payload should activate")
            .expect("portable payload should exist");

        let tampered_tool = extracted.join("ffmpeg.exe");
        fs::write(&tampered_tool, b"tampered-tool").expect("extracted tool should be tamperable");
        let tampered_hash = hash_file(&tampered_tool).expect("tampered tool should hash");
        let manifest_path = extracted.join(super::MANIFEST_PATH);
        let rewritten = fs::read_to_string(&manifest_path)
            .expect("manifest should be readable")
            .lines()
            .map(|line| {
                if line.ends_with("  ffmpeg.exe") {
                    format!("{tampered_hash}  ffmpeg.exe")
                } else {
                    line.to_owned()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&manifest_path, format!("{rewritten}\n")).expect("manifest should be tamperable");

        let repaired = activate(&portable, &app_data)
            .expect("payload should repair")
            .expect("portable payload should exist");
        assert_eq!(repaired, extracted);
        assert_eq!(
            hash_file(&repaired.join("ffmpeg.exe")).expect("repaired tool should hash"),
            hash_file(&workspace.path().join("source-ffmpeg.exe")).expect("source should hash")
        );
    }

    #[test]
    fn accepts_a_raw_executable_without_a_payload() {
        let workspace = tempdir().expect("temporary workspace should exist");
        let base = workspace.path().join("base.exe");
        write_base_executable(&base);
        assert_eq!(
            activate(&base, &workspace.path().join("app-data"))
                .expect("raw build should be accepted"),
            None
        );
    }

    #[test]
    fn rejects_nested_portable_builds() {
        let workspace = tempdir().expect("temporary workspace should exist");
        let base = workspace.path().join("base.exe");
        write_base_executable(&base);
        let portable = workspace.path().join("LocalStream.exe");
        let resources = resources(workspace.path());
        create_portable_executable(&base, &portable, &resources)
            .expect("portable executable should be built");
        assert!(matches!(
            create_portable_executable(&portable, &workspace.path().join("Nested.exe"), &resources),
            Err(PortableError::NestedPortableExecutable)
        ));
    }

    #[test]
    fn rejects_a_symlink_entry_even_with_a_valid_outer_hash() {
        use sha2::{Digest, Sha256};
        use tar::{Builder, EntryType, Header};

        let workspace = tempdir().expect("temporary workspace should exist");
        let portable = workspace.path().join("LocalStream.exe");
        let mut image = base_image();
        let base_size = image.len() as u64;
        let mut archive = Builder::new(Vec::new());
        let manifest = "0000000000000000000000000000000000000000000000000000000000000000  ffmpeg.exe\n0000000000000000000000000000000000000000000000000000000000000000  ffprobe.exe\n0000000000000000000000000000000000000000000000000000000000000000  web/index.html\n0000000000000000000000000000000000000000000000000000000000000000  licenses/ffmpeg/LICENSE\n0000000000000000000000000000000000000000000000000000000000000000  licenses/ffmpeg/README.txt\n";
        super::append_bytes(&mut archive, super::MANIFEST_PATH, manifest.as_bytes())
            .expect("manifest should be appended");
        let mut link_header = Header::new_gnu();
        link_header.set_entry_type(EntryType::Symlink);
        link_header.set_mode(0o777);
        link_header.set_size(0);
        link_header.set_mtime(0);
        link_header.set_cksum();
        archive
            .append_link(&mut link_header, "web/index.html", "../../escape")
            .expect("symlink entry should be appended");
        let payload = archive.into_inner().expect("payload should finalize");
        let payload_hash: [u8; 32] = Sha256::digest(&payload).into();
        image.extend_from_slice(&payload);
        image.extend_from_slice(&super::TRAILER_MAGIC);
        image.extend_from_slice(&base_size.to_le_bytes());
        image.extend_from_slice(&(payload.len() as u64).to_le_bytes());
        image.extend_from_slice(&payload_hash);
        image.extend_from_slice(&Sha256::digest(base_image()));
        fs::write(&portable, image).expect("malicious portable should be written");

        assert!(matches!(
            activate(&portable, &workspace.path().join("app-data")),
            Err(PortableError::UnsupportedEntry)
        ));
    }

    #[test]
    fn rejects_malformed_trailer_ranges_before_extraction() {
        let workspace = tempdir().expect("temporary workspace should exist");
        let base = workspace.path().join("base.exe");
        write_base_executable(&base);
        let portable = workspace.path().join("LocalStream.exe");
        create_portable_executable(&base, &portable, &resources(workspace.path()))
            .expect("portable executable should be built");
        let mut file = fs::OpenOptions::new()
            .write(true)
            .open(&portable)
            .expect("portable executable should reopen");
        let size = file.metadata().expect("metadata should exist").len();
        file.seek(std::io::SeekFrom::Start(size - TRAILER_SIZE + 8))
            .expect("base range should be seekable");
        file.write_all(&u64::MAX.to_le_bytes())
            .expect("base range should be corruptible");
        drop(file);

        assert!(matches!(
            activate(&portable, &workspace.path().join("app-data")),
            Err(PortableError::MalformedPayload)
        ));
    }

    #[test]
    fn rejects_paths_outside_the_portable_allowlist() {
        let workspace = tempdir().expect("temporary workspace should exist");
        let base = workspace.path().join("base.exe");
        write_base_executable(&base);
        let source = workspace.path().join("escape");
        fs::write(&source, b"escape").expect("escape fixture should be written");
        let resources = vec![PortableResource {
            source,
            archive_path: "../escape".to_owned(),
        }];
        assert!(matches!(
            create_portable_executable(
                &base,
                &workspace.path().join("LocalStream.exe"),
                &resources
            ),
            Err(PortableError::UnsafeArchivePath)
        ));
    }
}
