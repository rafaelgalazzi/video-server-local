# Node Identity

## Purpose

Own the persistent private root identity required by ADR-0007 without coupling certificate generation to Tauri, HTTP, or a particular deployment.

## Features

- P-256 root signing-key generation through `rcgen`/ring.
- Stable node ID and SHA-256 SPKI fingerprint derived from the public key.
- Self-signed CA certificate material for later leaf issuance.
- Fail-closed loading: corrupt stored keys are never silently replaced.
- A narrow secret-store boundary plus a platform keyring adapter.

## Important Files

- `mod.rs`: identity service, public summary, secret-store contract, keyring adapter, and deterministic lifecycle tests.

## Public Interfaces

- `NodeIdentityService::load_or_create`: loads the root key or creates and stores it exactly once.
- `NodeIdentitySummary`: non-secret stable node ID and display fingerprint.
- `NodeSecretStore`: storage boundary for platform and headless adapters.
- `KeyringNodeSecretStore`: Windows Credential Manager, Apple Keychain, or Linux Secret Service adapter.

## Dependencies

`rcgen` generates and restores PKCS#8 key material and CA certificates. SHA-256 and URL-safe Base64 derive non-secret identity values. `keyring` selects native credential backends by target platform.

## Limitations

The service is not connected to application startup, TLS, leaf issuance, trust installation, identity reset, or LAN binding. Linux desktop persistence requires an available Secret Service. Headless systems must provide a separately reviewed `NodeSecretStore`; plaintext fallback is intentionally absent.

## Planned Work

Integrate one serialized startup initializer, define explicit identity reset with peer/session revocation, issue short-lived leaf certificates, and add loopback-only TLS tests before any LAN exposure.
