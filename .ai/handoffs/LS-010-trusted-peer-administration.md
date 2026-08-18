# LS-010 Handoff — Trusted-peer administration and revocation

## Objective

Close ADR-0006's local revocation-control gate with safe active-peer metadata and a deliberate desktop revocation flow.

## State

Completed on 2026-08-18. LS-010 changes are uncommitted at handoff time; preserve them.

## Implemented

- Database query returns only non-revoked peers ordered by creation time, display name, and ID.
- Public `TrustedPeerSummary` contains only opaque ID, display name, `library_read`, and Unix creation seconds.
- Unknown stored capabilities fail closed; revoked records cannot enter active listings.
- Revocation stays idempotent, persists across restart, removes active peers, and invalidates credentials.
- Tauri exposes only safe local list/revoke commands.
- `useTrustedPeers` owns loading, confirmation, cancellation, revocation, notices, and retryable errors.
- `TrustedPeersPanel.vue` displays paired date/capability and requires a separate confirmation action.
- No HTTP route, credential material, filesystem data, or bind change was added.

## Changed Files

- `crates/localstream-core/src/database/mod.rs`
- `crates/localstream-core/src/auth/mod.rs`
- `crates/localstream-core/src/lib.rs`
- `src-tauri/src/lib.rs`
- `src/composables/useTrustedPeers.ts`
- `src/composables/useTrustedPeers.test.ts`
- `src/components/TrustedPeersPanel.vue`
- `src/App.vue`, `src/styles.css`, and canonical documentation.

## Verified

- `npm run verify` — PASS; format, lint, typecheck, 21 tests across 6 files, and production build.
- `cargo fmt --all --check` — PASS.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — PASS.
- `cargo test --workspace` — PASS; 28 tests.
- `cargo check --workspace` — PASS.
- `git diff --check` — PASS before final documentation-only edits.

## Not Verified

- Interactive Tauri peer administration with real persisted peers.
- Target-webview screen-reader/focus behavior.
- Non-Windows platforms.

## Security-Critical Notes

- Never add token plaintext or token digests to `TrustedPeerSummary`.
- Keep peer list/revoke adapters local-only; remote `library.read` clients must never administer peers.
- Do not delete revoked database records while revoked-vs-invalid credential semantics are required.
- Revocation controls satisfy only one ADR-0006 gate; encrypted transport and HTTP authorization remain absent.
- LAN binding remains prohibited.

## Next Exact Action

Create LS-011 with a separate protected Axum router/policy layer that parses strict `Authorization: Bearer` headers, authenticates through `LocalStreamCore`, requires `library.read` for library/stream routes, maps missing/invalid/revoked credentials to the same safe `401`, and receives negative contract tests. Do not replace the current desktop loopback router or change binding yet.
