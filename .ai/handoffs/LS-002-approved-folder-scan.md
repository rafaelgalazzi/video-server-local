# Handoff — LS-002

## Objective

Let a desktop user approve one folder, scan supported video candidates in the Rust core, and display safe metadata in Vue without exposing filesystem paths.

## Current State

Implementation and automated verification are complete. A fresh Windows Tauri window is running and responding. The interactive folder-selection/result-rendering check remains.

## Changed Files

- `crates/localstream-core/src/media/` scanner, models, documentation, and tests.
- Core and Tauri manifests plus `Cargo.lock` dependency resolution.
- `src-tauri/src/lib.rs` thin native picker/scan adapter.
- `src/composables/useMediaLibrary.ts` and tests.
- `src/components/MediaLibraryPanel.vue`, `src/App.vue`, and styles.
- Source, security, status, and test-matrix documentation.

## Important Decisions

- LS-002 supports MP4, MKV, WebM, MOV, and M4V candidates only.
- Extension matching does not claim playback compatibility.
- UUIDv5 IDs are derived internally from canonical locations and paths never cross the response boundary.
- Directory symlinks are not followed; inaccessible entries are counted and skipped.
- Results remain in memory; SQLite persistence is a later task.

## Completed

- Core scan and path-safe response contract.
- Native folder picker adapter.
- Vue scan lifecycle and media-list presentation.
- Frontend and Rust automated verification.
- Fresh Windows application launch.

## Remaining

- Click **Choose folder** in the running app.
- Select a folder containing a supported video.
- Confirm the title, extension, and size render and no raw path is displayed.
- Mark LS-002 completed and define the persistence task.

## Tests Executed

- `npm run verify` — PASS; 5 frontend tests.
- `cargo fmt --all --check` — PASS.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — PASS.
- `cargo test --workspace` — PASS; 4 core tests.
- `cargo check --workspace` — PASS.
- `npm run tauri dev -- --no-watch` — PASS; Windows window visible and responding.

## Tests Not Executed

- Interactive native folder-selection/result-rendering smoke check.
- Non-Windows platforms.

## Known Failures

- None.

## Assumptions

- Candidate discovery by extension is sufficient for this slice.
- Persistence, ffprobe, audio, incremental reconciliation, and playback remain out of scope.

## Next Exact Action

Use **Choose folder** in the running LocalStream window and verify a supported video row renders without a filesystem path.

## Do Not

- Expose or persist raw paths in frontend/public API models.
- Follow directory symlinks during scans.
- Claim file compatibility based only on extension.
- Start the persistence task before LS-002 verification is resolved or explicitly suspended.
