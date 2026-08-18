# Core Source

## Purpose

Contains reusable Rust domain code with no dependency on Tauri or HTTP transport frameworks.

## Features and Interfaces

`lib.rs` defines the core facade. `media/` owns approved-folder scanning and safe result models.

## Dependencies

Serde, walkdir, UUID, and thiserror. Tests use tempfile.

## Current Limitations and Planned Work

Scanning currently identifies video candidates by extension only and does not persist results. Add responsibility-focused modules as vertical product slices require them.
