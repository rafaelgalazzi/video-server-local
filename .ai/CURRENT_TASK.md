# Current Task

## ID

LS-006

## Title

Vue Direct Play interface

## Status

Completed

## Goal

Let a desktop user select an indexed video and play it through the embedded loopback Direct Play endpoint without exposing filesystem paths or moving networking logic into presentation components.

## Acceptance Criteria

- Indexed media rows expose an accessible Play action.
- A playback composable owns the selected item, stream URL, and idle/loading/playing/error state.
- Stream URLs use `ServerInfo.baseUrl`, the versioned route, and encoded opaque media IDs only.
- Playback is disabled with clear feedback while the embedded API is unavailable.
- A dedicated component renders native video controls plus selected-title, loading, error, and close states.
- Changing libraries clears stale playback state.
- Composable tests cover URL generation, unavailable-server behavior, event states, and reset behavior.
- Frontend and project documentation are current.

## Completed

- Added `usePlayback` for selected media, encoded versioned stream URLs, API availability, and playback event state.
- Added four deterministic composable tests covering URL generation, unavailable API behavior, playing/error transitions, and reset.
- Added an accessible native-controls `PlaybackPanel` with loading, compatibility error, and close states.
- Added API-aware Play buttons to indexed media rows and clear disabled feedback before the private API is ready.
- Integrated playback into `App.vue` and clear stale playback whenever library selection begins or the selected item leaves the restored library.
- Updated frontend, project, and root documentation.

## Tests Last Executed

- `npm run verify` — PASS; format, lint, typecheck, 4 files / 12 frontend tests, and production build passed.
- `cargo fmt --all --check` — PASS.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — PASS.
- `cargo test --workspace` — PASS; 20 core/workspace tests passed.
- `cargo check --workspace` — PASS.
- `git diff --check` — PASS before final task/handoff documentation update.

## Tests Not Yet Executed

- Interactive playback, seeking, close, and library-change behavior in a running Windows Tauri window with a supported real media file.
- Browser/container compatibility across MP4, MKV, WebM, MOV, and M4V.
- Non-Windows platforms and remote browser clients.

## Known Problems

- None confirmed for the LS-006 automated scope.

## Assumptions

- Native `<video>` controls are the correct first interface and browser codec support determines whether Direct Play succeeds.
- The user-initiated Play action requests autoplay; native controls remain available if autoplay is declined.
- Playback stays loopback-only and desktop-local until pairing/authentication permits LAN exposure.

## Next Exact Step

Define LS-007 for pairing/authentication and its threat model before changing the server bind address or hosting the UI for LAN clients.
