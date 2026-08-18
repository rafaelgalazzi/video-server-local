# Database Domain

## Purpose

Persist approved libraries and internal media-location records in embedded SQLite while returning path-free public models.

## Features

- Versioned schema migration using SQLite `user_version`.
- Atomic library snapshot replacement after a scan.
- Current-library restoration across core/application restarts.
- Internal absolute paths retained for trusted, containment-checked playback resolution.
- Bundled SQLite library; no external database process is required.

## Important Files

- `mod.rs`: connection ownership, migrations, persistence, restoration, and tests.

## Public Interfaces

Database details remain private behind `LocalStreamCore::scan_and_persist_library` and `LocalStreamCore::current_library`.

## Dependencies

Rusqlite with bundled SQLite.

## Current Limitations

The schema stores one current library selection and full-snapshot rescans. Incremental updates, backup/recovery UI, and migration tooling beyond schema version 1 are not implemented.

## Planned Work

Add incremental reconciliation while preserving opaque-ID playback lookups.
