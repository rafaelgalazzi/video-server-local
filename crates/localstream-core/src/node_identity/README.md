# Node Identity

## Purpose

Own the persistent private root identity required by ADR-0007 without coupling certificate generation to Tauri, HTTP, or a particular deployment.

## Features

- P-256 root signing-key generation through `rcgen`/ring.
- Stable node ID and SHA-256 SPKI fingerprint derived from the public key.
- Self-signed CA certificate material for later leaf issuance.
- Fail-closed loading: corrupt stored keys are never silently replaced.
- A narrow secret-store boundary plus a platform keyring adapter.
- Short-lived server leaf issuance for validated DNS/IP subject alternative names.
- Hardened Rustls configuration with TLS 1.3/1.2 and HTTP/1.1 ALPN.

## Important Files

- `mod.rs`: identity service, public summary, secret-store contract, keyring adapter, and deterministic lifecycle tests.

## Public Interfaces

- `NodeIdentityService::load_or_create`: loads the root key or creates and stores it exactly once.
- `NodeIdentityService::load_existing`: restores an identity without ever creating a missing replacement.
- `NodeIdentitySummary`: non-secret stable node ID and display fingerprint.
- `NodeSecretStore`: storage boundary for platform and headless adapters.
- `KeyringNodeSecretStore`: Windows Credential Manager, Apple Keychain, or Linux Secret Service adapter.
- `LocalStreamCore::reset_node_identity`: revokes all persisted peer trust before deleting the protected root.
- `NodeIdentity::issue_server_leaf`: creates a fresh P-256 server key and a root-signed 30-day leaf-first chain.
- `ServerLeaf::into_server_config`: consumes private leaf material directly into Rustls.

## Dependencies

`rcgen` generates and restores PKCS#8 key material and CA certificates. SHA-256 and URL-safe Base64 derive non-secret identity values. `keyring` selects native credential backends by target platform.

## Limitations

Desktop startup loads this service before the loopback server and retains only its public summary. Trusted-local export reloads existing identity without creation, verifies the startup summary, and writes only public root DER to a user-selected file. Explicit reset revokes all peers before deleting the protected root and requires restart before regeneration. Leaf issuance accepts at most 16 unique explicit DNS/IP names, rejects wildcards, and consumes leaf keys directly into a Rustls configuration. TLS permits only 1.3/1.2, requests no client certificate, and advertises only HTTP/1.1. Automatic trust installation, remote certificate download, and LAN binding remain prohibited.

## Planned Work

Add loopback-only HTTPS lifecycle tests before any LAN exposure.
