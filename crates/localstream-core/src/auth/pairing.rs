use std::{
    collections::{HashMap, VecDeque},
    fmt,
    sync::Mutex,
    time::{Duration, Instant},
};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::Serialize;
use subtle::ConstantTimeEq;
use thiserror::Error;

use crate::database::LibraryDatabase;

use super::{digest_token, issue_credential, validate_display_name, AuthError, IssuedCredential};

const DEFAULT_TTL: Duration = Duration::from_secs(120);
const MAX_ACTIVE_REQUESTS: usize = 32;
const MAX_TOMBSTONES: usize = 64;
const REQUEST_BYTES: usize = 16;
const CLAIM_BYTES: usize = 32;
const CLAIM_PREFIX: &str = "ls_claim_";
const CLAIM_ENCODED_LENGTH: usize = 43;

pub struct PairingReceipt {
    pub request_id: String,
    pub verification_code: String,
    pub claim_secret: String,
    pub expires_in_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingPairing {
    pub request_id: String,
    pub display_name: String,
    pub verification_code: String,
    pub expires_in_seconds: u64,
}

#[derive(Debug, Error)]
pub enum PairingError {
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error("pairing request capacity has been reached")]
    CapacityReached,
    #[error("the pairing request does not exist")]
    UnknownRequest,
    #[error("the pairing request has expired")]
    ExpiredRequest,
    #[error("the pairing verification code is invalid")]
    InvalidVerificationCode,
    #[error("the pairing claim secret is invalid")]
    InvalidClaimSecret,
    #[error("the pairing request has not been approved")]
    NotApproved,
    #[error("the pairing request was rejected")]
    Rejected,
    #[error("the pairing request has already been decided")]
    AlreadyDecided,
    #[error("the pairing request has already been consumed")]
    ReplayedRequest,
    #[error("the pairing request store is unavailable")]
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RequestStatus {
    Pending,
    Approved,
    Claiming,
}

struct PairingRequest {
    display_name: String,
    verification_code: String,
    claim_digest: [u8; 32],
    expires_at: Instant,
    status: RequestStatus,
}

#[derive(Default)]
struct PairingState {
    requests: HashMap<String, PairingRequest>,
    tombstones: VecDeque<(String, TerminalStatus)>,
}

#[derive(Debug, Clone, Copy)]
enum TerminalStatus {
    Consumed,
    Rejected,
    Expired,
}

pub(crate) struct PairingService {
    state: Mutex<PairingState>,
    ttl: Duration,
    capacity: usize,
}

impl fmt::Debug for PairingService {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PairingService")
            .field("ttl", &self.ttl)
            .field("capacity", &self.capacity)
            .finish_non_exhaustive()
    }
}

impl Default for PairingService {
    fn default() -> Self {
        Self::new(DEFAULT_TTL, MAX_ACTIVE_REQUESTS)
    }
}

impl PairingService {
    fn new(ttl: Duration, capacity: usize) -> Self {
        Self {
            state: Mutex::new(PairingState::default()),
            ttl,
            capacity,
        }
    }

    pub(crate) fn begin(&self, display_name: &str) -> Result<PairingReceipt, PairingError> {
        let display_name = validate_display_name(display_name)?.to_owned();
        let now = Instant::now();
        let mut state = self.state.lock().map_err(|_| PairingError::Unavailable)?;
        prune_expired(&mut state, now);
        if state.requests.len() >= self.capacity {
            return Err(PairingError::CapacityReached);
        }

        let request_id = loop {
            let candidate = format!("ls_pair_{}", random_base64::<REQUEST_BYTES>()?);
            if !state.requests.contains_key(&candidate)
                && !state
                    .tombstones
                    .iter()
                    .any(|(terminal_id, _)| terminal_id == &candidate)
            {
                break candidate;
            }
        };
        let claim_secret = format!("{CLAIM_PREFIX}{}", random_base64::<CLAIM_BYTES>()?);
        let verification_code = random_verification_code()?;
        state.requests.insert(
            request_id.clone(),
            PairingRequest {
                display_name,
                verification_code: verification_code.clone(),
                claim_digest: digest_token(&claim_secret),
                expires_at: now + self.ttl,
                status: RequestStatus::Pending,
            },
        );

        Ok(PairingReceipt {
            request_id,
            verification_code,
            claim_secret,
            expires_in_seconds: self.ttl.as_secs(),
        })
    }

    pub(crate) fn pending(&self) -> Result<Vec<PendingPairing>, PairingError> {
        let now = Instant::now();
        let mut state = self.state.lock().map_err(|_| PairingError::Unavailable)?;
        prune_expired(&mut state, now);
        let mut requests = state
            .requests
            .iter()
            .filter(|(_, request)| request.status == RequestStatus::Pending)
            .map(|(request_id, request)| PendingPairing {
                request_id: request_id.clone(),
                display_name: request.display_name.clone(),
                verification_code: request.verification_code.clone(),
                expires_in_seconds: request.expires_at.saturating_duration_since(now).as_secs(),
            })
            .collect::<Vec<_>>();
        requests.sort_unstable_by(|left, right| left.request_id.cmp(&right.request_id));
        Ok(requests)
    }

    pub(crate) fn approve(
        &self,
        request_id: &str,
        verification_code: &str,
    ) -> Result<(), PairingError> {
        let now = Instant::now();
        let mut state = self.state.lock().map_err(|_| PairingError::Unavailable)?;
        ensure_not_replayed(&state, request_id)?;
        let request = state
            .requests
            .get_mut(request_id)
            .ok_or(PairingError::UnknownRequest)?;
        if now >= request.expires_at {
            state.requests.remove(request_id);
            add_tombstone(&mut state, request_id.to_owned(), TerminalStatus::Expired);
            return Err(PairingError::ExpiredRequest);
        }
        if request.status != RequestStatus::Pending {
            return Err(PairingError::AlreadyDecided);
        }
        if request.verification_code != verification_code {
            return Err(PairingError::InvalidVerificationCode);
        }
        request.status = RequestStatus::Approved;
        Ok(())
    }

    pub(crate) fn reject(&self, request_id: &str) -> Result<(), PairingError> {
        let mut state = self.state.lock().map_err(|_| PairingError::Unavailable)?;
        ensure_not_replayed(&state, request_id)?;
        let request = state
            .requests
            .remove(request_id)
            .ok_or(PairingError::UnknownRequest)?;
        if Instant::now() >= request.expires_at {
            add_tombstone(&mut state, request_id.to_owned(), TerminalStatus::Expired);
            return Err(PairingError::ExpiredRequest);
        }
        if request.status != RequestStatus::Pending {
            state.requests.insert(request_id.to_owned(), request);
            return Err(PairingError::AlreadyDecided);
        }
        add_tombstone(&mut state, request_id.to_owned(), TerminalStatus::Rejected);
        Ok(())
    }

    pub(crate) fn claim(
        &self,
        database: &LibraryDatabase,
        request_id: &str,
        claim_secret: &str,
    ) -> Result<IssuedCredential, PairingError> {
        let display_name = {
            let mut state = self.state.lock().map_err(|_| PairingError::Unavailable)?;
            ensure_not_replayed(&state, request_id)?;
            let request = state
                .requests
                .get_mut(request_id)
                .ok_or(PairingError::UnknownRequest)?;
            if Instant::now() >= request.expires_at {
                state.requests.remove(request_id);
                add_tombstone(&mut state, request_id.to_owned(), TerminalStatus::Expired);
                return Err(PairingError::ExpiredRequest);
            }
            if request.status == RequestStatus::Pending {
                return Err(PairingError::NotApproved);
            }
            if request.status == RequestStatus::Claiming {
                return Err(PairingError::ReplayedRequest);
            }
            if !valid_claim_secret(claim_secret)
                || !bool::from(
                    digest_token(claim_secret)
                        .as_slice()
                        .ct_eq(request.claim_digest.as_slice()),
                )
            {
                return Err(PairingError::InvalidClaimSecret);
            }
            request.status = RequestStatus::Claiming;
            request.display_name.clone()
        };

        match issue_credential(database, &display_name) {
            Ok(credential) => {
                let mut state = self.state.lock().map_err(|_| PairingError::Unavailable)?;
                state.requests.remove(request_id);
                add_tombstone(&mut state, request_id.to_owned(), TerminalStatus::Consumed);
                Ok(credential)
            }
            Err(error) => {
                if let Ok(mut state) = self.state.lock() {
                    if let Some(request) = state.requests.get_mut(request_id) {
                        request.status = RequestStatus::Approved;
                    }
                }
                Err(error.into())
            }
        }
    }
}

fn random_base64<const SIZE: usize>() -> Result<String, PairingError> {
    let mut bytes = [0_u8; SIZE];
    getrandom::fill(&mut bytes).map_err(|_| AuthError::RandomnessUnavailable)?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn random_verification_code() -> Result<String, PairingError> {
    const CODE_SPACE: u32 = 1_000_000;
    const ACCEPT_LIMIT: u32 = u32::MAX - (u32::MAX % CODE_SPACE);
    loop {
        let mut bytes = [0_u8; 4];
        getrandom::fill(&mut bytes).map_err(|_| AuthError::RandomnessUnavailable)?;
        let value = u32::from_le_bytes(bytes);
        if value < ACCEPT_LIMIT {
            return Ok(format!("{:06}", value % CODE_SPACE));
        }
    }
}

fn valid_claim_secret(secret: &str) -> bool {
    secret.starts_with(CLAIM_PREFIX)
        && secret.len() == CLAIM_PREFIX.len() + CLAIM_ENCODED_LENGTH
        && secret[CLAIM_PREFIX.len()..]
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn ensure_not_replayed(state: &PairingState, request_id: &str) -> Result<(), PairingError> {
    match state
        .tombstones
        .iter()
        .find(|(terminal_id, _)| terminal_id == request_id)
        .map(|(_, status)| status)
    {
        Some(TerminalStatus::Consumed) => Err(PairingError::ReplayedRequest),
        Some(TerminalStatus::Rejected) => Err(PairingError::Rejected),
        Some(TerminalStatus::Expired) => Err(PairingError::ExpiredRequest),
        None => Ok(()),
    }
}

fn prune_expired(state: &mut PairingState, now: Instant) {
    let expired = state
        .requests
        .iter()
        .filter(|(_, request)| now >= request.expires_at)
        .map(|(request_id, _)| request_id.clone())
        .collect::<Vec<_>>();
    for request_id in expired {
        state.requests.remove(&request_id);
        add_tombstone(state, request_id, TerminalStatus::Expired);
    }
}

fn add_tombstone(state: &mut PairingState, request_id: String, status: TerminalStatus) {
    state.tombstones.push_back((request_id, status));
    while state.tombstones.len() > MAX_TOMBSTONES {
        state.tombstones.pop_front();
    }
}

#[cfg(test)]
impl PairingService {
    pub(crate) fn for_test(ttl: Duration, capacity: usize) -> Self {
        Self::new(ttl, capacity)
    }

    pub(crate) fn expire_for_test(&self, request_id: &str) {
        if let Ok(mut state) = self.state.lock() {
            if let Some(request) = state.requests.get_mut(request_id) {
                request.expires_at = Instant::now();
            }
        }
    }
}
