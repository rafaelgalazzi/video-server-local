use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rcgen::{
    date_time_ymd, BasicConstraints, Certificate, CertificateParams, DistinguishedName, DnType,
    ExtendedKeyUsagePurpose, IsCa, Issuer, KeyPair, KeyUsagePurpose, PublicKeyData, SanType,
    SerialNumber,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use thiserror::Error;

const NODE_ID_PREFIX: &str = "ls_node_";
const NODE_ID_BYTES: usize = 16;
const MAX_LEAF_NAMES: usize = 16;
const LEAF_VALIDITY_DAYS: i64 = 30;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeIdentitySummary {
    pub node_id: String,
    pub fingerprint: String,
}

pub struct NodeIdentity {
    summary: NodeIdentitySummary,
    root_params: CertificateParams,
    root_certificate: Certificate,
    signing_key: KeyPair,
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

    pub fn issue_server_leaf(&self, names: &[String]) -> Result<ServerLeaf, LeafIssuanceError> {
        self.issue_server_leaf_at(names, time::OffsetDateTime::now_utc())
    }

    pub(crate) fn issue_server_leaf_at(
        &self,
        names: &[String],
        now: time::OffsetDateTime,
    ) -> Result<ServerLeaf, LeafIssuanceError> {
        let subject_alt_names = validate_leaf_names(names)?;
        let signing_key = KeyPair::generate().map_err(|_| LeafIssuanceError::GenerationFailed)?;
        let digest: [u8; 32] = Sha256::digest(signing_key.subject_public_key_info()).into();
        let mut distinguished_name = DistinguishedName::new();
        distinguished_name.push(DnType::OrganizationName, "LocalStream");
        distinguished_name.push(DnType::CommonName, self.summary.node_id.clone());
        let mut params = CertificateParams::default();
        params.not_before = now - time::Duration::minutes(5);
        params.not_after = now + time::Duration::days(LEAF_VALIDITY_DAYS);
        params.serial_number = Some(SerialNumber::from_slice(&digest[..20]));
        params.subject_alt_names = subject_alt_names;
        params.distinguished_name = distinguished_name;
        params.is_ca = IsCa::NoCa;
        params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
        params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
        params.use_authority_key_identifier_extension = true;
        let issuer = Issuer::from_params(&self.root_params, &self.signing_key);
        let certificate = params
            .signed_by(&signing_key, &issuer)
            .map_err(|_| LeafIssuanceError::GenerationFailed)?;

        Ok(ServerLeaf {
            certificate_chain_der: vec![
                certificate.der().to_vec(),
                self.root_certificate.der().to_vec(),
            ],
            signing_key,
        })
    }
}

pub struct ServerLeaf {
    certificate_chain_der: Vec<Vec<u8>>,
    signing_key: KeyPair,
}

impl ServerLeaf {
    #[must_use]
    pub fn certificate_chain_der(&self) -> &[Vec<u8>] {
        &self.certificate_chain_der
    }

    pub fn into_server_config(self) -> Result<rustls::ServerConfig, TlsConfigError> {
        let certificate_chain = self
            .certificate_chain_der
            .into_iter()
            .map(rustls::pki_types::CertificateDer::from)
            .collect();
        let private_key =
            rustls::pki_types::PrivatePkcs8KeyDer::from(self.signing_key.serialize_der());
        let provider = std::sync::Arc::new(rustls::crypto::ring::default_provider());
        let mut config = rustls::ServerConfig::builder_with_provider(provider)
            .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])
            .map_err(|_| TlsConfigError::Unavailable)?
            .with_no_client_auth()
            .with_single_cert(certificate_chain, private_key.into())
            .map_err(|_| TlsConfigError::InvalidMaterial)?;
        config.alpn_protocols = vec![b"http/1.1".to_vec()];
        Ok(config)
    }
}

impl std::fmt::Debug for ServerLeaf {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ServerLeaf")
            .field("certificate_count", &self.certificate_chain_der.len())
            .field("algorithm", &self.signing_key.algorithm())
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum LeafIssuanceError {
    #[error("at least one valid server name is required")]
    InvalidNames,
    #[error("a server leaf certificate could not be generated")]
    GenerationFailed,
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum TlsConfigError {
    #[error("the TLS configuration is unavailable")]
    Unavailable,
    #[error("the TLS certificate material is invalid")]
    InvalidMaterial,
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
    fn delete(&self) -> Result<(), SecretStoreError>;
}

impl<T> NodeSecretStore for &T
where
    T: NodeSecretStore + ?Sized,
{
    fn load(&self) -> Result<Option<Vec<u8>>, SecretStoreError> {
        T::load(*self)
    }

    fn store(&self, secret: &[u8]) -> Result<(), SecretStoreError> {
        T::store(*self, secret)
    }

    fn delete(&self) -> Result<(), SecretStoreError> {
        T::delete(*self)
    }
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum NodeIdentityError {
    #[error("the protected node identity store is unavailable")]
    StoreUnavailable,
    #[error("the stored node identity is invalid")]
    InvalidStoredIdentity,
    #[error("the stored node identity is missing")]
    MissingStoredIdentity,
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

    pub fn into_store(self) -> S {
        self.store
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

    pub fn load_existing(&self) -> Result<NodeIdentity, NodeIdentityError> {
        let secret = self
            .store
            .load()
            .map_err(map_store_error)?
            .ok_or(NodeIdentityError::MissingStoredIdentity)?;
        let signing_key =
            KeyPair::try_from(secret).map_err(|_| NodeIdentityError::InvalidStoredIdentity)?;
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
        root_params: params,
        root_certificate,
        signing_key,
    })
}

fn validate_leaf_names(names: &[String]) -> Result<Vec<SanType>, LeafIssuanceError> {
    if names.is_empty() || names.len() > MAX_LEAF_NAMES {
        return Err(LeafIssuanceError::InvalidNames);
    }
    let mut normalized = std::collections::HashSet::with_capacity(names.len());
    let mut sans = Vec::with_capacity(names.len());
    for name in names {
        let name = name.trim();
        if name.is_empty() || name.contains('*') {
            return Err(LeafIssuanceError::InvalidNames);
        }
        if let Ok(address) = name.parse::<std::net::IpAddr>() {
            let canonical = address.to_string();
            if !normalized.insert(canonical) {
                return Err(LeafIssuanceError::InvalidNames);
            }
            sans.push(SanType::IpAddress(address));
            continue;
        }
        let canonical = name.to_ascii_lowercase();
        if !is_valid_dns_name(&canonical) || !normalized.insert(canonical.clone()) {
            return Err(LeafIssuanceError::InvalidNames);
        }
        let dns_name = canonical
            .try_into()
            .map_err(|_| LeafIssuanceError::InvalidNames)?;
        sans.push(SanType::DnsName(dns_name));
    }
    Ok(sans)
}

fn is_valid_dns_name(name: &str) -> bool {
    name.len() <= 253
        && name.is_ascii()
        && name.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                && label
                    .as_bytes()
                    .first()
                    .is_some_and(u8::is_ascii_alphanumeric)
                && label
                    .as_bytes()
                    .last()
                    .is_some_and(u8::is_ascii_alphanumeric)
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

    fn delete(&self) -> Result<(), SecretStoreError> {
        match self.entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err(SecretStoreError::Unavailable),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::{
        LeafIssuanceError, NodeIdentityError, NodeIdentityService, NodeSecretStore,
        SecretStoreError, LEAF_VALIDITY_DAYS, MAX_LEAF_NAMES, NODE_ID_PREFIX,
    };
    use base64::Engine;
    use x509_parser::{extensions::GeneralName, parse_x509_certificate};

    #[derive(Clone, Default)]
    struct MemoryStore {
        value: Arc<Mutex<Option<Vec<u8>>>>,
        fail_load: bool,
        fail_store: bool,
        fail_delete: bool,
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

        fn delete(&self) -> Result<(), SecretStoreError> {
            if self.fail_delete {
                return Err(SecretStoreError::Unavailable);
            }
            *self.value.lock().expect("store should lock") = None;
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

    #[test]
    fn existing_only_load_never_creates_a_missing_identity() {
        let store = MemoryStore::default();
        let error = NodeIdentityService::new(&store)
            .load_existing()
            .expect_err("missing identity should fail");

        assert_eq!(error, NodeIdentityError::MissingStoredIdentity);
        assert!(store.value.lock().expect("store should lock").is_none());

        let created = NodeIdentityService::new(&store)
            .load_or_create()
            .expect("identity should create");
        let existing = NodeIdentityService::new(&store)
            .load_existing()
            .expect("identity should load");
        assert_eq!(created.summary(), existing.summary());
    }

    #[test]
    fn issues_a_short_lived_server_only_leaf_with_dns_and_ip_sans() {
        let identity = NodeIdentityService::new(MemoryStore::default())
            .load_or_create()
            .expect("identity should generate");
        let now = time::OffsetDateTime::from_unix_timestamp(1_800_000_000)
            .expect("test timestamp should be valid");
        let leaf = identity
            .issue_server_leaf_at(
                &[
                    "LocalHost".to_owned(),
                    "127.0.0.1".to_owned(),
                    "::1".to_owned(),
                ],
                now,
            )
            .expect("leaf should issue");

        assert_eq!(leaf.certificate_chain_der().len(), 2);
        assert_eq!(
            leaf.certificate_chain_der()[1],
            identity.root_certificate_der()
        );
        let (_, parsed_leaf) =
            parse_x509_certificate(&leaf.certificate_chain_der()[0]).expect("leaf should parse");
        let (_, parsed_root) =
            parse_x509_certificate(&leaf.certificate_chain_der()[1]).expect("root should parse");
        parsed_leaf
            .verify_signature(Some(parsed_root.public_key()))
            .expect("root should verify leaf signature");

        let sans = parsed_leaf
            .subject_alternative_name()
            .expect("SAN parsing should succeed")
            .expect("SAN should exist");
        assert!(sans
            .value
            .general_names
            .contains(&GeneralName::DNSName("localhost")));
        assert!(sans
            .value
            .general_names
            .contains(&GeneralName::IPAddress(&[127, 0, 0, 1])));
        assert!(sans.value.general_names.contains(&GeneralName::IPAddress(&[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1
        ])));

        let key_usage = parsed_leaf
            .key_usage()
            .expect("key usage should parse")
            .expect("key usage should exist");
        assert!(key_usage.value.digital_signature());
        assert!(!key_usage.value.key_cert_sign());
        let extended = parsed_leaf
            .extended_key_usage()
            .expect("extended usage should parse")
            .expect("extended usage should exist");
        assert!(extended.value.server_auth);
        assert!(!extended.value.client_auth);
        assert!(!extended.value.any);

        let validity = parsed_leaf.validity();
        assert_eq!(
            validity.not_before.timestamp(),
            (now - time::Duration::minutes(5)).unix_timestamp()
        );
        assert_eq!(
            validity.not_after.timestamp(),
            (now + time::Duration::days(LEAF_VALIDITY_DAYS)).unix_timestamp()
        );
        assert!(parsed_root.validity().not_after.timestamp() > validity.not_after.timestamp());
    }

    #[test]
    fn generates_a_unique_key_for_each_leaf_and_redacts_debug_output() {
        let identity = NodeIdentityService::new(MemoryStore::default())
            .load_or_create()
            .expect("identity should generate");
        let names = ["localhost".to_owned()];
        let first = identity
            .issue_server_leaf(&names)
            .expect("first leaf should issue");
        let second = identity
            .issue_server_leaf(&names)
            .expect("second leaf should issue");
        let (_, first_cert) = parse_x509_certificate(&first.certificate_chain_der()[0])
            .expect("first leaf should parse");
        let (_, second_cert) = parse_x509_certificate(&second.certificate_chain_der()[0])
            .expect("second leaf should parse");

        assert_ne!(first_cert.public_key().raw, second_cert.public_key().raw);
        let debug = format!("{first:?}");
        assert!(debug.starts_with("ServerLeaf { certificate_count: 2, algorithm: "));
        assert!(!debug.contains(
            &base64::engine::general_purpose::STANDARD.encode(first.signing_key.serialize_der())
        ));
    }

    #[test]
    fn rejects_empty_wildcard_malformed_duplicate_and_excessive_leaf_names() {
        let identity = NodeIdentityService::new(MemoryStore::default())
            .load_or_create()
            .expect("identity should generate");
        let invalid = [
            Vec::new(),
            vec!["".to_owned()],
            vec!["*.localstream.test".to_owned()],
            vec!["-bad.local".to_owned()],
            vec!["bad_.local".to_owned()],
            vec!["LOCALHOST".to_owned(), "localhost".to_owned()],
            vec!["127.0.0.1".to_owned(), "127.0.0.1".to_owned()],
            (0..=MAX_LEAF_NAMES)
                .map(|index| format!("node-{index}.local"))
                .collect(),
        ];

        for names in invalid {
            assert_eq!(
                identity
                    .issue_server_leaf(&names)
                    .expect_err("names should fail"),
                LeafIssuanceError::InvalidNames
            );
        }
    }

    fn client_config(
        root_der: &[u8],
        versions: &[&'static rustls::SupportedProtocolVersion],
    ) -> rustls::ClientConfig {
        let mut roots = rustls::RootCertStore::empty();
        roots
            .add(rustls::pki_types::CertificateDer::from(root_der.to_vec()))
            .expect("test root should add");
        let provider = std::sync::Arc::new(rustls::crypto::ring::default_provider());
        let mut config = rustls::ClientConfig::builder_with_provider(provider)
            .with_protocol_versions(versions)
            .expect("test protocol should configure")
            .with_root_certificates(roots)
            .with_no_client_auth();
        config.alpn_protocols = vec![b"http/1.1".to_vec()];
        config
    }

    fn complete_handshake(
        client: &mut rustls::ClientConnection,
        server: &mut rustls::ServerConnection,
    ) -> Result<(), rustls::Error> {
        for _ in 0..20 {
            let mut client_bytes = Vec::new();
            client
                .write_tls(&mut client_bytes)
                .expect("memory write should succeed");
            if !client_bytes.is_empty() {
                server
                    .read_tls(&mut std::io::Cursor::new(client_bytes))
                    .expect("memory read should succeed");
                server.process_new_packets()?;
            }

            let mut server_bytes = Vec::new();
            server
                .write_tls(&mut server_bytes)
                .expect("memory write should succeed");
            if !server_bytes.is_empty() {
                client
                    .read_tls(&mut std::io::Cursor::new(server_bytes))
                    .expect("memory read should succeed");
                client.process_new_packets()?;
            }
            if !client.is_handshaking() && !server.is_handshaking() {
                return Ok(());
            }
        }
        Err(rustls::Error::General(
            "in-memory handshake did not complete".to_owned(),
        ))
    }

    fn handshake_with(
        identity: &super::NodeIdentity,
        trusted_root: &[u8],
        server_name: &str,
        versions: &[&'static rustls::SupportedProtocolVersion],
    ) -> Result<(rustls::ClientConnection, rustls::ServerConnection), rustls::Error> {
        let leaf = identity
            .issue_server_leaf(&["localhost".to_owned(), "127.0.0.1".to_owned()])
            .expect("leaf should issue");
        let server_config = leaf.into_server_config().expect("server should configure");
        let client_config = client_config(trusted_root, versions);
        let name = rustls::pki_types::ServerName::try_from(server_name.to_owned())
            .expect("test name should parse");
        let mut client = rustls::ClientConnection::new(std::sync::Arc::new(client_config), name)
            .expect("client should construct");
        let mut server = rustls::ServerConnection::new(std::sync::Arc::new(server_config))
            .expect("server should construct");
        complete_handshake(&mut client, &mut server)?;
        Ok((client, server))
    }

    #[test]
    fn rustls_configuration_handshakes_for_tls_13_and_tls_12_without_client_auth() {
        let identity = NodeIdentityService::new(MemoryStore::default())
            .load_or_create()
            .expect("identity should generate");
        for version in [&rustls::version::TLS13, &rustls::version::TLS12] {
            let (client, server) = handshake_with(
                &identity,
                identity.root_certificate_der(),
                "localhost",
                &[version],
            )
            .expect("trusted handshake should complete");
            assert_eq!(client.alpn_protocol(), Some(b"http/1.1".as_slice()));
            assert_eq!(server.alpn_protocol(), Some(b"http/1.1".as_slice()));
            assert_eq!(client.protocol_version(), Some(version.version));
        }
    }

    #[test]
    fn rustls_configuration_rejects_wrong_root_and_wrong_server_name() {
        let identity = NodeIdentityService::new(MemoryStore::default())
            .load_or_create()
            .expect("identity should generate");
        let other = NodeIdentityService::new(MemoryStore::default())
            .load_or_create()
            .expect("other identity should generate");

        assert!(handshake_with(
            &identity,
            other.root_certificate_der(),
            "localhost",
            &[&rustls::version::TLS13],
        )
        .is_err());
        assert!(handshake_with(
            &identity,
            identity.root_certificate_der(),
            "other.local",
            &[&rustls::version::TLS13],
        )
        .is_err());
    }

    #[test]
    fn rustls_configuration_fails_closed_for_invalid_certificate_material() {
        let identity = NodeIdentityService::new(MemoryStore::default())
            .load_or_create()
            .expect("identity should generate");
        let mut leaf = identity
            .issue_server_leaf(&["localhost".to_owned()])
            .expect("leaf should issue");
        leaf.certificate_chain_der[0] = b"not a certificate".to_vec();

        assert_eq!(
            leaf.into_server_config()
                .expect_err("invalid material should fail"),
            super::TlsConfigError::InvalidMaterial
        );
    }
}
