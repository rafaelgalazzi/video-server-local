# Current Task

## ID

LS-004

## Title

Embedded Axum HTTP server and versioned library API

## Status

Completed

## Goal

Run an embedded Axum server inside LocalStream with versioned health and path-free library endpoints backed by the same Rust core used by Tauri.

## Acceptance Criteria

- Axum runs in-process with graceful shutdown and no separate backend executable.
- `GET /api/v1/health` returns stable service/capability information.
- `GET /api/v1/library` delegates to `LocalStreamCore::current_library` and never returns raw paths.
- API errors use a stable JSON envelope without internal details.
- The server binds to loopback on an ephemeral port until pairing/authentication is implemented.
- Tauri exposes safe server address/status information to Vue.
- Handler/contract and lifecycle tests pass.
- API, security, status, and source documentation are current.

## Relevant Files

- `crates/localstream-core/src/server/`
- `src-tauri/src/lib.rs`
- `src/composables/`
- `docs/api/README.md`
- `docs/security/README.md`
- `docs/development/TEST_MATRIX.md`

## Completed

- LS-003 SQLite persistence is complete and verified by automated checks.
- Reviewed ADR-0003, the API source of truth, and the security model.
- Added an in-process Axum router and graceful lifecycle handle.
- Added versioned health and path-free current-library endpoints with stable error JSON.
- Bound the pre-authentication server to an ephemeral loopback address only.
- Initialized the shared core/server in Tauri and exposed safe server status to Vue.
- Added router contract, path non-disclosure, loopback lifecycle, and frontend state tests.
- Launched LocalStream on Windows and queried its real health endpoint successfully.

## In Progress

- None.

## Remaining

- None for LS-004.

## Tests Last Executed

- `npm run verify` — PASS; 3 files and 8 frontend tests passed.
- `cargo fmt --all --check` — PASS.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — PASS.
- `cargo test --workspace` — PASS; 11 core/workspace tests passed.
- `cargo check --workspace` — PASS.
- Windows Tauri launch — PASS; visible `LocalStream` window responding.
- `GET http://127.0.0.1:53744/api/v1/health` — PASS with versioned JSON on the tested run.

## Tests Not Yet Executed

- LAN binding, authentication/pairing, static web hosting, streaming, and non-Windows platforms.

## Known Problems

- The server intentionally cannot be reached from another device until pairing/authentication exists.
- The port is ephemeral and changes between runs.

## Assumptions

- Loopback-only binding is mandatory until pairing/authentication protects LAN routes.
- An ephemeral port avoids conflicts for this foundation; configurable LAN binding is a later task.
- Static web UI hosting and media streaming are out of scope for LS-004.

## Next Exact Step

Define LS-005 for pairing/authentication foundations before enabling LAN exposure, or explicitly scope a loopback Direct Play/HTTP Range slice first.
