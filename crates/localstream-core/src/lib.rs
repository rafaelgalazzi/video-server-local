use serde::Serialize;
use thiserror::Error;

pub mod auth;
mod database;
pub mod media;
pub mod server;
pub mod streaming;

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
    stream_permits: std::sync::Arc<tokio::sync::Semaphore>,
}

const MAX_CONCURRENT_STREAMS: usize = 8;

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
            stream_permits: std::sync::Arc::new(tokio::sync::Semaphore::new(
                MAX_CONCURRENT_STREAMS,
            )),
        })
    }

    #[cfg(test)]
    fn in_memory() -> Result<Self, DatabaseError> {
        Ok(Self {
            database: database::LibraryDatabase::in_memory()?,
            stream_permits: std::sync::Arc::new(tokio::sync::Semaphore::new(
                MAX_CONCURRENT_STREAMS,
            )),
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

    pub async fn open_direct_play(
        &self,
        media_id: &str,
    ) -> Result<streaming::DirectPlaySource, streaming::StreamingError> {
        let location = self.database.media_location(media_id)?;
        let permit = self
            .stream_permits
            .clone()
            .try_acquire_owned()
            .map_err(|_| streaming::StreamingError::Busy)?;
        streaming::open_direct_play(location, permit).await
    }

    pub fn issue_peer_credential(
        &self,
        display_name: &str,
    ) -> Result<auth::IssuedCredential, auth::AuthError> {
        auth::issue_credential(&self.database, display_name)
    }

    pub fn authenticate_peer(
        &self,
        bearer_token: Option<&str>,
    ) -> Result<auth::TrustedPeer, auth::AuthError> {
        auth::authenticate(&self.database, bearer_token)
    }

    pub fn revoke_peer(&self, peer_id: &str) -> Result<bool, auth::AuthError> {
        self.database.revoke_peer(peer_id).map_err(Into::into)
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

    #[tokio::test]
    async fn limits_concurrent_direct_play_sources() {
        let workspace = tempdir().expect("temporary workspace should be created");
        let library = workspace.path().join("Videos");
        fs::create_dir(&library).expect("library should be created");
        fs::write(library.join("Movie.mp4"), b"video").expect("video should be created");
        let core = LocalStreamCore::in_memory().expect("in-memory core should open");
        let scan = core
            .scan_and_persist_library(&library)
            .expect("scan should persist");
        let id = &scan.items[0].id;
        let mut sources = Vec::new();
        for _ in 0..super::MAX_CONCURRENT_STREAMS {
            sources.push(
                core.open_direct_play(id)
                    .await
                    .expect("capacity should remain"),
            );
        }

        let error = core
            .open_direct_play(id)
            .await
            .expect_err("capacity must be enforced");
        assert!(matches!(error, crate::streaming::StreamingError::Busy));

        drop(sources);
        core.open_direct_play(id)
            .await
            .expect("released capacity should be reusable");
    }

    #[test]
    fn issues_authenticates_and_revokes_a_peer_credential() {
        let core = LocalStreamCore::in_memory().expect("in-memory core should open");
        let issued = core
            .issue_peer_credential("Living Room TV")
            .expect("credential should issue");

        assert!(issued.bearer_token.starts_with("ls_peer_"));
        assert_eq!(issued.peer.display_name, "Living Room TV");
        assert!(matches!(
            issued.peer.capability,
            crate::auth::PeerCapability::LibraryRead
        ));
        assert!(matches!(
            core.authenticate_peer(None),
            Err(crate::auth::AuthError::MissingCredential)
        ));
        assert!(matches!(
            core.authenticate_peer(Some("ls_peer_invalid")),
            Err(crate::auth::AuthError::InvalidCredential)
        ));

        let authenticated = core
            .authenticate_peer(Some(&issued.bearer_token))
            .expect("issued credential should authenticate");
        assert_eq!(authenticated, issued.peer);

        assert!(core
            .revoke_peer(&issued.peer.id)
            .expect("revocation should persist"));
        assert!(matches!(
            core.authenticate_peer(Some(&issued.bearer_token)),
            Err(crate::auth::AuthError::RevokedCredential)
        ));
        assert!(!core
            .revoke_peer(&issued.peer.id)
            .expect("repeat revocation should be safe"));
    }

    #[test]
    fn peer_credential_and_revocation_survive_restart_without_plaintext_storage() {
        let workspace = tempdir().expect("temporary workspace should be created");
        let database = workspace.path().join("localstream.sqlite3");
        let (peer_id, token) = {
            let core = LocalStreamCore::open(&database).expect("database should open");
            let issued = core
                .issue_peer_credential("Bedroom Tablet")
                .expect("credential should issue");
            (issued.peer.id, issued.bearer_token)
        };

        {
            let core = LocalStreamCore::open(&database).expect("database should reopen");
            core.authenticate_peer(Some(&token))
                .expect("credential should survive restart");
            assert!(core.revoke_peer(&peer_id).expect("peer should revoke"));
        }

        let core = LocalStreamCore::open(&database).expect("database should reopen again");
        assert!(matches!(
            core.authenticate_peer(Some(&token)),
            Err(crate::auth::AuthError::RevokedCredential)
        ));
        let database_bytes = fs::read(database).expect("database should be readable");
        assert!(!database_bytes
            .windows(token.len())
            .any(|window| window == token.as_bytes()));
    }

    #[test]
    fn rejects_invalid_peer_display_names() {
        let core = LocalStreamCore::in_memory().expect("in-memory core should open");
        assert!(matches!(
            core.issue_peer_credential("   "),
            Err(crate::auth::AuthError::InvalidDisplayName)
        ));
        assert!(matches!(
            core.issue_peer_credential(&"x".repeat(101)),
            Err(crate::auth::AuthError::InvalidDisplayName)
        ));
        assert!(matches!(
            core.issue_peer_credential("Living\nRoom"),
            Err(crate::auth::AuthError::InvalidDisplayName)
        ));
    }
}
