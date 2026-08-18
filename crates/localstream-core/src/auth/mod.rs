use std::time::{SystemTime, UNIX_EPOCH};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::Serialize;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::{database::LibraryDatabase, DatabaseError};

mod pairing;
pub(crate) use pairing::PairingService;
pub use pairing::{PairingError, PairingReceipt, PendingPairing};

const TOKEN_PREFIX: &str = "ls_peer_";
const TOKEN_BYTES: usize = 32;
const TOKEN_ENCODED_LENGTH: usize = 43;
const LIBRARY_READ: &str = "library.read";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PeerCapability {
    LibraryRead,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrustedPeer {
    pub id: String,
    pub display_name: String,
    pub capability: PeerCapability,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrustedPeerSummary {
    pub id: String,
    pub display_name: String,
    pub capability: PeerCapability,
    pub created_at: i64,
}

pub struct IssuedCredential {
    pub peer: TrustedPeer,
    pub bearer_token: String,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("a peer display name is required")]
    InvalidDisplayName,
    #[error("a bearer credential is required")]
    MissingCredential,
    #[error("the bearer credential is invalid")]
    InvalidCredential,
    #[error("the peer credential has been revoked")]
    RevokedCredential,
    #[error("secure credential generation is unavailable")]
    RandomnessUnavailable,
    #[error("the credential store is unavailable")]
    Unavailable,
}

impl From<DatabaseError> for AuthError {
    fn from(_: DatabaseError) -> Self {
        Self::Unavailable
    }
}

pub(crate) fn issue_credential(
    database: &LibraryDatabase,
    display_name: &str,
) -> Result<IssuedCredential, AuthError> {
    let display_name = validate_display_name(display_name)?;

    let mut secret = [0_u8; TOKEN_BYTES];
    getrandom::fill(&mut secret).map_err(|_| AuthError::RandomnessUnavailable)?;
    let bearer_token = format!("{TOKEN_PREFIX}{}", URL_SAFE_NO_PAD.encode(secret));
    let digest = digest_token(&bearer_token);
    let peer = TrustedPeer {
        id: uuid::Uuid::new_v4().to_string(),
        display_name: display_name.to_owned(),
        capability: PeerCapability::LibraryRead,
    };
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| AuthError::Unavailable)?
        .as_secs()
        .try_into()
        .map_err(|_| AuthError::Unavailable)?;
    database.insert_peer(
        &peer.id,
        &peer.display_name,
        &digest,
        LIBRARY_READ,
        created_at,
    )?;

    Ok(IssuedCredential { peer, bearer_token })
}

pub(super) fn validate_display_name(display_name: &str) -> Result<&str, AuthError> {
    let display_name = display_name.trim();
    if display_name.is_empty()
        || display_name.chars().count() > 100
        || display_name.chars().any(char::is_control)
    {
        return Err(AuthError::InvalidDisplayName);
    }
    Ok(display_name)
}

pub(crate) fn authenticate(
    database: &LibraryDatabase,
    bearer_token: Option<&str>,
) -> Result<TrustedPeer, AuthError> {
    let bearer_token = bearer_token
        .filter(|token| {
            token.starts_with(TOKEN_PREFIX)
                && token.len() == TOKEN_PREFIX.len() + TOKEN_ENCODED_LENGTH
                && token[TOKEN_PREFIX.len()..]
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        })
        .ok_or_else(|| {
            if bearer_token.is_some() {
                AuthError::InvalidCredential
            } else {
                AuthError::MissingCredential
            }
        })?;
    let record = database
        .peer_by_digest(&digest_token(bearer_token))?
        .ok_or(AuthError::InvalidCredential)?;
    if record.revoked {
        return Err(AuthError::RevokedCredential);
    }
    let capability = parse_capability(&record.capability)?;
    Ok(TrustedPeer {
        id: record.id,
        display_name: record.display_name,
        capability,
    })
}

pub(crate) fn active_peers(
    database: &LibraryDatabase,
) -> Result<Vec<TrustedPeerSummary>, AuthError> {
    database
        .active_peers()?
        .into_iter()
        .map(|record| {
            if record.revoked {
                return Err(AuthError::Unavailable);
            }
            Ok(TrustedPeerSummary {
                id: record.id,
                display_name: record.display_name,
                capability: parse_capability(&record.capability)?,
                created_at: record.created_at,
            })
        })
        .collect()
}

fn parse_capability(value: &str) -> Result<PeerCapability, AuthError> {
    match value {
        LIBRARY_READ => Ok(PeerCapability::LibraryRead),
        _ => Err(AuthError::InvalidCredential),
    }
}

pub(super) fn digest_token(token: &str) -> [u8; 32] {
    Sha256::digest(token.as_bytes()).into()
}
