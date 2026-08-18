# Core Source

## Purpose

Contains reusable Rust domain code with no dependency on Tauri or HTTP transport frameworks.

## Features and Interfaces

`lib.rs` defines the core facade. `media/` owns approved-folder scanning and safe result models. `database/` owns SQLite schema, persistence, and restoration. `server/` owns Axum adapters and lifecycle.

## Dependencies

Axum, Tokio, rusqlite with bundled SQLite, Serde, walkdir, UUID, and thiserror. Tests use tower, http-body-util, serde_json, and tempfile.

## Current Limitations and Planned Work

Scanning currently identifies video candidates by extension only and does not persist results. Add responsibility-focused modules as vertical product slices require them.
