use std::path::{Path, PathBuf};

use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;
use walkdir::WalkDir;

const VIDEO_EXTENSIONS: &[&str] = &["m4v", "mkv", "mov", "mp4", "webm"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaItem {
    pub id: String,
    pub title: String,
    pub extension: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryScan {
    pub library_name: String,
    pub items: Vec<MediaItem>,
    pub skipped_entries: usize,
}

#[derive(Debug)]
pub(crate) struct ScannedMedia {
    pub item: MediaItem,
    pub path: PathBuf,
}

#[derive(Debug)]
pub(crate) struct ScannedLibrary {
    pub library_name: String,
    pub root_path: PathBuf,
    pub items: Vec<ScannedMedia>,
    pub skipped_entries: usize,
}

impl ScannedLibrary {
    pub(crate) fn public_view(&self) -> LibraryScan {
        LibraryScan {
            library_name: self.library_name.clone(),
            items: self.items.iter().map(|media| media.item.clone()).collect(),
            skipped_entries: self.skipped_entries,
        }
    }
}

#[derive(Debug, Error)]
pub enum LibraryScanError {
    #[error("the approved library does not exist")]
    MissingDirectory,
    #[error("the approved library is not a directory")]
    NotDirectory,
    #[error("the approved library could not be accessed")]
    InaccessibleDirectory,
}

pub fn scan_approved_directory(approved_directory: &Path) -> Result<LibraryScan, LibraryScanError> {
    scan_approved_directory_records(approved_directory).map(|scan| scan.public_view())
}

pub(crate) fn scan_approved_directory_records(
    approved_directory: &Path,
) -> Result<ScannedLibrary, LibraryScanError> {
    if !approved_directory.exists() {
        return Err(LibraryScanError::MissingDirectory);
    }
    if !approved_directory.is_dir() {
        return Err(LibraryScanError::NotDirectory);
    }

    let approved_directory = approved_directory
        .canonicalize()
        .map_err(|_| LibraryScanError::InaccessibleDirectory)?;
    let library_name = approved_directory
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("Media library")
        .to_owned();

    let mut items = Vec::new();
    let mut skipped_entries = 0;

    for entry in WalkDir::new(&approved_directory).follow_links(false) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => {
                skipped_entries += 1;
                continue;
            }
        };

        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let Some(extension) = supported_extension(path) else {
            continue;
        };
        let Ok(metadata) = entry.metadata() else {
            skipped_entries += 1;
            continue;
        };

        let title = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .filter(|stem| !stem.is_empty())
            .unwrap_or("Untitled video")
            .to_owned();
        let opaque_id =
            Uuid::new_v5(&Uuid::NAMESPACE_URL, path.to_string_lossy().as_bytes()).to_string();

        items.push(ScannedMedia {
            item: MediaItem {
                id: opaque_id,
                title,
                extension,
                size_bytes: metadata.len(),
            },
            path: path.to_path_buf(),
        });
    }

    items.sort_unstable_by(|left, right| {
        left.item
            .title
            .to_lowercase()
            .cmp(&right.item.title.to_lowercase())
            .then_with(|| left.item.id.cmp(&right.item.id))
    });

    Ok(ScannedLibrary {
        library_name,
        root_path: approved_directory,
        items,
        skipped_entries,
    })
}

fn supported_extension(path: &Path) -> Option<String> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    VIDEO_EXTENSIONS
        .contains(&extension.as_str())
        .then_some(extension)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{scan_approved_directory, LibraryScanError};

    #[test]
    fn discovers_supported_videos_recursively_without_paths() {
        let directory = tempdir().expect("temporary library should be created");
        let nested = directory.path().join("Season 1");
        fs::create_dir(&nested).expect("nested directory should be created");
        fs::write(directory.path().join("Movie.MP4"), b"video")
            .expect("video fixture should be written");
        fs::write(nested.join("Episode.mkv"), b"episode")
            .expect("nested video fixture should be written");
        fs::write(directory.path().join("notes.txt"), b"not media")
            .expect("non-media fixture should be written");

        let scan = scan_approved_directory(directory.path()).expect("scan should succeed");

        assert_eq!(scan.items.len(), 2);
        assert_eq!(scan.items[0].title, "Episode");
        assert_eq!(scan.items[0].extension, "mkv");
        assert_eq!(scan.items[1].title, "Movie");
        assert_eq!(scan.items[1].extension, "mp4");
        assert_eq!(scan.items[1].size_bytes, 5);
        assert!(scan.items.iter().all(|item| !item.id.contains('\\')));
        assert!(scan.items.iter().all(|item| !item.id.contains('/')));
        assert_eq!(scan.skipped_entries, 0);
    }

    #[test]
    fn produces_stable_opaque_identifiers() {
        let directory = tempdir().expect("temporary library should be created");
        fs::write(directory.path().join("Movie.mp4"), b"video")
            .expect("video fixture should be written");

        let first = scan_approved_directory(directory.path()).expect("first scan should succeed");
        let second = scan_approved_directory(directory.path()).expect("second scan should succeed");

        assert_eq!(first.items[0].id, second.items[0].id);
        assert_ne!(first.items[0].id, "Movie.mp4");
    }

    #[test]
    fn rejects_a_file_as_an_approved_library() {
        let directory = tempdir().expect("temporary directory should be created");
        let file = directory.path().join("Movie.mp4");
        fs::write(&file, b"video").expect("video fixture should be written");

        let error = scan_approved_directory(&file).expect_err("file must be rejected");

        assert!(matches!(error, LibraryScanError::NotDirectory));
    }
}
