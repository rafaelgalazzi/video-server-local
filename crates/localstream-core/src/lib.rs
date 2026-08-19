use serde::Serialize;
use thiserror::Error;

pub mod auth;
mod database;
pub mod lan;
pub mod media;
pub mod media_tools;
pub mod native_client;
pub mod node_identity;
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
    pairing_rate_limiter: auth::PairingRateLimiter,
    stream_permits: std::sync::Arc<tokio::sync::Semaphore>,
}

const MAX_CONCURRENT_STREAMS: usize = 8;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error(transparent)]
    Scan(#[from] LibraryScanError),
    #[error(transparent)]
    Database(#[from] DatabaseError),
    #[error(transparent)]
    MediaTools(#[from] media_tools::ToolDiscoveryError),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum IdentityResetError {
    #[error("trusted device revocation is unavailable")]
    RevocationUnavailable,
    #[error("the protected node identity store is unavailable")]
    StoreUnavailable,
}

impl LocalStreamCore {
    pub fn open(database_path: impl AsRef<std::path::Path>) -> Result<Self, DatabaseError> {
        Ok(Self {
            database: database::LibraryDatabase::open(database_path.as_ref())?,
            pairing: auth::PairingService::default(),
            pairing_rate_limiter: auth::PairingRateLimiter::default(),
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
            pairing_rate_limiter: auth::PairingRateLimiter::default(),
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

    pub async fn scan_and_persist_library_with_probe(
        &self,
        approved_directory: impl AsRef<std::path::Path>,
        cancellation: tokio_util::sync::CancellationToken,
    ) -> Result<LibraryScan, CoreError> {
        let ffprobe = media_tools::MediaToolPaths::discover_ffprobe().await?;
        let mut scan = media::scan_approved_directory_records(approved_directory.as_ref())?;
        for media in &mut scan.items {
            match media_tools::probe_media(
                &ffprobe,
                &media.item.id,
                &media.path,
                cancellation.child_token(),
            )
            .await
            {
                Ok(probe) => {
                    media.item.metadata = Some(probe.metadata);
                    media.item.probe_status = media::ProbeStatus::Available;
                    media.track_mappings = probe.mappings;
                }
                Err(media_tools::ProbeError::Process(media_tools::ProcessError::Cancelled)) => {
                    return Err(CoreError::MediaTools(
                        media_tools::ToolDiscoveryError::Unavailable {
                            tool: "ffprobe",
                            source: media_tools::ProcessError::Cancelled,
                        },
                    ));
                }
                Err(_) => media.item.probe_status = media::ProbeStatus::Unavailable,
            }
        }
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

    pub fn authenticate_browser_session(
        &self,
        session_token: Option<&str>,
    ) -> Result<auth::TrustedPeer, auth::AuthError> {
        auth::session::authenticate(&self.database, session_token)
    }

    pub fn revoke_peer(&self, peer_id: &str) -> Result<bool, auth::AuthError> {
        self.database.revoke_peer(peer_id).map_err(Into::into)
    }

    pub fn trusted_peers(&self) -> Result<Vec<auth::TrustedPeerSummary>, auth::AuthError> {
        auth::active_peers(&self.database)
    }

    pub fn reset_node_identity<S>(&self, store: &S) -> Result<usize, IdentityResetError>
    where
        S: node_identity::NodeSecretStore,
    {
        let revoked = self
            .database
            .revoke_all_peers()
            .map_err(|_| IdentityResetError::RevocationUnavailable)?;
        store
            .delete()
            .map_err(|_| IdentityResetError::StoreUnavailable)?;
        Ok(revoked)
    }

    pub fn begin_pairing(
        &self,
        display_name: &str,
    ) -> Result<auth::PairingReceipt, auth::PairingError> {
        self.pairing.begin(display_name)
    }

    pub fn check_pairing_attempt(
        &self,
        kind: auth::PairingAttemptKind,
        remote: std::net::SocketAddr,
    ) -> auth::RateLimitDecision {
        self.pairing_rate_limiter.check(kind, remote)
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

    pub fn claim_browser_pairing(
        &self,
        request_id: &str,
        claim_secret: &str,
    ) -> Result<auth::IssuedBrowserSession, auth::PairingError> {
        self.pairing
            .claim_browser(&self.database, request_id, claim_secret)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, sync::Arc, time::Duration};

    use tempfile::tempdir;

    use super::LocalStreamCore;

    #[derive(Default)]
    struct ResetStore {
        deleted: std::sync::atomic::AtomicBool,
        fail: bool,
    }

    impl crate::node_identity::NodeSecretStore for ResetStore {
        fn load(&self) -> Result<Option<Vec<u8>>, crate::node_identity::SecretStoreError> {
            Ok(None)
        }

        fn store(&self, _secret: &[u8]) -> Result<(), crate::node_identity::SecretStoreError> {
            Ok(())
        }

        fn delete(&self) -> Result<(), crate::node_identity::SecretStoreError> {
            if self.fail {
                return Err(crate::node_identity::SecretStoreError::Unavailable);
            }
            self.deleted
                .store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
    }

    fn pairing_core(ttl: Duration, capacity: usize) -> LocalStreamCore {
        LocalStreamCore {
            database: crate::database::LibraryDatabase::in_memory()
                .expect("in-memory database should open"),
            pairing: crate::auth::PairingService::for_test(ttl, capacity),
            pairing_rate_limiter: crate::auth::PairingRateLimiter::default(),
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
    async fn probes_persists_and_restores_mkv_tracks_while_isolating_corrupt_media() {
        if std::process::Command::new("ffmpeg")
            .arg("-version")
            .output()
            .is_err()
            || std::process::Command::new("ffprobe")
                .arg("-version")
                .output()
                .is_err()
        {
            return;
        }

        let workspace = tempdir().expect("temporary workspace should be created");
        let library = workspace.path().join("Videos");
        let database = workspace.path().join("localstream.sqlite3");
        fs::create_dir(&library).expect("library should be created");
        let subtitles = library.join("captions.srt");
        fs::write(&subtitles, "1\n00:00:00,000 --> 00:00:00,150\nHello\n")
            .expect("subtitle fixture should be written");
        let movie = library.join("Dual Audio.mkv");
        let generated = std::process::Command::new("ffmpeg")
            .args([
                "-v",
                "error",
                "-y",
                "-f",
                "lavfi",
                "-i",
                "color=size=64x64:duration=0.2",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:duration=0.2",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=880:duration=0.2",
                "-i",
            ])
            .arg(&subtitles)
            .args([
                "-map",
                "0:v",
                "-map",
                "1:a",
                "-map",
                "2:a",
                "-map",
                "3:s",
                "-c:v",
                "mpeg4",
                "-c:a",
                "aac",
                "-c:s",
                "srt",
                "-metadata:s:a:0",
                "language=eng",
                "-metadata:s:a:1",
                "language=por",
                "-metadata:s:s:0",
                "language=eng",
            ])
            .arg(&movie)
            .status()
            .expect("ffmpeg fixture command should start");
        assert!(generated.success(), "ffmpeg fixture should generate");
        fs::write(library.join("Corrupt.mkv"), b"not media")
            .expect("corrupt fixture should be written");

        {
            let core = LocalStreamCore::open(&database).expect("database should open");
            let scan = core
                .scan_and_persist_library_with_probe(
                    &library,
                    tokio_util::sync::CancellationToken::new(),
                )
                .await
                .expect("one corrupt item must not abort the scan");
            assert_eq!(scan.items.len(), 2);
            let valid = scan
                .items
                .iter()
                .find(|item| item.title == "Dual Audio")
                .unwrap();
            let metadata = valid
                .metadata
                .as_ref()
                .expect("metadata should be available");
            assert_eq!(valid.probe_status, crate::media::ProbeStatus::Available);
            assert_eq!(metadata.container, "matroska");
            assert_eq!(metadata.audio_tracks.len(), 2);
            assert_eq!(metadata.subtitle_tracks.len(), 1);
            let corrupt = scan
                .items
                .iter()
                .find(|item| item.title == "Corrupt")
                .unwrap();
            assert_eq!(corrupt.probe_status, crate::media::ProbeStatus::Unavailable);
            assert!(corrupt.metadata.is_none());
        }

        let reopened = LocalStreamCore::open(&database).expect("database should reopen");
        let restored = reopened.current_library().unwrap().unwrap();
        let metadata = restored
            .items
            .iter()
            .find(|item| item.title == "Dual Audio")
            .and_then(|item| item.metadata.as_ref())
            .expect("metadata should survive restart");
        assert_eq!(metadata.audio_tracks.len(), 2);
        assert_eq!(metadata.audio_tracks[1].language.as_deref(), Some("por"));
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
    fn browser_session_survives_restart_without_plaintext_storage_and_tracks_revocation() {
        let workspace = tempdir().expect("temporary workspace should be created");
        let database = workspace.path().join("localstream.sqlite3");
        let (peer_id, session_token) = {
            let core = LocalStreamCore::open(&database).expect("database should open");
            let receipt = core
                .begin_pairing("Restart Browser")
                .expect("pairing should begin");
            core.approve_pairing(&receipt.request_id, &receipt.verification_code)
                .expect("pairing should approve");
            let session = core
                .claim_browser_pairing(&receipt.request_id, &receipt.claim_secret)
                .expect("browser session should issue");
            (session.peer.id, session.session_token)
        };

        let database_bytes = fs::read(&database).expect("database should be readable");
        assert!(!database_bytes
            .windows(session_token.len())
            .any(|window| window == session_token.as_bytes()));
        {
            let core = LocalStreamCore::open(&database).expect("database should reopen");
            core.authenticate_browser_session(Some(&session_token))
                .expect("session should survive restart");
            assert!(core.revoke_peer(&peer_id).expect("peer should revoke"));
        }
        let core = LocalStreamCore::open(&database).expect("database should reopen again");
        assert!(core
            .authenticate_browser_session(Some(&session_token))
            .is_err());
    }

    #[test]
    fn lists_only_safe_active_peer_metadata_and_persists_revocation() {
        let workspace = tempdir().expect("temporary workspace should be created");
        let database = workspace.path().join("localstream.sqlite3");
        let (revoked_id, active_id) = {
            let core = LocalStreamCore::open(&database).expect("database should open");
            let revoked = core
                .issue_peer_credential("Old Television")
                .expect("first credential should issue");
            let active = core
                .issue_peer_credential("Kitchen Tablet")
                .expect("second credential should issue");
            assert!(core
                .revoke_peer(&revoked.peer.id)
                .expect("peer should revoke"));
            (revoked.peer.id, active.peer.id)
        };

        let core = LocalStreamCore::open(&database).expect("database should reopen");
        let peers = core.trusted_peers().expect("active peers should load");

        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].id, active_id);
        assert_ne!(peers[0].id, revoked_id);
        assert_eq!(peers[0].display_name, "Kitchen Tablet");
        assert!(peers[0].created_at > 0);
        let json = serde_json::to_string(&peers).expect("safe peers should serialize");
        assert!(!json.contains("token"));
        assert!(!json.contains("digest"));
        assert!(!json.contains("path"));
        assert!(!core
            .revoke_peer(&revoked_id)
            .expect("repeat revocation should be idempotent"));
    }

    #[test]
    fn identity_reset_revokes_all_peers_before_deleting_the_root() {
        let core = LocalStreamCore::in_memory().expect("in-memory core should open");
        let first = core
            .issue_peer_credential("Living Room TV")
            .expect("first credential should issue");
        let second = core
            .issue_peer_credential("Bedroom Tablet")
            .expect("second credential should issue");
        let store = ResetStore::default();

        let revoked = core
            .reset_node_identity(&store)
            .expect("identity should reset");

        assert_eq!(revoked, 2);
        assert!(store.deleted.load(std::sync::atomic::Ordering::SeqCst));
        assert!(core.trusted_peers().expect("peers should load").is_empty());
        assert!(core.authenticate_peer(Some(&first.bearer_token)).is_err());
        assert!(core.authenticate_peer(Some(&second.bearer_token)).is_err());
    }

    #[test]
    fn identity_reset_keeps_peers_revoked_when_protected_deletion_fails() {
        let core = LocalStreamCore::in_memory().expect("in-memory core should open");
        let issued = core
            .issue_peer_credential("Living Room TV")
            .expect("credential should issue");
        let store = ResetStore {
            fail: true,
            ..ResetStore::default()
        };

        let error = core
            .reset_node_identity(&store)
            .expect_err("store failure should fail reset");

        assert_eq!(error, super::IdentityResetError::StoreUnavailable);
        assert!(core.trusted_peers().expect("peers should load").is_empty());
        assert!(core.authenticate_peer(Some(&issued.bearer_token)).is_err());
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
