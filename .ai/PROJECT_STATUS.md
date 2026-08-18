# LocalStream Project Status

## Last Updated

2026-08-18

## Current Milestone

Milestone 2 foundation: embedded versioned HTTP API.

## Working

- Two detailed planning documents describe product direction and quality expectations.
- Repository-local governance, continuation, architecture, security, API, and development documentation exists.
- Vue 3 + TypeScript + Vite frontend scaffold and responsive foundation screen.
- Composition API backend state with success/failure unit coverage.
- Tauri 2 shell configured with a thin `app_info` command.
- Framework-independent `localstream-core` Rust crate with a unit test.
- Frontend format, lint, typecheck, test, build, and combined verification scripts.
- Approved-folder Rust scanner for supported video candidates with opaque IDs and no returned paths.
- Native folder-picker adapter and Vue media-library states.
- Embedded SQLite schema, atomic library snapshot persistence, and startup restoration.
- Migration, restart restoration, and new/deleted media reconciliation tests.
- Embedded Axum server with graceful lifecycle, versioned health/library routes, and safe JSON errors.
- Vue displays the actual loopback API address and exposure status.

## In Progress

- Nothing. LS-004 is complete; LS-002's interactive UI check remains unverified and documented.

## Not Started

- Pairing/authentication and safe LAN binding.
- Web UI hosting and HTTP Range streaming.
- Direct Play and playback UI.
- Node discovery, pairing, trust, and distributed libraries.
- FFmpeg probing/transcoding and concurrency management.
- Automated tests, CI, packaging, and platform verification.

## Known Major Limitations

- Release bundling and installer behavior is unknown / not verified.
- The current local Node.js 22.12 environment is below one transitive lint dependency's declared minimum of 22.13, although verification executed successfully.
- Scans are extension-based; compatibility metadata is not inspected.
- Rescans replace the full stored snapshot rather than updating incrementally.
- HTTP is loopback-only on an ephemeral port until pairing/authentication is implemented.
- The repository has no commits; all current files are untracked at the time of this inspection.

## Next Major Goal

Define LS-005 for authentication/pairing foundations before LAN exposure, or a loopback-only Direct Play/HTTP Range slice.
