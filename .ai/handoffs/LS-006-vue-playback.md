# LS-006 Handoff — Vue Direct Play interface

## Objective

Connect the indexed Vue library to LS-005's opaque-ID loopback stream route through an accessible native-controls player.

## State

Completed on 2026-08-18. The LS-006 files are uncommitted at handoff time; preserve them.

## Implemented

- `usePlayback` owns selected media, encoded stream URL construction, API readiness, and idle/loading/playing/error state.
- `PlaybackPanel.vue` renders the selected title, native video controls, loading/error feedback, autoplay request, and close action.
- Media rows expose Play buttons that remain disabled with visible feedback until `ServerInfo` is available.
- `App.vue` coordinates composables and clears stale playback on library selection or reconciliation.
- Presentation components do not perform networking, filesystem, database, or streaming business logic.

## Changed Files

- `src/composables/usePlayback.ts`
- `src/composables/usePlayback.test.ts`
- `src/components/PlaybackPanel.vue`
- `src/components/MediaLibraryPanel.vue`
- `src/App.vue`
- `src/styles.css`
- Root/frontend/component/composable README files and `.ai/PROJECT_STATUS.md`.

## Verified

- `npm run verify` — PASS; format, lint, typecheck, 12 tests across 4 files, and production build.
- `cargo fmt --all --check` — PASS.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — PASS.
- `cargo test --workspace` — PASS; 20 tests.
- `cargo check --workspace` — PASS.
- `git diff --check` — PASS before final documentation-only edits.

## Not Verified

- Interactive Windows Tauri playback and seeking with real media.
- Native webview support for each scanner-supported container/codec combination.
- Remote browser clients and non-Windows platforms.

## Design Notes

- The stream URL is `${ServerInfo.baseUrl}/api/v1/media/${encodeURIComponent(id)}/stream`; no title or path enters it.
- Browser media events update composable state through component emits.
- A library-selection attempt clears playback immediately, including when the picker is later cancelled, to prevent an old stream from remaining open during library mutation.
- Direct Play compatibility failure is expected and reported; FFmpeg fallback remains future work.

## Next Exact Action

Create LS-007 for pairing/authentication design and threat modeling. Document credential generation, approval, storage, revocation, route protection, and bind-transition rules before enabling any non-loopback listener.
