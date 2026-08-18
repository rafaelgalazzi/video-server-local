# LS-013 Handoff — Persistent node-root identity

## Objective

Implement a reusable fail-closed private-CA identity service and protected-storage boundary without adding TLS serving or LAN exposure.

## State

Completed on 2026-08-18. LS-013 changes are uncommitted at handoff time; preserve them.

## Implemented

- `NodeIdentityService` loads existing PKCS#8 root material or generates and stores a P-256 key before returning identity.
- `NodeIdentitySummary` exposes only a stable `ls_node_` ID and colon-formatted SHA-256 SPKI fingerprint.
- Generated root certificates are CA-constrained and use key-cert-sign, CRL-sign, and digital-signature usages.
- `NodeSecretStore` isolates secret persistence from certificate/domain behavior.
- `KeyringNodeSecretStore` selects Windows Credential Manager, Apple Keychain, and Linux Secret Service backends through `keyring 3.6.3`.
- Missing identity creates once; corrupt and unavailable stores fail closed without replacement or ephemeral success.
- Private key material is not serializable, debugged, retained in the returned public identity, stored in SQLite, or exposed through an adapter.
- Tests use an injected memory store and do not access the developer OS keyring.
- `time 0.3.36` is pinned for the new certificate graph; TD-001 records pre-existing workspace MSRV drift.

## Changed Files

- `crates/localstream-core/src/node_identity/mod.rs`
- `crates/localstream-core/src/node_identity/README.md`
- `crates/localstream-core/src/lib.rs`
- `crates/localstream-core/Cargo.toml`
- `Cargo.lock`
- Core/security/test-matrix and repository-memory documentation.

## Verified

- `npm run verify` — PASS; format, lint, typecheck, 21 tests across 6 files, and production build.
- `cargo fmt --all --check` — PASS.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` — PASS.
- `cargo test --workspace --locked` — PASS; 36 Rust tests.
- `cargo check --workspace --locked` — PASS.
- Focused `cargo test -p localstream-core node_identity --locked` — PASS; 3 tests.

## Not Verified

- Real Windows Credential Manager reads/writes; automated tests intentionally use memory storage.
- Apple Keychain, Linux Secret Service, mobile targets, and headless protected stores.
- Rust 1.77.2 execution because only Rust 1.97.1 is installed. See TD-001 for pre-existing lockfile incompatibilities.
- Startup integration, identity-reset revocation, TLS leaf issuance, HTTPS, certificate trust installation, or LAN behavior.

## Security-Critical Notes

- Do not add a plaintext file fallback for keyring failure.
- Do not silently replace corrupt or missing-after-initialization identity; identity reset must later revoke peers and sessions.
- Serialize startup initialization so competing creators cannot produce different roots.
- Keep the SPKI fingerprint as the stable identity. A regenerated self-signed certificate encoding is not a new identity when the root key is unchanged.
- Do not expose PKCS#8, keyring errors, or certificate internals through Tauri or HTTP.
- LAN binding remains prohibited.

## Next Exact Action

Create LS-014 to instantiate one `KeyringNodeSecretStore` during desktop setup, load the identity before any future remote listener, manage only its cloned `NodeIdentitySummary`, and expose that summary through a thin trusted-local Tauri command and accessible local UI. Add adapter/composable tests and a Windows interactive check if feasible. Do not add TLS, pairing HTTP routes, certificate installation, or LAN binding.
