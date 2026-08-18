# Core Source

## Purpose

Contains reusable Rust domain code with no dependency on Tauri or HTTP transport frameworks.

## Features and Interfaces

`lib.rs` defines the core facade. `media/` owns approved-folder scanning and safe result models. `database/` owns SQLite schema, persistence, and restoration. `streaming/` owns safe Direct Play sources. `auth/` owns peer credential lifecycle. `server/` owns Axum adapters and lifecycle.

## Dependencies

Axum, Tokio, rusqlite with bundled SQLite, Serde, walkdir, UUID, OS randomness, Base64, SHA-256, and thiserror. Tests use tower, http-body-util, serde_json, and tempfile.

## Current Limitations and Planned Work

Scanning identifies candidates by extension, and pairing has no remote protocol or approval flow. Add responsibility-focused modules as vertical product slices require them.
