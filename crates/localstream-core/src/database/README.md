# Database Domain

## Purpose

Persist approved libraries and internal media-location records in embedded SQLite while returning path-free public models.

## Features

- Versioned schema migration using SQLite `user_version`.
- Atomic library snapshot replacement after a scan.
- Current-library restoration across core/application restarts.
- Internal absolute paths retained for trusted, containment-checked playback resolution.
- Schema-v3 trusted peer metadata plus digest-only browser sessions bound to peer capability, expiry, and revocation.
- Bulk peer revocation used before destructive node-identity reset.
- Active-peer listing that excludes token digests and revoked records from public core models.
- Bundled SQLite library; no external database process is required.
- Schema-v4 normalized media metadata JSON plus private opaque-track-to-source-index mappings.
- Schema-v5 per-media audio preferences retained only while the fingerprinted track remains valid.
- Schema-v6 subtitle preferences with distinct Automatic, Off, and validated-track semantics.
- Transactional local-data clearing removes library records, track preferences, sessions, and peers while preserving the migrated schema.

## Important Files

- `mod.rs`: connection ownership, migrations, persistence, restoration, and tests.

## Public Interfaces

Database details remain private behind `LocalStreamCore` operations, including scan, restoration, and explicit local-database clearing.

## Dependencies

Rusqlite with bundled SQLite.

## Current Limitations

The schema stores one current library selection and full-snapshot rescans. Clearing requires an available database connection; recovery from a database that cannot be opened and backup restoration are not implemented.

## Planned Work

Add incremental reconciliation while preserving opaque-ID playback lookups.
