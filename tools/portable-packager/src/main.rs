use std::{
    fs,
    path::{Path, PathBuf},
};

use localstream_portable_payload::{create_portable_executable, PortableError, PortableResource};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tool_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repository_root = tool_root
        .parent()
        .and_then(Path::parent)
        .ok_or(PortableError::MalformedPayload)?;
    let tauri_root = repository_root.join("src-tauri");
    let target_triple = "x86_64-pc-windows-msvc";
    let base_executable = repository_root
        .join("target")
        .join(target_triple)
        .join("release")
        .join("localstream-app.exe");
    let destination = repository_root
        .join("target")
        .join("release")
        .join("portable")
        .join("LocalStream.exe");

    let mut resources = vec![
        resource(
            tauri_root
                .join("binaries")
                .join(format!("ffmpeg-{target_triple}.exe")),
            "ffmpeg.exe",
        ),
        resource(
            tauri_root
                .join("binaries")
                .join(format!("ffprobe-{target_triple}.exe")),
            "ffprobe.exe",
        ),
    ];
    collect_resources(&repository_root.join("dist"), "web", &mut resources)?;
    collect_resources(
        &tauri_root.join("generated").join("ffmpeg"),
        "licenses/ffmpeg",
        &mut resources,
    )?;

    let summary = create_portable_executable(&base_executable, &destination, &resources)?;
    println!("Portable executable: {}", summary.destination.display());
    println!("Payload bytes: {}", summary.payload_size);
    println!("Payload SHA-256: {}", summary.payload_sha256);
    println!("File SHA-256: {}", summary.file_sha256);
    Ok(())
}

fn resource(source: PathBuf, archive_path: &str) -> PortableResource {
    PortableResource {
        source,
        archive_path: archive_path.to_owned(),
    }
}

fn collect_resources(
    source_root: &Path,
    archive_root: &str,
    resources: &mut Vec<PortableResource>,
) -> Result<(), PortableError> {
    let mut files = Vec::new();
    collect_files(source_root, &mut files)?;
    files.sort();
    for file in files {
        let relative = file
            .strip_prefix(source_root)
            .map_err(|_| PortableError::MalformedPayload)?
            .to_string_lossy()
            .replace('\\', "/");
        resources.push(resource(file, &format!("{archive_root}/{relative}")));
    }
    Ok(())
}

fn collect_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), PortableError> {
    for entry in fs::read_dir(directory).map_err(|source| PortableError::Io {
        context: "enumerating portable build resources",
        source,
    })? {
        let entry = entry.map_err(|source| PortableError::Io {
            context: "reading a portable build resource",
            source,
        })?;
        let file_type = entry.file_type().map_err(|source| PortableError::Io {
            context: "reading a portable build resource type",
            source,
        })?;
        if file_type.is_symlink() {
            return Err(PortableError::UnsafeExtractionDirectory);
        }
        if file_type.is_dir() {
            collect_files(&entry.path(), files)?;
        } else if file_type.is_file() {
            files.push(entry.path());
        } else {
            return Err(PortableError::UnsupportedEntry);
        }
    }
    Ok(())
}
