# Core Source

## Purpose

Contains reusable Rust domain code with no dependency on Tauri or HTTP transport frameworks.

## Features and Interfaces

`lib.rs` defines the core facade. `media/` owns scanning, `media_tools/` bounded FFmpeg execution, `media_jobs/` transform lifecycle, `remux/` stream-copy, `transcode/` software encoding, `playback/` Direct Play-first coordination, `compatibility/` decisions, `database/` persistence, `streaming/` Direct Play, `auth/` credentials, `node_identity/` PKI, `native_client/` pins, `lan/` activation policy, and `server/` Axum adapters.

## Dependencies

Axum, Tokio, rusqlite with bundled SQLite, Serde, walkdir, UUID, OS randomness, Base64, SHA-256, rcgen, keyring, and thiserror. Tests use tower, http-body-util, serde_json, and tempfile.

## Current Limitations and Planned Work

The standalone scanner identifies candidates by extension; desktop persisted scans add bounded ffprobe metadata. Physical browser/device fallback compatibility remains unverified. Add responsibility-focused modules as vertical product slices require them.
