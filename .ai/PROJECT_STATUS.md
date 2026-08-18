# LocalStream Project Status

## Last Updated

2026-08-18

## Current Milestone

Milestone 2 foundation: embedded HTTP API and loopback Direct Play.

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
- Loopback-only opaque-ID Direct Play with containment validation, HTTP single-range responses, bounded asynchronous I/O, and an eight-stream concurrency limit.
- Vue displays the actual loopback API address and exposure status.
- Vue media rows can open an accessible native-controls player using the versioned opaque-ID stream URL, with loading and compatibility-error states.
- Core peer credentials use 256-bit OS randomness, digest-only SQLite persistence, explicit `library.read` capability, restart-safe authentication, and revocation.
- ADR-0006 and the security threat model define pairing, encrypted transport, authorization, rate-limit, and revocation gates that must precede LAN binding.
- Bounded two-minute pairing requests use cryptographic IDs/claim secrets, human verification codes, explicit local decisions, and single-use replay protection.
- Thin Tauri commands can list, approve, or reject pending pairing requests without exposing creation or credential claiming.
- Vue polls pending pairing requests only after native availability and presents accessible code comparison, expiry, Allow/Reject, and retry states.
- Active trusted peers can be listed through path/token-free metadata and revoked persistently through a confirmation-based local UI.
- A dormant authenticated Axum router strictly validates bearer credentials, enforces `library.read`, inserts safe peer identity, and returns uniform unauthorized responses.

## In Progress

- Nothing. LS-011 is complete.

## Not Started

- Encrypted pairing/session routes, client secure storage, rate limiting, and safe LAN binding.
- Web UI hosting for remote browser clients.
- Node discovery, pairing, trust, and distributed libraries.
- FFmpeg probing/transcoding and concurrency management.
- Automated tests, CI, packaging, and platform verification.

## Known Major Limitations

- Release bundling and installer behavior is unknown / not verified.
- The current local Node.js 22.12 environment is below one transitive lint dependency's declared minimum of 22.13, although verification executed successfully.
- Scans are extension-based; compatibility metadata is not inspected.
- Rescans replace the full stored snapshot rather than updating incrementally.
- HTTP and Direct Play are loopback-only on an ephemeral port until pairing/authentication is implemented.
- Direct Play supports one byte range per request; multipart ranges and conditional caching are not implemented.
- Playback compatibility is delegated to the embedded browser; ffprobe metadata and transcoding fallback are not implemented.
- Peer credential mechanics are core-only; there are no pairing endpoints, approval UI, client secret storage, or authenticated LAN routes.
- Pairing requests are intentionally memory-only and disappear on restart; network rate limiting is not implemented because no remote pairing route exists.
- The approval UI has no real incoming requests until a later encrypted remote pairing transport exists.
- The authenticated router is not attached to a listener; native browser media credential transport is deferred in DD-007.
- The repository has no commits; all current files are untracked at the time of this inspection.

## Next Major Goal

Define LS-012 for authenticated encrypted LAN server identity and transport design before implementing remote routes.
