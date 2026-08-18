# LS-018 Handoff — Loopback authenticated HTTPS lifecycle

## Objective

Serve the authenticated Axum router through a separately testable loopback-only TLS lifecycle without changing desktop startup or exposing the LAN.

## State

Completed on 2026-08-18. Changes remain uncommitted in the existing working tree.

## Implemented

- `start_loopback_https_server` binds only `127.0.0.1:0`, issues a leaf for localhost/loopback addresses, and fails before spawning if listener, identity, or TLS setup fails.
- Tokio-Rustls accepts encrypted connections; Hyper HTTP/1.1 serves the existing authenticated Axum router without duplicated handler logic.
- TLS handshake failures are contained per connection and never receive a plaintext fallback response.
- `HttpsServerHandle::shutdown` stops acceptance, signals active HTTP connections gracefully, waits for completion, and releases the listener.
- The active `start_local_server` and Tauri setup remain unchanged.
- Real-socket tests verify trusted health, unauthorized and authorized library responses, wrong-root/name failures, plaintext downgrade rejection, loopback scope, graceful shutdown, and port rebinding.

## Changed Files

- `crates/localstream-core/src/server/mod.rs`
- `crates/localstream-core/src/server/README.md`
- `crates/localstream-core/Cargo.toml`
- `Cargo.lock`
- `docs/security/README.md`
- `.ai/PROJECT_STATUS.md`
- `.ai/CURRENT_TASK.md`

## Verified

- `cargo test -p localstream-core https_lifecycle --locked -- --nocapture` — PASS; 2 real-socket HTTPS tests.
- `cargo test --workspace --locked` — PASS; 46 Rust tests.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` — PASS.
- `cargo check --workspace --locked` — PASS.
- `cargo fmt --all --check` — PASS.

## Not Verified

- Desktop startup with HTTPS, non-loopback addresses, certificate installation, renewal, or address changes.
- Long-running connection drain under production load.

## Next Exact Action

Implement LS-019 trusted-local root-certificate export to a user-selected file with full fingerprint guidance. Do not automatically install trust or expose an unauthenticated certificate download route.
