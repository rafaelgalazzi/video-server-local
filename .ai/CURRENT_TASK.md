# Current Task

## ID

LS-008

## Title

Bounded user-approved pairing requests

## Status

Completed

## Goal

Add a reusable, in-memory pairing-request lifecycle with cryptographic request/claim secrets, short-lived verification codes, explicit trusted-local approval or rejection, and single-use credential issuance while retaining loopback-only networking.

## Acceptance Criteria

- Pairing requests use cryptographically random request IDs and 256-bit claim secrets.
- Each request has a six-digit verification code and expires using monotonic time.
- Pending requests are bounded globally and invalid display names are rejected.
- Trusted-local callers can list pending requests and explicitly approve with the matching code or reject them.
- Credential claim requires the matching high-entropy secret and an approved, unexpired request.
- Approved requests issue exactly one LS-007 credential; replay and rejected requests fail closed.
- Secret-bearing receipts and issued credentials are not serializable or debug-printable.
- Thin Tauri commands expose only pending-list, approve, and reject operations—not remote request creation or credential claim.
- Tests cover valid flow, invalid code/secret, rejection, expiration, replay, and capacity.
- HTTP routes and loopback binding remain unchanged.

## Completed

- Added an in-memory pairing service limited to 32 active requests with two-minute monotonic expiration.
- Added collision-checked 128-bit request IDs, uniformly generated six-digit verification codes, and 256-bit claim secrets.
- Stored only claim-secret digests in memory and compare them in constant time after strict shape validation.
- Added pending listing plus explicit approve/reject decisions; approved requests disappear from the pending view.
- Added single-use credential claims with bounded consumed/rejected/expired tombstones and fail-closed replay behavior.
- Added thin trusted-local Tauri list/approve/reject commands without exposing request creation or credential claiming.
- Added deterministic tests for approval, invalid code/secret, rejection, expiration, replay, capacity, and credential authentication.
- Updated auth/core/Tauri documentation, API/security boundaries, test matrix, and project status.

## Tests Last Executed

- `npm run verify` — PASS; format, lint, typecheck, 4 files / 12 frontend tests, and production build passed.
- `cargo fmt --all --check` — PASS.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — PASS.
- `cargo test --workspace` — PASS; 27 core/workspace tests passed.
- `cargo check --workspace` — PASS.
- `git diff --check` — PASS before final task/handoff documentation update.

## Tests Not Yet Executed

- Pairing approval UI, remote encrypted pairing routes, network rate limiting, and HTTP authorization middleware; these do not exist.
- Interactive Tauri approval actions and non-Windows platforms.

## Known Problems

- None confirmed for the LS-008 scope.

## Assumptions

- Pairing request creation and credential claim remain core APIs without HTTP or Tauri exposure.
- Verification codes support human comparison; only the 256-bit claim secret authorizes credential claiming.
- Requests intentionally disappear on process restart.
- LAN binding remains prohibited by ADR-0006.

## Next Exact Step

Define LS-009 for a Vue trusted-local pairing approval interface that polls pending requests and invokes only approve/reject commands while networking remains loopback-only.
