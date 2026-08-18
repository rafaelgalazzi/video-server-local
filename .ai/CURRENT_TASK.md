# Current Task

## ID

LS-013

## Title

Persistent node-root identity and protected storage

## Status

Completed

## Goal

Implement a reusable fail-closed node-root identity service with stable non-secret identity, platform protected-key storage, and deterministic lifecycle tests without adding TLS serving or LAN binding.

## Acceptance Criteria

- A reusable core module generates a CA-capable P-256 root key and certificate.
- Stable node ID and display fingerprint derive from the root SubjectPublicKeyInfo.
- Private PKCS#8 material crosses only a narrow secret-store boundary and is never serializable or debugged.
- A platform adapter targets Windows Credential Manager, Apple Keychain, and Linux Secret Service.
- Missing identity is generated and stored before being returned.
- Existing identity restores the same node ID and fingerprint across service reconstruction.
- Corrupt or unavailable storage fails closed without silent identity replacement.
- Unit tests use an injected in-memory store and do not mutate the developer's OS keyring.
- No HTTP route, TLS listener, Tauri command, certificate installation, identity reset, or bind change is introduced.

## Relevant Files

- `crates/localstream-core/src/node_identity/mod.rs`
- `crates/localstream-core/src/node_identity/README.md`
- `crates/localstream-core/Cargo.toml`
- `Cargo.lock`
- `docs/security/README.md`
- `.ai/PROJECT_STATUS.md`

## Completed

- Reviewed LS-012, ADR-0007, existing persistence patterns, server boundaries, and platform startup.
- Evaluated current compatible `rcgen` and `keyring` releases against the workspace Rust 1.77.2 requirement.
- Added the initial identity service, public summary, keyring adapter, and fail-closed lifecycle tests.
- Pinned the new certificate time dependency to a Rust-1.77-compatible release and recorded broader pre-existing MSRV drift as TD-001.
- Verified all frontend and Rust checks with 21 frontend tests and 36 Rust tests.

## In Progress

- Nothing.

## Remaining

- Nothing for LS-013.

## Assumptions

- The root SPKI, rather than a particular self-signed certificate encoding, is the stable node identity.
- The service has one serialized startup initializer; cross-process concurrent creation is outside LS-013 and must fail closed at integration time.
- Headless deployments must inject a separately reviewed protected store and receive no plaintext fallback.

## Next Exact Step

Start LS-014 by integrating one serialized node-identity initializer into desktop startup and exposing only `NodeIdentitySummary` through a trusted-local Tauri command/UI, without adding TLS serving or LAN binding.
