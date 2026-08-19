# Core Source

## Purpose

Contains reusable Rust domain code with no dependency on Tauri or HTTP transport frameworks.

## Features and Interfaces

`lib.rs` defines the core facade. `media/` owns scanning, `media_tools/` owns bounded FFmpeg process execution, `database/` persistence, `streaming/` Direct Play, `auth/` server credentials, `node_identity/` private PKI, `native_client/` protected peer pins, `lan/` endpoint/TLS activation policy, and `server/` Axum adapters and lifecycle.

## Dependencies

Axum, Tokio, rusqlite with bundled SQLite, Serde, walkdir, UUID, OS randomness, Base64, SHA-256, rcgen, keyring, and thiserror. Tests use tower, http-body-util, serde_json, and tempfile.

## Current Limitations and Planned Work

The standalone scanner identifies candidates by extension; desktop persisted scans add bounded ffprobe metadata. Compatibility decisions, track preferences, and media transforms remain planned. Add responsibility-focused modules as vertical product slices require them.
