use std::time::{SystemTime, UNIX_EPOCH};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

use crate::database::{LibraryDatabase, NewBrowserSession};

use super::{
    digest_token, parse_capability, validate_display_name, AuthError, TrustedPeer, LIBRARY_READ,
};

pub const SESSION_COOKIE_NAME: &str = "__Host-localstream_session";
pub const SESSION_TTL_SECONDS: u64 = 24 * 60 * 60;
const SESSION_PREFIX: &str = "ls_session_";
const SESSION_BYTES: usize = 32;
const SESSION_ENCODED_LENGTH: usize = 43;

pub struct IssuedBrowserSession {
    pub peer: TrustedPeer,
    pub session_token: String,
    pub expires_in_seconds: u64,
}

pub(crate) fn issue(
    database: &LibraryDatabase,
    display_name: &str,
) -> Result<IssuedBrowserSession, AuthError> {
    let now = unix_time()?;
    issue_at(database, display_name, now, SESSION_TTL_SECONDS)
}

pub(crate) fn issue_at(
    database: &LibraryDatabase,
    display_name: &str,
    now: i64,
    ttl_seconds: u64,
) -> Result<IssuedBrowserSession, AuthError> {
    let display_name = validate_display_name(display_name)?;
    let ttl: i64 = ttl_seconds.try_into().map_err(|_| AuthError::Unavailable)?;
    let expires_at = now.checked_add(ttl).ok_or(AuthError::Unavailable)?;
    if ttl_seconds == 0 {
        return Err(AuthError::Unavailable);
    }
    let mut session_secret = [0_u8; SESSION_BYTES];
    getrandom::fill(&mut session_secret).map_err(|_| AuthError::RandomnessUnavailable)?;
    let session_token = format!("{SESSION_PREFIX}{}", URL_SAFE_NO_PAD.encode(session_secret));
    let session_digest = digest_token(&session_token);
    let mut inaccessible_peer_secret = [0_u8; 32];
    getrandom::fill(&mut inaccessible_peer_secret).map_err(|_| AuthError::RandomnessUnavailable)?;
    let inaccessible_peer_digest = digest_token(&format!(
        "ls_peer_{}",
        URL_SAFE_NO_PAD.encode(inaccessible_peer_secret)
    ));
    let peer = TrustedPeer {
        id: uuid::Uuid::new_v4().to_string(),
        display_name: display_name.to_owned(),
        capability: super::PeerCapability::LibraryRead,
    };
    database.insert_browser_peer_and_session(&NewBrowserSession {
        peer_id: &peer.id,
        display_name: &peer.display_name,
        peer_token_digest: &inaccessible_peer_digest,
        capability: LIBRARY_READ,
        session_digest: &session_digest,
        created_at: now,
        expires_at,
    })?;

    Ok(IssuedBrowserSession {
        peer,
        session_token,
        expires_in_seconds: ttl_seconds,
    })
}

pub(crate) fn authenticate(
    database: &LibraryDatabase,
    session_token: Option<&str>,
) -> Result<TrustedPeer, AuthError> {
    authenticate_at(database, session_token, unix_time()?)
}

pub(crate) fn authenticate_at(
    database: &LibraryDatabase,
    session_token: Option<&str>,
    now: i64,
) -> Result<TrustedPeer, AuthError> {
    let session_token = session_token
        .filter(|token| valid_session_token(token))
        .ok_or_else(|| {
            if session_token.is_some() {
                AuthError::InvalidCredential
            } else {
                AuthError::MissingCredential
            }
        })?;
    database.prune_expired_browser_sessions(now)?;
    let record = database
        .browser_session_by_digest(&digest_token(session_token))?
        .ok_or(AuthError::InvalidCredential)?;
    if record.session_revoked || record.peer_revoked || now >= record.expires_at {
        return Err(AuthError::InvalidCredential);
    }
    Ok(TrustedPeer {
        id: record.peer_id,
        display_name: record.display_name,
        capability: parse_capability(&record.capability)?,
    })
}

fn valid_session_token(token: &str) -> bool {
    token.starts_with(SESSION_PREFIX)
        && token.len() == SESSION_PREFIX.len() + SESSION_ENCODED_LENGTH
        && token[SESSION_PREFIX.len()..]
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn unix_time() -> Result<i64, AuthError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| AuthError::Unavailable)?
        .as_secs()
        .try_into()
        .map_err(|_| AuthError::Unavailable)
}

#[cfg(test)]
mod tests {
    use crate::{auth::digest_token, database::LibraryDatabase};

    use super::{authenticate_at, issue_at, SESSION_PREFIX};

    #[test]
    fn issues_digest_only_session_and_enforces_expiry() {
        let database = LibraryDatabase::in_memory().expect("database should open");
        let issued =
            issue_at(&database, "Living Room Browser", 1_000, 60).expect("session should issue");

        assert!(issued.session_token.starts_with(SESSION_PREFIX));
        assert_eq!(issued.expires_in_seconds, 60);
        assert_eq!(
            authenticate_at(&database, Some(&issued.session_token), 1_059)
                .expect("session should authenticate"),
            issued.peer
        );
        assert!(authenticate_at(&database, Some(&issued.session_token), 1_060).is_err());
        assert!(database
            .browser_session_by_digest(&digest_token(&issued.session_token))
            .expect("session query should work")
            .is_none());
    }

    #[test]
    fn peer_revocation_immediately_invalidates_the_bound_session() {
        let database = LibraryDatabase::in_memory().expect("database should open");
        let issued =
            issue_at(&database, "Bedroom Browser", 2_000, 60).expect("session should issue");

        assert!(database
            .revoke_peer(&issued.peer.id)
            .expect("peer should revoke"));
        assert!(authenticate_at(&database, Some(&issued.session_token), 2_001).is_err());
        let record = database
            .browser_session_by_digest(&digest_token(&issued.session_token))
            .expect("session should query")
            .expect("revoked session should remain");
        assert!(record.peer_revoked);
        assert!(record.session_revoked);
    }

    #[test]
    fn malformed_unknown_and_unknown_capability_sessions_fail_uniformly() {
        let database = LibraryDatabase::in_memory().expect("database should open");
        let issued =
            issue_at(&database, "Kitchen Browser", 3_000, 60).expect("session should issue");
        let malformed = authenticate_at(&database, Some("invalid"), 3_001)
            .expect_err("malformed token should fail")
            .to_string();
        let unknown = authenticate_at(
            &database,
            Some("ls_session_AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"),
            3_001,
        )
        .expect_err("unknown token should fail")
        .to_string();
        assert_eq!(malformed, unknown);

        database
            .set_browser_session_capability(
                &digest_token(&issued.session_token),
                "unknown.capability",
            )
            .expect("capability should change for test");
        assert!(authenticate_at(&database, Some(&issued.session_token), 3_001).is_err());
    }
}
