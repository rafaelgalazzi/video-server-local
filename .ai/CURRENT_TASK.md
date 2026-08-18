# Current Task

## ID

LS-009

## Title

Trusted-local pairing approval interface

## Status

Completed

## Goal

Let the local desktop user review pending pairing requests and explicitly approve or reject them through thin Tauri decision commands without exposing request creation, credential claiming, or LAN access.

## Acceptance Criteria

- A pairing composable owns pending requests, loading/error/notice state, per-request decision state, and polling lifecycle.
- Polling starts only after a successful native load and stops when the Vue shell unmounts.
- Approval sends only the opaque request ID and displayed verification code; rejection sends only the request ID.
- Successful decisions remove the request locally and provide accessible feedback.
- Failed decisions retain the request and expose a retryable safe error.
- A dedicated accessible component displays requesting-device name, grouped six-digit code, approximate expiry, and clear Allow/Reject actions.
- The UI warns users to approve only when the code matches the requesting device.
- Tests cover load, approval, rejection failure, and polling/cleanup.
- No HTTP routes, request creation/claim adapter, or bind changes are introduced.

## Completed

- Added `usePairingRequests` with typed Tauri adapters, pending state, decision state, safe feedback, and five-second polling.
- Polling starts only after a successful first native load and is explicitly stopped when the app unmounts.
- Successful approve/reject decisions remove requests locally; failed decisions retain them for retry.
- Added five deterministic tests for load, exact approval arguments, rejection failure retention, polling/cleanup, and no polling after initial failure.
- Added an accessible pairing panel with device name, grouped code, approximate expiry, comparison warning, Allow/Reject actions, retry, empty, loading, notice, and error states.
- Integrated the panel into the Vue shell without adding request creation, claim, HTTP, or bind capabilities.
- Updated root/frontend/component/composable documentation, security model, and project status.

## Tests Last Executed

- `npm run verify` — PASS; format, lint, typecheck, 5 files / 17 frontend tests, and production build passed.
- `cargo fmt --all --check` — PASS.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — PASS.
- `cargo test --workspace` — PASS; 27 core/workspace tests passed.
- `cargo check --workspace` — PASS.
- `git diff --check` — PASS before final task/handoff documentation update.

## Tests Not Yet Executed

- Interactive Windows Tauri rendering, polling, approval, rejection, and retry with a real pending request.
- Remote encrypted pairing transport, because it does not exist.
- Non-Windows platforms.

## Known Problems

- None confirmed for the LS-009 automated scope.

## Assumptions

- Five-second polling is appropriate for short-lived local approval requests.
- Browser preview performs one failed native load and does not continue polling.
- Verification codes are passed back exactly as displayed after the user chooses Allow.
- ADR-0006 continues to prohibit LAN binding.

## Next Exact Step

Define LS-010 for safe trusted-peer listing and persistent revocation controls in the core, Tauri adapter, and Vue UI.
