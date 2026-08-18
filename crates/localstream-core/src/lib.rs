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
    pairing: auth::PairingService,
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
            pairing: auth::PairingService::default(),
            stream_permits: std::sync::Arc::new(tokio::sync::Semaphore::new(
                MAX_CONCURRENT_STREAMS,
            )),
        })
    }

    #[cfg(test)]
    fn in_memory() -> Result<Self, DatabaseError> {
        Ok(Self {
            database: database::LibraryDatabase::in_memory()?,
            pairing: auth::PairingService::default(),
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

    pub fn begin_pairing(
        &self,
        display_name: &str,
    ) -> Result<auth::PairingReceipt, auth::PairingError> {
        self.pairing.begin(display_name)
    }

    pub fn pending_pairings(&self) -> Result<Vec<auth::PendingPairing>, auth::PairingError> {
        self.pairing.pending()
    }

    pub fn approve_pairing(
        &self,
        request_id: &str,
        verification_code: &str,
    ) -> Result<(), auth::PairingError> {
        self.pairing.approve(request_id, verification_code)
    }

    pub fn reject_pairing(&self, request_id: &str) -> Result<(), auth::PairingError> {
        self.pairing.reject(request_id)
    }

    pub fn claim_pairing(
        &self,
        request_id: &str,
        claim_secret: &str,
    ) -> Result<auth::IssuedCredential, auth::PairingError> {
        self.pairing.claim(&self.database, request_id, claim_secret)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, sync::Arc, time::Duration};

    use tempfile::tempdir;

    use super::LocalStreamCore;

    fn pairing_core(ttl: Duration, capacity: usize) -> LocalStreamCore {
        LocalStreamCore {
            database: crate::database::LibraryDatabase::in_memory()
                .expect("in-memory database should open"),
            pairing: crate::auth::PairingService::for_test(ttl, capacity),
            stream_permits: Arc::new(tokio::sync::Semaphore::new(super::MAX_CONCURRENT_STREAMS)),
        }
    }

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

    #[test]
    fn pairing_requires_approval_and_issues_exactly_one_credential() {
        let core = pairing_core(Duration::from_secs(120), 4);
        let receipt = core
            .begin_pairing("Living Room TV")
            .expect("request should begin");

        assert!(receipt.request_id.starts_with("ls_pair_"));
        assert!(receipt.claim_secret.starts_with("ls_claim_"));
        assert_eq!(receipt.verification_code.len(), 6);
        assert!(receipt
            .verification_code
            .bytes()
            .all(|byte| byte.is_ascii_digit()));
        let pending = core.pending_pairings().expect("requests should list");
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].display_name, "Living Room TV");

        assert!(matches!(
            core.claim_pairing(&receipt.request_id, &receipt.claim_secret),
            Err(crate::auth::PairingError::NotApproved)
        ));
        let invalid_code = if receipt.verification_code == "000000" {
            "000001"
        } else {
            "000000"
        };
        assert!(matches!(
            core.approve_pairing(&receipt.request_id, invalid_code),
            Err(crate::auth::PairingError::InvalidVerificationCode)
        ));
        core.approve_pairing(&receipt.request_id, &receipt.verification_code)
            .expect("matching code should approve");
        assert!(core
            .pending_pairings()
            .expect("requests should list")
            .is_empty());
        assert!(matches!(
            core.claim_pairing(&receipt.request_id, "ls_claim_invalid"),
            Err(crate::auth::PairingError::InvalidClaimSecret)
        ));

        let credential = core
            .claim_pairing(&receipt.request_id, &receipt.claim_secret)
            .expect("approved request should issue");
        core.authenticate_peer(Some(&credential.bearer_token))
            .expect("issued peer should authenticate");
        assert!(matches!(
            core.claim_pairing(&receipt.request_id, &receipt.claim_secret),
            Err(crate::auth::PairingError::ReplayedRequest)
        ));
    }

    #[test]
    fn rejected_pairing_cannot_be_claimed() {
        let core = pairing_core(Duration::from_secs(120), 4);
        let receipt = core
            .begin_pairing("Unknown Browser")
            .expect("request should begin");

        core.reject_pairing(&receipt.request_id)
            .expect("pending request should reject");

        assert!(matches!(
            core.claim_pairing(&receipt.request_id, &receipt.claim_secret),
            Err(crate::auth::PairingError::Rejected)
        ));
    }

    #[test]
    fn pairing_requests_expire_and_capacity_is_bounded() {
        let expiring = pairing_core(Duration::from_millis(1), 1);
        let receipt = expiring
            .begin_pairing("Short Lived Client")
            .expect("request should begin");
        expiring.pairing.expire_for_test(&receipt.request_id);
        assert!(matches!(
            expiring.approve_pairing(&receipt.request_id, &receipt.verification_code),
            Err(crate::auth::PairingError::ExpiredRequest)
        ));

        let bounded = pairing_core(Duration::from_secs(120), 1);
        let first = bounded
            .begin_pairing("First Client")
            .expect("first request should begin");
        assert!(matches!(
            bounded.begin_pairing("Second Client"),
            Err(crate::auth::PairingError::CapacityReached)
        ));
        bounded
            .reject_pairing(&first.request_id)
            .expect("rejection should release capacity");
        bounded
            .begin_pairing("Second Client")
            .expect("released capacity should be reusable");
    }
}
