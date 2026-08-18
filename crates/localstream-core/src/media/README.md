# Media Domain

## Purpose

Discover media inside directories explicitly approved by the user while keeping filesystem locations private to the Rust core.

## Features

- Recursive candidate discovery for MP4, MKV, WebM, MOV, and M4V video files.
- Stable opaque UUID identifiers derived internally from canonical locations.
- Bounded metadata reads; files are never loaded into memory.
- Per-entry traversal failures are counted without aborting the entire scan.
- Directory symlinks are not followed.

## Important Files

- `mod.rs`: scanner models, validation, traversal, and unit tests.

## Public Interfaces

- `scan_approved_directory`: scans one already-approved directory.
- `LibraryScan`: safe result containing display metadata and issue counts.
- `MediaItem`: safe media summary without any filesystem path.
- `LibraryScanError`: root-level validation and access failures.

## Dependencies

`walkdir` for controlled traversal, `uuid` for opaque identifiers, `thiserror` for typed errors, and `serde` for adapter serialization.

## Current Limitations

Extension matching only identifies candidates. Compatibility, codecs, duration, dimensions, persistence, deletion reconciliation, incremental scanning, audio, and ffprobe metadata are not implemented.

## Planned Work

Persist internal path-to-ID records in SQLite before exposing media through HTTP or playback routes.
