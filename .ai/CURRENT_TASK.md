# Current Task

## ID

LS-007

## Title

Revocable peer credential foundation and threat model

## Status

Completed

## Goal

Establish secure, persisted, revocable peer credentials and an explicit trust model in the reusable Rust core without exposing an incomplete pairing protocol or enabling LAN binding.

## Acceptance Criteria

- The core can issue a cryptographically random bearer credential only through a trusted local call.
- Plaintext credentials are returned once and never persisted; SQLite stores a SHA-256 digest and safe peer metadata.
- Credentials carry an explicit initial `library.read` capability.
- Authentication distinguishes trusted, missing/invalid, and revoked credentials without exposing secrets.
- Revocation persists across core restarts and prevents future authentication.
- Database schema migration from version 1 to version 2 preserves existing library data.
- Tests cover issuance, valid/invalid authentication, revocation, restart persistence, and schema migration.
- An accepted ADR and security threat model document credential lifecycle, trust boundaries, route policy, transport limitations, and LAN bind gates.
- The HTTP listener remains loopback-only and existing routes remain unchanged for the desktop client.

## Completed

- Added 256-bit OS-random, URL-safe peer bearer credential issuance with strict input/token shape validation.
- Plaintext credentials are returned once and intentionally do not implement `Debug`; only SHA-256 digests are stored.
- Added safe peer identity, explicit `library.read` capability, authentication outcomes, and persistent revocation.
- Migrated SQLite schema from v1 to v2 with trusted-peer storage while preserving current library data.
- Added issuance, missing/invalid credential, authentication, revocation, restart persistence, plaintext non-storage, validation, and v1 migration tests.
- Accepted ADR-0006 and expanded the security document with assets, threats, mitigations, route policy, and mandatory LAN gates.
- Updated API, architecture, test matrix, core/database/server READMEs, dependencies, and project status.

## Tests Last Executed

- `npm run verify` — PASS; format, lint, typecheck, 4 files / 12 frontend tests, and production build passed.
- `cargo fmt --all --check` — PASS.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — PASS.
- `cargo test --workspace` — PASS; 24 core/workspace tests passed.
- `cargo check --workspace` — PASS.
- `git diff --check` — PASS before final task/handoff documentation update.

## Tests Not Yet Executed

- Remote pairing protocol, user approval, HTTP bearer middleware, encrypted transport, client secure storage, and LAN binding; these do not exist yet.
- Interactive credential management UI and non-Windows platforms.

## Known Problems

- None confirmed for the LS-007 scope.

## Assumptions

- `library.read` is the only capability in this foundation; unknown stored capabilities fail closed.
- Credential issuance remains core-only and trusted-local until an explicit approval protocol exists.
- Bearer credentials must never cross plaintext LAN HTTP.
- Existing loopback routes remain unauthenticated for the local Tauri webview.

## Next Exact Step

Define LS-008 for bounded, expiring, replay-resistant pairing requests and explicit local approval while retaining loopback-only binding.
