use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rcgen::{
    date_time_ymd, BasicConstraints, Certificate, CertificateParams, DistinguishedName, DnType,
    IsCa, KeyPair, KeyUsagePurpose, PublicKeyData, SerialNumber,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use thiserror::Error;

const NODE_ID_PREFIX: &str = "ls_node_";
const NODE_ID_BYTES: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeIdentitySummary {
    pub node_id: String,
    pub fingerprint: String,
}

pub struct NodeIdentity {
    summary: NodeIdentitySummary,
    root_certificate: Certificate,
}

impl NodeIdentity {
    #[must_use]
    pub fn summary(&self) -> &NodeIdentitySummary {
        &self.summary
    }

    #[must_use]
    pub fn root_certificate_der(&self) -> &[u8] {
        self.root_certificate.der()
    }
}

impl std::fmt::Debug for NodeIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NodeIdentity")
            .field("summary", &self.summary)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum SecretStoreError {
    #[error("the protected secret store is unavailable")]
    Unavailable,
}

pub trait NodeSecretStore {
    fn load(&self) -> Result<Option<Vec<u8>>, SecretStoreError>;
    fn store(&self, secret: &[u8]) -> Result<(), SecretStoreError>;
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum NodeIdentityError {
    #[error("the protected node identity store is unavailable")]
    StoreUnavailable,
    #[error("the stored node identity is invalid")]
    InvalidStoredIdentity,
    #[error("a node identity could not be generated")]
    GenerationFailed,
}

pub struct NodeIdentityService<S> {
    store: S,
}

impl<S> NodeIdentityService<S>
where
    S: NodeSecretStore,
{
    pub const fn new(store: S) -> Self {
        Self { store }
    }

    pub fn load_or_create(&self) -> Result<NodeIdentity, NodeIdentityError> {
        let signing_key = match self.store.load().map_err(map_store_error)? {
            Some(secret) => {
                KeyPair::try_from(secret).map_err(|_| NodeIdentityError::InvalidStoredIdentity)?
            }
            None => {
                let signing_key =
                    KeyPair::generate().map_err(|_| NodeIdentityError::GenerationFailed)?;
                self.store
                    .store(&signing_key.serialize_der())
                    .map_err(map_store_error)?;
                signing_key
            }
        };

        identity_from_key(signing_key)
    }
}

fn map_store_error(_: SecretStoreError) -> NodeIdentityError {
    NodeIdentityError::StoreUnavailable
}

fn identity_from_key(signing_key: KeyPair) -> Result<NodeIdentity, NodeIdentityError> {
    let digest: [u8; 32] = Sha256::digest(signing_key.subject_public_key_info()).into();
    let node_id = format!(
        "{NODE_ID_PREFIX}{}",
        URL_SAFE_NO_PAD.encode(&digest[..NODE_ID_BYTES])
    );
    let fingerprint = digest
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(":");

    let mut distinguished_name = DistinguishedName::new();
    distinguished_name.push(DnType::OrganizationName, "LocalStream");
    distinguished_name.push(DnType::CommonName, node_id.clone());
    let mut params = CertificateParams::default();
    params.not_before = date_time_ymd(2025, 1, 1);
    params.not_after = date_time_ymd(2125, 1, 1);
    params.serial_number = Some(SerialNumber::from_slice(&digest[..20]));
    params.distinguished_name = distinguished_name;
    params.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));
    params.key_usages = vec![
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::CrlSign,
    ];
    let root_certificate = params
        .self_signed(&signing_key)
        .map_err(|_| NodeIdentityError::GenerationFailed)?;

    Ok(NodeIdentity {
        summary: NodeIdentitySummary {
            node_id,
            fingerprint,
        },
        root_certificate,
    })
}

pub struct KeyringNodeSecretStore {
    entry: keyring::Entry,
}

impl KeyringNodeSecretStore {
    pub fn new(instance: &str) -> Result<Self, SecretStoreError> {
        if instance.trim().is_empty() {
            return Err(SecretStoreError::Unavailable);
        }
        let entry = keyring::Entry::new("org.localstream.node-identity", instance)
            .map_err(|_| SecretStoreError::Unavailable)?;
        Ok(Self { entry })
    }
}

impl NodeSecretStore for KeyringNodeSecretStore {
    fn load(&self) -> Result<Option<Vec<u8>>, SecretStoreError> {
        match self.entry.get_secret() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err(SecretStoreError::Unavailable),
        }
    }

    fn store(&self, secret: &[u8]) -> Result<(), SecretStoreError> {
        self.entry
            .set_secret(secret)
            .map_err(|_| SecretStoreError::Unavailable)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::{
        NodeIdentityError, NodeIdentityService, NodeSecretStore, SecretStoreError, NODE_ID_PREFIX,
    };

    #[derive(Clone, Default)]
    struct MemoryStore {
        value: Arc<Mutex<Option<Vec<u8>>>>,
        fail_load: bool,
        fail_store: bool,
    }

    impl NodeSecretStore for MemoryStore {
        fn load(&self) -> Result<Option<Vec<u8>>, SecretStoreError> {
            if self.fail_load {
                return Err(SecretStoreError::Unavailable);
            }
            Ok(self.value.lock().expect("store should lock").clone())
        }

        fn store(&self, secret: &[u8]) -> Result<(), SecretStoreError> {
            if self.fail_store {
                return Err(SecretStoreError::Unavailable);
            }
            *self.value.lock().expect("store should lock") = Some(secret.to_vec());
            Ok(())
        }
    }

    #[test]
    fn generates_and_restores_a_stable_public_identity() {
        let store = MemoryStore::default();
        let first = NodeIdentityService::new(store.clone())
            .load_or_create()
            .expect("identity should generate");
        let restored = NodeIdentityService::new(store)
            .load_or_create()
            .expect("identity should restore");

        assert_eq!(first.summary(), restored.summary());
        assert!(first.summary().node_id.starts_with(NODE_ID_PREFIX));
        assert_eq!(first.summary().fingerprint.len(), 95);
        assert!(!first.root_certificate_der().is_empty());
        assert!(!restored.root_certificate_der().is_empty());
    }

    #[test]
    fn fails_closed_for_corrupt_stored_identity_without_replacing_it() {
        let original = b"not a PKCS8 key".to_vec();
        let store = MemoryStore {
            value: Arc::new(Mutex::new(Some(original.clone()))),
            ..MemoryStore::default()
        };

        let error = NodeIdentityService::new(store.clone())
            .load_or_create()
            .expect_err("corrupt identity should fail");

        assert_eq!(error, NodeIdentityError::InvalidStoredIdentity);
        assert_eq!(
            store.value.lock().expect("store should lock").as_ref(),
            Some(&original)
        );
    }

    #[test]
    fn fails_without_returning_an_ephemeral_identity_when_storage_fails() {
        let load_error = NodeIdentityService::new(MemoryStore {
            fail_load: true,
            ..MemoryStore::default()
        })
        .load_or_create()
        .expect_err("load failure should fail closed");
        assert_eq!(load_error, NodeIdentityError::StoreUnavailable);

        let store_error = NodeIdentityService::new(MemoryStore {
            fail_store: true,
            ..MemoryStore::default()
        })
        .load_or_create()
        .expect_err("store failure should fail closed");
        assert_eq!(store_error, NodeIdentityError::StoreUnavailable);
    }
}
