use tokio::fs::File;

use crate::{database::MediaLocation, DatabaseError};

pub mod range;

#[derive(Debug)]
pub struct DirectPlaySource {
    pub file: File,
    pub size: u64,
    pub content_type: &'static str,
    _permit: tokio::sync::OwnedSemaphorePermit,
}

#[derive(Debug, thiserror::Error)]
pub enum StreamingError {
    #[error("the requested media does not exist")]
    NotFound,
    #[error("the requested media is outside the approved library")]
    OutsideApprovedLibrary,
    #[error("the requested media is unavailable")]
    Unavailable,
    #[error("the streaming capacity is currently in use")]
    Busy,
}

impl From<DatabaseError> for StreamingError {
    fn from(_: DatabaseError) -> Self {
        Self::Unavailable
    }
}

pub(crate) async fn open_direct_play(
    location: Option<MediaLocation>,
    permit: tokio::sync::OwnedSemaphorePermit,
) -> Result<DirectPlaySource, StreamingError> {
    let location = location.ok_or(StreamingError::NotFound)?;
    let root = tokio::fs::canonicalize(&location.root_path)
        .await
        .map_err(|_| StreamingError::Unavailable)?;
    let path = tokio::fs::canonicalize(&location.media_path)
        .await
        .map_err(|_| StreamingError::Unavailable)?;
    if !path.starts_with(&root) || path == root {
        return Err(StreamingError::OutsideApprovedLibrary);
    }

    let file = File::open(&path)
        .await
        .map_err(|_| StreamingError::Unavailable)?;
    let metadata = file
        .metadata()
        .await
        .map_err(|_| StreamingError::Unavailable)?;
    if !metadata.is_file() {
        return Err(StreamingError::Unavailable);
    }

    Ok(DirectPlaySource {
        file,
        size: metadata.len(),
        content_type: content_type(&location.extension),
        _permit: permit,
    })
}

fn content_type(extension: &str) -> &'static str {
    match extension {
        "m4v" | "mp4" => "video/mp4",
        "mkv" => "video/x-matroska",
        "mov" => "video/quicktime",
        "webm" => "video/webm",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use crate::database::MediaLocation;

    use super::{open_direct_play, StreamingError};

    fn permit() -> tokio::sync::OwnedSemaphorePermit {
        std::sync::Arc::new(tokio::sync::Semaphore::new(1))
            .try_acquire_owned()
            .expect("permit should be available")
    }

    #[tokio::test]
    async fn opens_a_file_contained_by_the_approved_root() {
        let workspace = tempdir().expect("workspace should exist");
        let library = workspace.path().join("library");
        fs::create_dir(&library).expect("library should exist");
        let media = library.join("movie.mp4");
        fs::write(&media, b"streamed bytes").expect("fixture should exist");

        let source = open_direct_play(
            Some(MediaLocation {
                root_path: library,
                media_path: media,
                extension: "mp4".to_owned(),
            }),
            permit(),
        )
        .await
        .expect("contained media should open");

        assert_eq!(source.size, 14);
        assert_eq!(source.content_type, "video/mp4");
    }

    #[tokio::test]
    async fn rejects_a_file_outside_the_approved_root() {
        let workspace = tempdir().expect("workspace should exist");
        let library = workspace.path().join("library");
        fs::create_dir(&library).expect("library should exist");
        let media = workspace.path().join("private.mp4");
        fs::write(&media, b"private").expect("fixture should exist");

        let error = open_direct_play(
            Some(MediaLocation {
                root_path: library,
                media_path: media,
                extension: "mp4".to_owned(),
            }),
            permit(),
        )
        .await
        .expect_err("outside media must be rejected");

        assert!(matches!(error, StreamingError::OutsideApprovedLibrary));
    }
}
