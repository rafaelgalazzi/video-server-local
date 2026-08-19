# LocalStream Core Crate

## Purpose

Provide framework-independent LocalStream domain services reusable by Tauri, HTTP, headless, CLI, and future platform adapters.

## Features

The crate exposes application identity information, scans explicitly approved video-library directories, persists the current library in SQLite, opens containment-checked Direct Play sources, manages revocable peer credentials, owns a protected node-root identity boundary, and hosts thin versioned Axum HTTP adapters.

## Important Files

- `src/lib.rs`: public core facade and application information model.
- `src/media/`: approved-directory scanner and safe media models.
- `src/database/`: SQLite schema, snapshot persistence, and restoration.
- `src/server/`: Axum router, loopback lifecycle, and API contracts.
- `src/streaming/`: opaque-ID resolution, containment checks, and byte ranges.
- `src/auth/`: secure peer credentials and bounded user-approved pairing lifecycle.
- `src/node_identity/`: persistent private-CA identity and protected secret-store boundary.
- `src/media_tools/`: safe FFmpeg/ffprobe discovery and bounded process execution.
- `src/media_jobs/`: bounded transform admission, lifecycle, quota, and temporary storage.
- `src/remux/`: selected-track FFmpeg stream-copy fallback into browser containers.
- `src/transcode/`: software-only browser fallback encoding and subtitle delivery.
- `src/playback/`: Direct Play-first preparation and fallback lifecycle coordination.
- `src/compatibility/`: Direct Play-first capability and fallback decisions.

## Public Interfaces

- `LocalStreamCore`: core service facade.
- `AppInfo`: serializable application identity value.
- `LocalStreamCore::scan_library`: approved-directory scan entry point.
- `LocalStreamCore::scan_and_persist_library`: atomic scan-and-store operation.
- `LocalStreamCore::current_library`: safe persisted-library view.
- `server::start_local_server`: embedded loopback HTTP lifecycle.
- `LocalStreamCore::open_direct_play`: bounded Direct Play source resolution.
- `LocalStreamCore` peer credential issuance, authentication, and revocation methods.
- `node_identity::NodeIdentityService`: fail-closed root identity generation/restoration.
- `node_identity::NodeIdentitySummary`: safe stable node ID and SPKI fingerprint.

## Dependencies

Axum and Tokio for the embedded server, bundled SQLite through rusqlite, Serde for transport-neutral serialization, walkdir for controlled traversal, UUID for opaque IDs, `rcgen` for X.509 identity material, `keyring` for platform protected storage, and thiserror for typed errors. The crate intentionally does not depend on Tauri.

## Current Limitations

Discovery and headless protected storage are not implemented. Direct Play and completed fallbacks support single byte ranges; completed transform reservations require explicit release. Rescans replace a complete library snapshot, and release packaging does not yet bundle FFmpeg tools.

## Planned Work

Introduce domain modules only with their implementing tasks; avoid speculative generic abstractions.
