# LS-016 Handoff — Short-lived leaf-certificate issuance

## Objective

Issue bounded-lifetime TLS server leaves from the persistent node root for validated explicit DNS/IP names without starting a listener.

## State

Completed on 2026-08-18. Changes remain uncommitted in the existing working tree.

## Implemented

- `NodeIdentity` retains the protected root signing key and certificate parameters without exposing them through serialization or debug output.
- `issue_server_leaf` generates a new P-256 key per call and signs a 30-day leaf with the persistent root.
- Leaves contain only digital-signature key usage and TLS server-auth extended usage.
- SAN validation accepts canonicalized DNS names and IP addresses, with a 16-name cap; empty, wildcard, malformed, duplicate, and excessive input fails safely.
- `ServerLeaf` exposes only its leaf-first certificate chain and redacts its signing key from debug output.
- X.509 parsing tests verify the real signature against the root, SAN encodings, usage extensions, bounded validity, unique public keys, chain order, and invalid inputs.

## Changed Files

- `crates/localstream-core/src/node_identity/mod.rs`
- `crates/localstream-core/src/node_identity/README.md`
- `crates/localstream-core/Cargo.toml`
- `Cargo.lock`
- `docs/security/README.md`
- `.ai/PROJECT_STATUS.md`
- `.ai/CURRENT_TASK.md`

## Verified

- `cargo test -p localstream-core node_identity --locked` — PASS; 6 focused tests.
- `cargo test --workspace --locked` — PASS; 41 Rust tests.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` — PASS.
- `cargo check --workspace --locked` — PASS.
- `cargo fmt --all` — PASS.

## Not Verified

- TLS handshakes or hostname verification; no TLS configuration/listener exists yet.
- Automatic renewal, address-change rotation, or active-connection behavior.
- Real OS protected-store operation.

## Next Exact Action

Implement LS-017 by converting `ServerLeaf` directly into a hardened Rustls `ServerConfig`, keeping PKCS#8 material inside the module and adding configuration/negative tests without binding a socket.
