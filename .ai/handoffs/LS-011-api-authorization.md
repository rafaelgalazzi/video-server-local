# LS-011 Handoff — API authorization middleware

## Objective

Add a separately testable authenticated Axum router that protects library metadata and media streams with strict bearer credentials and `library.read`, without changing the active desktop loopback listener.

## State

Completed on 2026-08-18. LS-011 changes are uncommitted at handoff time; preserve them.

## Implemented

- `authenticated_router` keeps health public and protects library and stream routes.
- Authorization accepts exactly one UTF-8 `Authorization: Bearer <token>` header with a non-empty token.
- Missing, malformed, duplicate, unknown, and revoked credentials share one safe `401` response and `WWW-Authenticate: Bearer`.
- Credential-store failures map to the existing generic `500` response without revealing authentication details.
- Successful authentication requires `library.read` and places the safe `TrustedPeer` identity in request extensions.
- Existing library and streaming handlers remain thin and reusable, including bounded ranged Direct Play.
- The existing `router` and `start_local_server` remain unchanged and loopback-only for the desktop webview.

## Changed Files

- `crates/localstream-core/src/server/mod.rs`
- `crates/localstream-core/src/server/README.md`
- `docs/api/README.md`
- `docs/security/README.md`
- `docs/development/TEST_MATRIX.md`
- `.ai/DEFERRED_DECISIONS.md`
- `.ai/PROJECT_STATUS.md`
- `.ai/CURRENT_TASK.md`

## Verified

- `npm run verify` — PASS; format, lint, typecheck, 21 tests across 6 files, and production build.
- `cargo fmt --all --check` — PASS.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — PASS.
- `cargo test --workspace` — PASS; 33 tests.
- `cargo check --workspace` — PASS.

## Not Verified

- Interactive or browser playback through the dormant authenticated router.
- Encrypted transport, certificate trust, or any LAN listener.
- Non-Windows platforms.

## Security-Critical Notes

- Do not attach `authenticated_router` to plaintext HTTP or enable LAN binding.
- Preserve uniform unauthorized responses; do not reveal whether a credential is unknown or revoked.
- Browser media elements cannot attach arbitrary bearer headers. DD-007 defers the same-origin session or signed-stream design.
- The desktop player remains a local preview using the trusted loopback router.

## Next Exact Action

Create LS-012 as an architecture/design task for authenticated encrypted LAN server identity and transport. Resolve certificate identity/trust, listener lifecycle, browser origin, and secret transport before adding remote routes or changing binding.
