# Current Task

## ID

LS-010

## Title

Trusted-peer administration and revocation

## Status

Completed

## Goal

Let the local desktop user list active trusted peers and deliberately revoke their credentials using safe metadata only, persistent core behavior, thin Tauri adapters, and confirmation-based Vue state.

## Acceptance Criteria

- The core returns active peer summaries containing only opaque ID, display name, capability, and creation timestamp.
- Token plaintext, token digests, filesystem paths, and revoked records never enter the peer-list response.
- Revocation remains idempotent, persists across restart, removes the peer from active listings, and prevents authentication.
- Thin Tauri commands expose only local peer listing and revocation.
- A composable owns load/error/notice, confirmation selection, cancellation, and revocation state.
- Revocation requires a distinct confirmation step and failed revocation retains the peer for retry.
- An accessible component renders active peers, capability, paired date, empty/loading/error states, and confirmation-oriented controls.
- Tests cover safe listing, persistent revocation, composable confirmation, success, failure retention, and cancellation.
- No HTTP routes or bind changes are introduced.

## Completed

- Added SQLite active-peer listing ordered by creation/name and excluding revoked records.
- Added safe `TrustedPeerSummary` serialization with only opaque ID, display name, `library.read`, and creation time.
- Added core tests proving no token/digest/path fields, restart-persistent revocation, active-list removal, and idempotence.
- Added thin local `trusted_peers` and `revoke_trusted_peer` Tauri commands.
- Added `useTrustedPeers` with load, confirmation selection, cancellation, revocation, success notice, and failure retention.
- Added four deterministic composable tests covering safe load, confirmation/cancellation, success, and retryable failure.
- Added an accessible trusted-device panel with paired date, capability label, refresh, empty/loading/error states, and a separate alert-dialog confirmation step.
- Updated root, core, database, Tauri, frontend, component, composable, security, and project documentation.

## Tests Last Executed

- `npm run verify` — PASS; format, lint, typecheck, 6 files / 21 frontend tests, and production build passed.
- `cargo fmt --all --check` — PASS.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — PASS.
- `cargo test --workspace` — PASS; 28 core/workspace tests passed.
- `cargo check --workspace` — PASS.
- `git diff --check` — PASS before final task/handoff documentation update.

## Tests Not Yet Executed

- Interactive Windows Tauri trusted-peer refresh, confirmation, cancellation, and revocation.
- Screen-reader and keyboard focus behavior in the target Windows webview.
- Non-Windows platforms.

## Known Problems

- None confirmed for the LS-010 automated scope.

## Assumptions

- Revoked records remain stored so their old credentials continue to receive revoked semantics.
- Active UI listings intentionally exclude revoked records.
- Unix creation seconds are safe as JavaScript numbers for relevant dates.
- ADR-0006 continues to prohibit LAN binding.

## Next Exact Step

Define LS-011 for bearer-token extraction and `library.read` authorization middleware tested on a protected loopback router while preserving the existing local desktop router and bind behavior.
