# Current Task

## ID

LS-002

## Title

Approved-folder video library scan

## Status

Verification

## Goal

Let a desktop user explicitly select a local folder, scan supported video files through the reusable Rust core, and display safe media metadata in Vue without exposing raw paths.

## Acceptance Criteria

- Folder approval happens through a native desktop picker.
- Only the selected directory is scanned; directory symlinks are not followed.
- The reusable Rust core recursively discovers supported video extensions and tolerates inaccessible entries.
- Results expose opaque media IDs, titles, extensions, and sizes, never raw paths.
- Vue uses a tested Composition API composable and renders loading, cancellation, empty, error, and success states.
- Rust and frontend quality gates pass and the flow is smoke-tested on Windows.
- Relevant source and governance documentation is current.

## Relevant Files

- `src/`
- `src-tauri/`
- `crates/localstream-core/src/media/`
- `docs/security/README.md`
- `docs/development/TEST_MATRIX.md`

## Completed

- LS-001 established and verified the Vue/Tauri/Rust foundation.
- Reviewed media scanner, identity, and filesystem exposure requirements.
- Implemented recursive approved-folder scanning for MP4, MKV, WebM, MOV, and M4V candidates.
- Added stable opaque UUID media IDs and path-free result models.
- Added a thin native folder-picker Tauri command and registered the dialog plugin.
- Added Vue selection, loading, cancellation, error, empty, skipped-entry, and result states.
- Added three frontend and three scanner tests.
- Passed all automated frontend and Rust quality gates.
- Launched a fresh Windows Tauri build; the LocalStream window is visible and responding.

## In Progress

- Interactive folder-picker and rendered-result smoke check.

## Remaining

- Smoke-test selection with a Windows folder.
- Confirm the selected folder's supported videos render without a displayed raw path.

## Tests Last Executed

- `npm run verify` — PASS; 2 files and 5 frontend tests passed.
- `cargo fmt --all --check` — PASS.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — PASS.
- `cargo test --workspace` — PASS; 4 core tests passed in the workspace.
- `cargo check --workspace` — PASS.
- Fresh `npm run tauri dev -- --no-watch` launch — PASS; Windows window visible and responding.

## Tests Not Yet Executed

- Interactive native folder selection and rendered media-list check.
- Linux, macOS, Android, and iOS verification.

## Known Problems

- No implementation failure is known. Interactive folder selection is not yet verified.

## Assumptions

- LS-002 scans video extensions only: MP4, MKV, WebM, MOV, and M4V.
- Extension matching identifies candidates, not confirmed playback compatibility.
- IDs are stable opaque UUIDv5 values derived internally from the canonical file location; paths never cross the adapter response.
- Persistence, deletion reconciliation, incremental scans, audio, ffprobe metadata, and playback are later tasks.

## Next Exact Step

In the running LocalStream window, choose a folder containing a supported video and confirm the safe media row renders; then complete LS-002.
