use std::collections::BTreeMap;

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativePeerTrust {
    pub node_id: String,
    pub root_fingerprint: String,
    pub endpoint_hints: Vec<String>,
    #[serde(skip_serializing)]
    pub bearer_credential: String,
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum NativeTrustError {
    #[error("the native peer trust record is invalid")]
    InvalidRecord,
    #[error("the protected native peer trust store is unavailable")]
    StoreUnavailable,
    #[error("the protected native peer trust record is corrupt")]
    CorruptRecord,
    #[error("the node identity does not match the verified pin")]
    PinMismatch,
    #[error("replacing a trusted node requires explicit approval")]
    ReplacementApprovalRequired,
}

pub trait NativePeerTrustStore {
    fn load(&self, node_id: &str) -> Result<Option<Vec<u8>>, NativeTrustError>;
    fn store(&self, node_id: &str, secret: &[u8]) -> Result<(), NativeTrustError>;
    fn delete(&self, node_id: &str) -> Result<(), NativeTrustError>;
}

pub struct NativePeerTrustService<S> {
    store: S,
}

impl<S: NativePeerTrustStore> NativePeerTrustService<S> {
    pub const fn new(store: S) -> Self {
        Self { store }
    }

    pub fn load(&self, node_id: &str) -> Result<Option<NativePeerTrust>, NativeTrustError> {
        self.store
            .load(node_id)?
            .map(|bytes| decode_record(&bytes))
            .transpose()
    }

    pub fn save_verified(
        &self,
        record: &NativePeerTrust,
        allow_replacement: bool,
    ) -> Result<(), NativeTrustError> {
        validate_record(record)?;
        if let Some(existing) = self.load(&record.node_id)? {
            let changed = existing.root_fingerprint != record.root_fingerprint
                || existing.bearer_credential != record.bearer_credential;
            if changed && !allow_replacement {
                return Err(NativeTrustError::ReplacementApprovalRequired);
            }
        }
        self.store.store(&record.node_id, &encode_record(record))
    }

    pub fn verify_pin(
        &self,
        node_id: &str,
        presented_fingerprint: &str,
    ) -> Result<NativePeerTrust, NativeTrustError> {
        let record = self.load(node_id)?.ok_or(NativeTrustError::PinMismatch)?;
        if record.root_fingerprint != presented_fingerprint {
            return Err(NativeTrustError::PinMismatch);
        }
        Ok(record)
    }

    pub fn delete(&self, node_id: &str) -> Result<(), NativeTrustError> {
        self.store.delete(node_id)
    }
}

fn validate_record(record: &NativePeerTrust) -> Result<(), NativeTrustError> {
    let fingerprint_valid = record.root_fingerprint.len() == 95
        && record
            .root_fingerprint
            .split(':')
            .all(|pair| pair.len() == 2 && pair.bytes().all(|byte| byte.is_ascii_hexdigit()));
    let endpoints_valid = !record.endpoint_hints.is_empty()
        && record.endpoint_hints.len() <= 16
        && record.endpoint_hints.iter().all(|endpoint| {
            endpoint.starts_with("https://")
                && !endpoint[8..].is_empty()
                && !endpoint[8..].contains(['/', '@', '#', '?'])
        });
    if !record.node_id.starts_with("ls_node_")
        || record.node_id.len() > 96
        || !fingerprint_valid
        || !endpoints_valid
        || !record.bearer_credential.starts_with("ls_peer_")
        || record.bearer_credential.len() > 512
    {
        return Err(NativeTrustError::InvalidRecord);
    }
    Ok(())
}

fn encode_record(record: &NativePeerTrust) -> Vec<u8> {
    let mut fields = BTreeMap::new();
    fields.insert("node", record.node_id.as_str());
    fields.insert("fingerprint", record.root_fingerprint.as_str());
    fields.insert("credential", record.bearer_credential.as_str());
    let mut encoded = b"LSTRUST1\n".to_vec();
    for (name, value) in fields {
        encoded.extend_from_slice(format!("{name}:{}:{value}\n", value.len()).as_bytes());
    }
    for endpoint in &record.endpoint_hints {
        encoded.extend_from_slice(format!("endpoint:{}:{endpoint}\n", endpoint.len()).as_bytes());
    }
    encoded
}

fn decode_record(bytes: &[u8]) -> Result<NativePeerTrust, NativeTrustError> {
    let text = std::str::from_utf8(bytes).map_err(|_| NativeTrustError::CorruptRecord)?;
    let mut lines = text.lines();
    if lines.next() != Some("LSTRUST1") {
        return Err(NativeTrustError::CorruptRecord);
    }
    let mut node_id = None;
    let mut fingerprint = None;
    let mut credential = None;
    let mut endpoints = Vec::new();
    for line in lines {
        let (name, rest) = line
            .split_once(':')
            .ok_or(NativeTrustError::CorruptRecord)?;
        let (length, value) = rest
            .split_once(':')
            .ok_or(NativeTrustError::CorruptRecord)?;
        if length.parse::<usize>().ok() != Some(value.len()) {
            return Err(NativeTrustError::CorruptRecord);
        }
        match name {
            "node" if node_id.is_none() => node_id = Some(value.to_owned()),
            "fingerprint" if fingerprint.is_none() => fingerprint = Some(value.to_owned()),
            "credential" if credential.is_none() => credential = Some(value.to_owned()),
            "endpoint" => endpoints.push(value.to_owned()),
            _ => return Err(NativeTrustError::CorruptRecord),
        }
    }
    let record = NativePeerTrust {
        node_id: node_id.ok_or(NativeTrustError::CorruptRecord)?,
        root_fingerprint: fingerprint.ok_or(NativeTrustError::CorruptRecord)?,
        endpoint_hints: endpoints,
        bearer_credential: credential.ok_or(NativeTrustError::CorruptRecord)?,
    };
    validate_record(&record).map_err(|_| NativeTrustError::CorruptRecord)?;
    Ok(record)
}

pub struct KeyringNativePeerTrustStore;

impl NativePeerTrustStore for KeyringNativePeerTrustStore {
    fn load(&self, node_id: &str) -> Result<Option<Vec<u8>>, NativeTrustError> {
        let entry = keyring::Entry::new("org.localstream.native-peer", node_id)
            .map_err(|_| NativeTrustError::StoreUnavailable)?;
        match entry.get_secret() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err(NativeTrustError::StoreUnavailable),
        }
    }

    fn store(&self, node_id: &str, secret: &[u8]) -> Result<(), NativeTrustError> {
        keyring::Entry::new("org.localstream.native-peer", node_id)
            .and_then(|entry| entry.set_secret(secret))
            .map_err(|_| NativeTrustError::StoreUnavailable)
    }

    fn delete(&self, node_id: &str) -> Result<(), NativeTrustError> {
        let entry = keyring::Entry::new("org.localstream.native-peer", node_id)
            .map_err(|_| NativeTrustError::StoreUnavailable)?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err(NativeTrustError::StoreUnavailable),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    struct MemoryStore(Arc<Mutex<Option<Vec<u8>>>>);
    impl NativePeerTrustStore for MemoryStore {
        fn load(&self, _: &str) -> Result<Option<Vec<u8>>, NativeTrustError> {
            Ok(self.0.lock().unwrap().clone())
        }
        fn store(&self, _: &str, value: &[u8]) -> Result<(), NativeTrustError> {
            *self.0.lock().unwrap() = Some(value.to_vec());
            Ok(())
        }
        fn delete(&self, _: &str) -> Result<(), NativeTrustError> {
            *self.0.lock().unwrap() = None;
            Ok(())
        }
    }

    fn record() -> NativePeerTrust {
        NativePeerTrust {
            node_id: "ls_node_example".into(),
            root_fingerprint: ["AA"; 32].join(":"),
            endpoint_hints: vec!["https://192.168.1.10:8443".into()],
            bearer_credential: "ls_peer_secret".into(),
        }
    }

    #[test]
    fn persists_verifies_and_deletes_protected_trust() {
        let store = MemoryStore::default();
        let service = NativePeerTrustService::new(store.clone());
        service.save_verified(&record(), false).unwrap();
        assert_eq!(
            service
                .verify_pin("ls_node_example", &["AA"; 32].join(":"))
                .unwrap(),
            record()
        );
        service.delete("ls_node_example").unwrap();
        assert!(service.load("ls_node_example").unwrap().is_none());
    }

    #[test]
    fn rejects_pin_change_corruption_and_silent_replacement() {
        let store = MemoryStore::default();
        let service = NativePeerTrustService::new(store.clone());
        service.save_verified(&record(), false).unwrap();
        assert_eq!(
            service
                .verify_pin("ls_node_example", &["BB"; 32].join(":"))
                .unwrap_err(),
            NativeTrustError::PinMismatch
        );
        let mut changed = record();
        changed.root_fingerprint = ["BB"; 32].join(":");
        assert_eq!(
            service.save_verified(&changed, false).unwrap_err(),
            NativeTrustError::ReplacementApprovalRequired
        );
        service.save_verified(&changed, true).unwrap();
        *store.0.lock().unwrap() = Some(b"corrupt".to_vec());
        assert_eq!(
            service.load("ls_node_example").unwrap_err(),
            NativeTrustError::CorruptRecord
        );
    }
}
