# Media Domain

## Purpose

Discover media inside directories explicitly approved by the user while keeping filesystem locations private to the Rust core.

## Features

- Recursive candidate discovery for MP4, MKV, WebM, MOV, and M4V video files.
- Stable opaque UUID identifiers derived internally from canonical locations.
- Bounded metadata reads; files are never loaded into memory.
- Per-entry traversal failures are counted without aborting the entire scan.
- Directory symlinks are not followed.
- Optional bounded ffprobe metadata for container, duration, video, audio, and subtitle streams.
- Opaque track identifiers and safe per-item `available`, `not_probed`, or `unavailable` status.
- Corrupt or inaccessible media probe failures remain isolated to one item.

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

The standalone synchronous scanner identifies candidates only; the core's async persisted scan adds ffprobe metadata. Compatibility decisions, track preferences, incremental scanning, remuxing, and transcoding are not implemented.

## Planned Work

Add persisted audio/subtitle preferences and compatibility decisions in LS-069, LS-070, and LS-045.
