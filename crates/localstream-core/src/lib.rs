use serde::Serialize;
use thiserror::Error;

mod database;
pub mod media;
pub mod server;

pub use database::DatabaseError;
pub use media::{LibraryScan, LibraryScanError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub local_first: bool,
}

#[derive(Debug)]
pub struct LocalStreamCore {
    database: database::LibraryDatabase,
}

#[derive(Debug, Error)]
pub enum CoreError {
    #[error(transparent)]
    Scan(#[from] LibraryScanError),
    #[error(transparent)]
    Database(#[from] DatabaseError),
}

impl LocalStreamCore {
    pub fn open(database_path: impl AsRef<std::path::Path>) -> Result<Self, DatabaseError> {
        Ok(Self {
            database: database::LibraryDatabase::open(database_path.as_ref())?,
        })
    }

    #[cfg(test)]
    fn in_memory() -> Result<Self, DatabaseError> {
        Ok(Self {
            database: database::LibraryDatabase::in_memory()?,
        })
    }

    #[must_use]
    pub const fn app_info(&self) -> AppInfo {
        AppInfo {
            name: "LocalStream",
            version: env!("CARGO_PKG_VERSION"),
            local_first: true,
        }
    }

    pub fn scan_library(
        &self,
        approved_directory: impl AsRef<std::path::Path>,
    ) -> Result<LibraryScan, LibraryScanError> {
        media::scan_approved_directory(approved_directory.as_ref())
    }

    pub fn scan_and_persist_library(
        &self,
        approved_directory: impl AsRef<std::path::Path>,
    ) -> Result<LibraryScan, CoreError> {
        let scan = media::scan_approved_directory_records(approved_directory.as_ref())?;
        self.database.replace_library(&scan)?;
        Ok(scan.public_view())
    }

    pub fn current_library(&self) -> Result<Option<LibraryScan>, DatabaseError> {
        self.database.current_library()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::LocalStreamCore;

    #[test]
    fn exposes_local_first_application_identity() {
        let core = LocalStreamCore::in_memory().expect("in-memory core should open");
        let info = core.app_info();

        assert_eq!(info.name, "LocalStream");
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
        assert!(info.local_first);
    }

    #[test]
    fn restores_the_current_library_from_sqlite() {
        let workspace = tempdir().expect("temporary workspace should be created");
        let library = workspace.path().join("Videos");
        let database = workspace.path().join("localstream.sqlite3");
        fs::create_dir(&library).expect("library should be created");
        fs::write(library.join("Movie.mp4"), b"video").expect("video should be created");

        {
            let core = LocalStreamCore::open(&database).expect("database should open");
            let scan = core
                .scan_and_persist_library(&library)
                .expect("scan should persist");
            assert_eq!(scan.items.len(), 1);
        }

        let reopened = LocalStreamCore::open(&database).expect("database should reopen");
        let restored = reopened
            .current_library()
            .expect("library should load")
            .expect("current library should exist");

        assert_eq!(restored.library_name, "Videos");
        assert_eq!(restored.items.len(), 1);
        assert_eq!(restored.items[0].title, "Movie");
    }

    #[test]
    fn rescan_reconciles_deleted_and_new_media() {
        let workspace = tempdir().expect("temporary workspace should be created");
        let library = workspace.path().join("Videos");
        fs::create_dir(&library).expect("library should be created");
        let old_video = library.join("Old.mp4");
        fs::write(&old_video, b"old").expect("old video should be created");
        let core = LocalStreamCore::in_memory().expect("in-memory core should open");
        core.scan_and_persist_library(&library)
            .expect("first scan should persist");

        fs::remove_file(old_video).expect("old video should be removed");
        fs::write(library.join("New.mkv"), b"new").expect("new video should be created");
        core.scan_and_persist_library(&library)
            .expect("second scan should persist");

        let restored = core
            .current_library()
            .expect("library should load")
            .expect("current library should exist");
        assert_eq!(restored.items.len(), 1);
        assert_eq!(restored.items[0].title, "New");
    }
}
