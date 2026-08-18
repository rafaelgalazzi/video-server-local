# LS-017 Handoff — Fail-closed Rustls server configuration

## Objective

Consume issued leaf material directly into a hardened TLS server configuration and verify trust/name/protocol behavior without opening a socket.

## State

Completed on 2026-08-18. Changes remain uncommitted in the existing working tree.

## Implemented

- `ServerLeaf::into_server_config` consumes the generated PKCS#8 key directly into Rustls without a public private-key accessor.
- Server policy explicitly enables TLS 1.3 and TLS 1.2, disables client-certificate requests, and advertises only `http/1.1` ALPN.
- Certificate configuration errors map to safe domain errors without returning certificate/key internals.
- In-memory Rustls handshakes succeed independently under TLS 1.3 and TLS 1.2 with the correct root and hostname.
- Negative handshakes reject an unrelated root and an unlisted hostname.
- Corrupt leaf DER fails closed during configuration.

## Changed Files

- `crates/localstream-core/src/node_identity/mod.rs`
- `crates/localstream-core/src/node_identity/README.md`
- `crates/localstream-core/Cargo.toml`
- `Cargo.lock`
- `docs/security/README.md`
- `.ai/PROJECT_STATUS.md`
- `.ai/CURRENT_TASK.md`

## Verified

- `cargo test -p localstream-core node_identity --locked` — PASS; 9 focused tests.
- `cargo test --workspace --locked` — PASS; 44 Rust tests.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` — PASS.
- `cargo check --workspace --locked` — PASS.
- `cargo fmt --all` — PASS.

## Not Verified

- Socket-level HTTPS, HTTP request handling over TLS, shutdown, or renewal.
- Browser/OS trust installation or real protected-store operation.

## Next Exact Action

Implement LS-018 as a separate loopback-only HTTPS lifecycle around the authenticated router, with trusted-client HTTP requests, plaintext downgrade rejection, graceful shutdown, and no desktop-startup or LAN bind change.
