use std::{collections::BTreeSet, net::IpAddr};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::node_identity::NodeIdentity;

const DEFAULT_HTTPS_PORT: u16 = 8443;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanServerConfig {
    pub enabled: bool,
    pub address: Option<IpAddr>,
    pub port: u16,
    pub dns_name: Option<String>,
}

impl Default for LanServerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            address: None,
            port: DEFAULT_HTTPS_PORT,
            dns_name: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanServerStatus {
    pub configured: bool,
    pub active: bool,
    pub endpoint: Option<String>,
    pub failure: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct ActiveTlsLeaf {
    pub generation: u64,
    pub names: Vec<String>,
    pub expires_at: time::OffsetDateTime,
    pub server_config: std::sync::Arc<rustls::ServerConfig>,
}

#[derive(Debug, Default)]
pub struct TlsLeafLifecycle {
    active: std::sync::Mutex<Option<ActiveTlsLeaf>>,
}

impl TlsLeafLifecycle {
    pub fn current(&self) -> Option<ActiveTlsLeaf> {
        self.active.lock().ok()?.clone()
    }

    pub fn ensure_current(
        &self,
        identity: &NodeIdentity,
        config: &LanServerConfig,
        now: time::OffsetDateTime,
    ) -> Result<ActiveTlsLeaf, LeafLifecycleError> {
        validate(config).map_err(|_| LeafLifecycleError::InvalidConfiguration)?;
        let mut names = Vec::new();
        if let Some(name) = &config.dns_name {
            names.push(name.to_ascii_lowercase());
        }
        if let Some(address) = config.address {
            names.push(address.to_string());
        }
        let mut active = self
            .active
            .lock()
            .map_err(|_| LeafLifecycleError::Unavailable)?;
        if let Some(current) = active.as_ref() {
            if current.names == names && now < current.expires_at - time::Duration::days(7) {
                return Ok(current.clone());
            }
        }
        let generation = active.as_ref().map_or(1, |leaf| leaf.generation + 1);
        let leaf = identity
            .issue_server_leaf_at(&names, now)
            .map_err(|_| LeafLifecycleError::IssuanceFailed)?;
        let next = ActiveTlsLeaf {
            generation,
            names,
            expires_at: now + time::Duration::days(30),
            server_config: std::sync::Arc::new(
                leaf.into_server_config()
                    .map_err(|_| LeafLifecycleError::IssuanceFailed)?,
            ),
        };
        *active = Some(next.clone());
        Ok(next)
    }
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum LeafLifecycleError {
    #[error("the TLS leaf configuration is invalid")]
    InvalidConfiguration,
    #[error("the TLS leaf could not be issued")]
    IssuanceFailed,
    #[error("the TLS leaf lifecycle is unavailable")]
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanSecurityEvidence {
    pub browser_trust_onboarding: bool,
    pub native_protected_storage: bool,
    pub negative_security_suite: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanSecurityAudit {
    pub passed: bool,
    pub failed_gates: Vec<&'static str>,
}

#[derive(Debug)]
pub struct LanActivationPermit(());

pub fn audit_activation(
    evidence: LanSecurityEvidence,
) -> (LanSecurityAudit, Option<LanActivationPermit>) {
    let mut failed = Vec::new();
    if !evidence.browser_trust_onboarding {
        failed.push("browser_trust_onboarding");
    }
    if !evidence.native_protected_storage {
        failed.push("native_protected_storage");
    }
    if !evidence.negative_security_suite {
        failed.push("negative_security_suite");
    }
    let passed = failed.is_empty();
    (
        LanSecurityAudit {
            passed,
            failed_gates: failed,
        },
        passed.then_some(LanActivationPermit(())),
    )
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum LanConfigError {
    #[error("the LAN server configuration is invalid")]
    Invalid,
    #[error("the LAN server configuration store is unavailable")]
    Unavailable,
    #[error("the stored LAN server configuration is corrupt")]
    Corrupt,
}

pub trait LanConfigStore {
    fn load(&self) -> Result<Option<Vec<u8>>, LanConfigError>;
    fn store(&self, value: &[u8]) -> Result<(), LanConfigError>;
}

impl<T: LanConfigStore + ?Sized> LanConfigStore for &T {
    fn load(&self) -> Result<Option<Vec<u8>>, LanConfigError> {
        T::load(*self)
    }
    fn store(&self, value: &[u8]) -> Result<(), LanConfigError> {
        T::store(*self, value)
    }
}

#[derive(Debug, Clone)]
pub struct FileLanConfigStore {
    path: std::path::PathBuf,
}

impl FileLanConfigStore {
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl LanConfigStore for FileLanConfigStore {
    fn load(&self) -> Result<Option<Vec<u8>>, LanConfigError> {
        match std::fs::read(&self.path) {
            Ok(value) => Ok(Some(value)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(_) => Err(LanConfigError::Unavailable),
        }
    }
    fn store(&self, value: &[u8]) -> Result<(), LanConfigError> {
        let parent = self.path.parent().ok_or(LanConfigError::Unavailable)?;
        std::fs::create_dir_all(parent).map_err(|_| LanConfigError::Unavailable)?;
        std::fs::write(&self.path, value).map_err(|_| LanConfigError::Unavailable)
    }
}

pub fn primary_lan_address() -> Option<IpAddr> {
    for target in ["192.0.2.1:9", "[2001:db8::1]:9"] {
        let target: std::net::SocketAddr = target.parse().ok()?;
        let bind = if target.is_ipv4() {
            "0.0.0.0:0"
        } else {
            "[::]:0"
        };
        let Ok(socket) = std::net::UdpSocket::bind(bind) else {
            continue;
        };
        if socket.connect(target).is_ok() {
            let address = socket.local_addr().ok()?.ip();
            if is_safe_lan_address(address) {
                return Some(address);
            }
        }
    }
    None
}

pub struct LanConfigService<S> {
    store: S,
}

impl<S: LanConfigStore> LanConfigService<S> {
    pub const fn new(store: S) -> Self {
        Self { store }
    }

    pub fn load(&self) -> Result<LanServerConfig, LanConfigError> {
        self.store
            .load()?
            .map_or_else(|| Ok(LanServerConfig::default()), |bytes| decode(&bytes))
    }

    pub fn save(&self, config: &LanServerConfig) -> Result<(), LanConfigError> {
        validate(config)?;
        self.store.store(encode(config).as_bytes())
    }
}

pub fn validate(config: &LanServerConfig) -> Result<(), LanConfigError> {
    if config.port < 1024 || config.port == u16::MAX {
        return Err(LanConfigError::Invalid);
    }
    if config.enabled && config.address.is_none() {
        return Err(LanConfigError::Invalid);
    }
    if let Some(address) = config.address {
        if !is_safe_lan_address(address) {
            return Err(LanConfigError::Invalid);
        }
    }
    if let Some(name) = &config.dns_name {
        let name = name.trim();
        if name.is_empty()
            || name.len() > 253
            || name.contains('*')
            || name.parse::<IpAddr>().is_ok()
            || !name.split('.').all(|label| {
                !label.is_empty()
                    && label.len() <= 63
                    && label
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'-')
                    && !label.starts_with('-')
                    && !label.ends_with('-')
            })
        {
            return Err(LanConfigError::Invalid);
        }
    }
    Ok(())
}

pub fn safe_lan_addresses(addresses: impl IntoIterator<Item = IpAddr>) -> Vec<IpAddr> {
    addresses
        .into_iter()
        .filter(|address| is_safe_lan_address(*address))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn is_safe_lan_address(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(value) => {
            !value.is_unspecified()
                && !value.is_loopback()
                && !value.is_multicast()
                && value != std::net::Ipv4Addr::BROADCAST
                && (value.is_private() || value.is_link_local())
        }
        IpAddr::V6(value) => {
            let octets = value.octets();
            let private =
                octets[0] & 0xfe == 0xfc || (octets[0] == 0xfe && octets[1] & 0xc0 == 0x80);
            !value.is_unspecified() && !value.is_loopback() && !value.is_multicast() && private
        }
    }
}

fn encode(config: &LanServerConfig) -> String {
    format!(
        "LSLAN1\nenabled={}\naddress={}\nport={}\ndns={}\n",
        config.enabled,
        config.address.map(|v| v.to_string()).unwrap_or_default(),
        config.port,
        config.dns_name.as_deref().unwrap_or_default()
    )
}

fn decode(bytes: &[u8]) -> Result<LanServerConfig, LanConfigError> {
    let text = std::str::from_utf8(bytes).map_err(|_| LanConfigError::Corrupt)?;
    let mut lines = text.lines();
    if lines.next() != Some("LSLAN1") {
        return Err(LanConfigError::Corrupt);
    }
    let enabled = lines
        .next()
        .and_then(|line| line.strip_prefix("enabled="))
        .and_then(|v| v.parse().ok())
        .ok_or(LanConfigError::Corrupt)?;
    let address = lines
        .next()
        .and_then(|line| line.strip_prefix("address="))
        .ok_or(LanConfigError::Corrupt)?;
    let address = if address.is_empty() {
        None
    } else {
        Some(address.parse().map_err(|_| LanConfigError::Corrupt)?)
    };
    let port = lines
        .next()
        .and_then(|line| line.strip_prefix("port="))
        .and_then(|v| v.parse().ok())
        .ok_or(LanConfigError::Corrupt)?;
    let dns = lines
        .next()
        .and_then(|line| line.strip_prefix("dns="))
        .ok_or(LanConfigError::Corrupt)?;
    if lines.next().is_some() {
        return Err(LanConfigError::Corrupt);
    }
    let config = LanServerConfig {
        enabled,
        address,
        port,
        dns_name: if dns.is_empty() {
            None
        } else {
            Some(dns.to_owned())
        },
    };
    validate(&config).map_err(|_| LanConfigError::Corrupt)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    #[derive(Clone, Default)]
    struct Store(Arc<Mutex<Option<Vec<u8>>>>);
    impl LanConfigStore for Store {
        fn load(&self) -> Result<Option<Vec<u8>>, LanConfigError> {
            Ok(self.0.lock().unwrap().clone())
        }
        fn store(&self, v: &[u8]) -> Result<(), LanConfigError> {
            *self.0.lock().unwrap() = Some(v.to_vec());
            Ok(())
        }
    }

    #[test]
    fn defaults_disabled_and_persists_explicit_endpoint() {
        let store = Store::default();
        let service = LanConfigService::new(store.clone());
        assert_eq!(service.load().unwrap(), LanServerConfig::default());
        let config = LanServerConfig {
            enabled: true,
            address: Some("192.168.1.20".parse().unwrap()),
            port: 8443,
            dns_name: Some("media.home".into()),
        };
        service.save(&config).unwrap();
        assert_eq!(service.load().unwrap(), config);
    }
    #[test]
    fn rejects_loopback_wildcard_public_multicast_and_invalid_ports() {
        for address in [
            "0.0.0.0",
            "127.0.0.1",
            "8.8.8.8",
            "224.0.0.1",
            "::",
            "::1",
            "2001:4860:4860::8888",
        ] {
            let config = LanServerConfig {
                enabled: true,
                address: Some(address.parse().unwrap()),
                port: 8443,
                dns_name: None,
            };
            assert_eq!(validate(&config), Err(LanConfigError::Invalid));
        }
        let mut config = LanServerConfig {
            enabled: true,
            ..LanServerConfig::default()
        };
        assert_eq!(validate(&config), Err(LanConfigError::Invalid));
        config.port = 80;
        assert_eq!(validate(&config), Err(LanConfigError::Invalid));
    }
    #[test]
    fn enumerates_only_deduplicated_safe_addresses() {
        assert_eq!(
            safe_lan_addresses([
                "127.0.0.1".parse().unwrap(),
                "192.168.1.4".parse().unwrap(),
                "192.168.1.4".parse().unwrap(),
                "8.8.8.8".parse().unwrap()
            ]),
            vec!["192.168.1.4".parse::<IpAddr>().unwrap()]
        );
    }
    #[test]
    fn corrupt_persistence_fails_closed() {
        let store = Store(Arc::new(Mutex::new(Some(b"bad".to_vec()))));
        assert_eq!(
            LanConfigService::new(store).load(),
            Err(LanConfigError::Corrupt)
        );
    }

    #[derive(Clone, Default)]
    struct IdentityStore(Arc<Mutex<Option<Vec<u8>>>>);
    impl crate::node_identity::NodeSecretStore for IdentityStore {
        fn load(&self) -> Result<Option<Vec<u8>>, crate::node_identity::SecretStoreError> {
            Ok(self.0.lock().unwrap().clone())
        }
        fn store(&self, v: &[u8]) -> Result<(), crate::node_identity::SecretStoreError> {
            *self.0.lock().unwrap() = Some(v.to_vec());
            Ok(())
        }
        fn delete(&self) -> Result<(), crate::node_identity::SecretStoreError> {
            *self.0.lock().unwrap() = None;
            Ok(())
        }
    }
    #[test]
    fn leaf_lifecycle_reuses_renews_and_rotates_for_address_changes() {
        let identity = crate::node_identity::NodeIdentityService::new(IdentityStore::default())
            .load_or_create()
            .unwrap();
        let lifecycle = TlsLeafLifecycle::default();
        let now = time::OffsetDateTime::from_unix_timestamp(1_800_000_000).unwrap();
        let mut config = LanServerConfig {
            enabled: true,
            address: Some("192.168.1.20".parse().unwrap()),
            port: 8443,
            dns_name: Some("media.home".into()),
        };
        let first = lifecycle.ensure_current(&identity, &config, now).unwrap();
        let same = lifecycle
            .ensure_current(&identity, &config, now + time::Duration::days(1))
            .unwrap();
        assert_eq!(same.generation, first.generation);
        let renewed = lifecycle
            .ensure_current(&identity, &config, now + time::Duration::days(24))
            .unwrap();
        assert_eq!(renewed.generation, 2);
        config.address = Some("192.168.1.21".parse().unwrap());
        let changed = lifecycle
            .ensure_current(&identity, &config, now + time::Duration::days(25))
            .unwrap();
        assert_eq!(changed.generation, 3);
        assert!(changed.names.contains(&"192.168.1.21".into()));
    }
    #[test]
    fn activation_audit_fails_closed_until_every_external_gate_is_evidenced() {
        let (audit, permit) = audit_activation(LanSecurityEvidence {
            browser_trust_onboarding: true,
            native_protected_storage: false,
            negative_security_suite: true,
        });
        assert!(!audit.passed);
        assert_eq!(audit.failed_gates, vec!["native_protected_storage"]);
        assert!(permit.is_none());
        let (audit, permit) = audit_activation(LanSecurityEvidence {
            browser_trust_onboarding: true,
            native_protected_storage: true,
            negative_security_suite: true,
        });
        assert!(audit.passed);
        assert!(permit.is_some());
    }
}
