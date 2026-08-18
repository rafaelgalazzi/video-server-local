# Handoff — LS-003

## Objective

Persist the current approved media library in embedded SQLite and restore its safe view after restart.

## Current State

Completed and covered by automated frontend, database, reconciliation, and restart-style core tests.

## Changed Files

- `crates/localstream-core/src/database/` SQLite schema, storage, restoration, and migration tests.
- Core manifest/facade/media internals and `Cargo.lock`.
- Tauri core initialization plus current-library adapter.
- Vue media-library restoration state and test.
- Status, security, source, and test documentation.

## Important Decisions

- Rusqlite uses bundled SQLite, requiring no external server.
- Database location comes from Tauri's platform application-data directory.
- Internal paths are stored in SQLite but never returned through public models.
- Rescans atomically replace one library's complete media snapshot.
- Schema version 1 uses SQLite `user_version`; unknown future versions are rejected.

## Completed

- Atomic persistence and current-library selection.
- Startup restoration adapter and Vue loading state.
- Deleted/new file reconciliation.
- Schema migration and downgrade protection.

## Remaining

- None for LS-003.

## Tests Executed

- `npm run verify` — PASS; 6 frontend tests.
- `cargo fmt --all --check` — PASS.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — PASS.
- `cargo test --workspace` — PASS; 8 tests.
- `cargo check --workspace` — PASS.

## Tests Not Executed

- Real user-library restart smoke test.
- Non-Windows platforms.

## Known Failures

- None.

## Assumptions

- Full-snapshot reconciliation is acceptable until incremental scanning is justified.
- One current library is sufficient for the present UI.

## Next Exact Action

Define LS-004 for an embedded Axum server and safe `/api/v1/` library endpoint.

## Do Not

- Return database paths to Vue or future HTTP clients.
- Replace bundled SQLite without an explicit architecture decision.
- Modify a released schema without adding a forward migration.
