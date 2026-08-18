# LS-007 Handoff — Revocable peer credentials and threat model

## Objective

Build and document reusable credential mechanics before exposing any pairing or LAN network surface.

## State

Completed on 2026-08-18. LS-007 changes are uncommitted at handoff time; preserve them.

## Implemented

- Core-only issuance of `ls_peer_` bearer credentials containing 256 OS-random bits.
- Strict token shape validation before hashing and display-name rejection for empty, oversized, or control-character input.
- SHA-256-only token persistence; plaintext is returned once and `IssuedCredential` intentionally lacks `Debug`/serialization.
- Safe `TrustedPeer` metadata with the single fail-closed `library.read` capability.
- Missing, invalid, revoked, and store/randomness error outcomes.
- Persistent revocation and restart-safe authentication.
- SQLite schema v2 plus a tested v1→v2 migration preserving library/media rows.
- ADR-0006 and a threat model with mandatory gates that still prohibit LAN binding.

## Changed Files

- `crates/localstream-core/src/auth/`
- `crates/localstream-core/src/database/mod.rs`
- `crates/localstream-core/src/lib.rs`
- `crates/localstream-core/Cargo.toml` and `Cargo.lock`
- `docs/architecture/adr/0006-revocable-peer-credentials.md`
- Canonical security, API, architecture, test-matrix, status, and core README documentation.

## Verified

- `npm run verify` — PASS; 12 frontend tests and production build.
- `cargo fmt --all --check` — PASS.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — PASS.
- `cargo test --workspace` — PASS; 24 tests.
- `cargo check --workspace` — PASS.
- `git diff --check` — PASS before final documentation-only edits.

## Not Verified / Not Implemented

- Pair request/confirmation endpoints, request expiration, replay protection, approval UI, and rate limiting.
- HTTP bearer extraction and route capability enforcement.
- Encrypted LAN transport and authenticated server identity.
- Client-side secure credential storage, peer administration UI, and LAN binding.
- Non-Windows platforms.

## Security-Critical Notes

- Do not expose `issue_peer_credential` directly as an unauthenticated command or HTTP handler.
- Do not send bearer credentials over plaintext LAN HTTP.
- Do not change the bind address until every ADR-0006 gate is implemented and negatively tested.
- HTTP error mapping should not reveal whether a presented remote token is unknown or revoked, even though the core distinguishes those states for administration/audit behavior.
- Keep secrets out of logs, serialization, panic messages, and debug output.

## Next Exact Action

Create LS-008 for an in-memory bounded pairing-request service with cryptographic request IDs, short-lived verification codes, monotonic expiration handling, single-use approval/rejection, and a trusted-local approval adapter. Keep it loopback-only; channel protection and LAN binding remain later gates.
