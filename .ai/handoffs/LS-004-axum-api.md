# Handoff — LS-004

## Objective

Run an embedded versioned Axum API backed by the same core as Tauri without exposing unpaired LAN services or filesystem paths.

## Current State

Completed and verified through contract tests plus a real Windows loopback health request.

## Changed Files

- `crates/localstream-core/src/server/` router, lifecycle, response models, tests, and documentation.
- Core dependencies, exports, documentation, and `Cargo.lock`.
- Tauri shared-core/server initialization and safe `server_info` adapter.
- Vue server-status composable, component, tests, and styling.
- API, security, architecture, status, and test documentation.

## Important Decisions

- The server binds to loopback only until authentication/pairing exists.
- The port is ephemeral and reported through a safe Tauri adapter.
- HTTP handlers call `LocalStreamCore`; they do not access SQLite directly.
- Library JSON reuses the path-free core model.

## Completed

- Graceful in-process server lifecycle.
- `/api/v1/health` and `/api/v1/library`.
- Stable generic error envelope.
- Desktop status display and full automated verification.

## Remaining

- None for LS-004.

## Tests Executed

- `npm run verify` — PASS; 8 frontend tests.
- `cargo fmt --all --check` — PASS.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — PASS.
- `cargo test --workspace` — PASS; 11 tests.
- `cargo check --workspace` — PASS.
- Windows Tauri launch and real loopback health request — PASS.

## Tests Not Executed

- LAN access, because it is intentionally disabled.
- Non-Windows platforms.

## Known Failures

- None.

## Assumptions

- Ephemeral loopback binding is appropriate before the trust model exists.
- Static hosting and streaming receive separate tasks.

## Next Exact Action

Define LS-005 for authentication/pairing foundations before enabling LAN binding, or explicitly scope loopback-only Direct Play/HTTP Range first.

## Do Not

- Bind to `0.0.0.0` before authentication and pairing are implemented.
- Add path fields to HTTP responses.
- Put database or media business logic in Axum handlers.
