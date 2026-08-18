# LS-005 Handoff — Loopback Direct Play with HTTP Range

## Objective

Deliver containment-checked, bounded Direct Play from the embedded loopback API using persisted opaque media IDs.

## State

Completed on 2026-08-18. The repository contains uncommitted LS-005 changes; do not discard them.

## Implemented

- `LocalStreamCore::open_direct_play` resolves current-library IDs through private SQLite records.
- The streaming domain canonicalizes both root and file, validates containment, opens asynchronously, and maps supported video content types.
- `GET /api/v1/media/{id}/stream` streams full files or one normalized byte range without buffering the complete file.
- Stable safe errors cover missing IDs, invalid ranges, internal/unavailable files, and exhausted stream capacity.
- Eight core-owned permits bound concurrent Direct Play sources and remain held for each response body's lifetime.
- The server remains loopback-only.

## Changed Files

- `crates/localstream-core/src/streaming/`
- `crates/localstream-core/src/database/mod.rs`
- `crates/localstream-core/src/lib.rs`
- `crates/localstream-core/src/server/mod.rs`
- `crates/localstream-core/Cargo.toml` and `Cargo.lock`
- Canonical API, security, architecture, test-matrix, project-status, and source README documentation.

## Verified

- `npm run verify` — PASS; 8 frontend tests and production build.
- `cargo fmt --all --check` — PASS.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — PASS.
- `cargo test --workspace` — PASS; 20 tests.
- `cargo check --workspace` — PASS.
- `git diff --check` — PASS before final task documentation update.

## Not Verified

- Real browser playback and seeking with a production media file.
- Large-file/long-running behavior and non-Windows platforms.
- Features explicitly outside scope: LAN access, authentication, multipart ranges, caching validators, compatibility probing, and transcoding.

## Design Notes

- Single ranges are deliberate for the first Direct Play slice.
- The permit is part of `DirectPlaySource`, so it is released when the streamed body/source drops.
- Unknown IDs return `404`; invalid ranges return `416` with the required unsatisfied `Content-Range`; unavailable or containment-rejected records return generic `500` without paths.

## Next Exact Action

Create LS-006 with acceptance criteria for selecting a library item and playing its opaque-ID URL using `ServerInfo.base_url`, while preserving clear loading, error, and unsupported-format states. If LAN access is prioritized instead, design pairing/authentication before changing the bind address.
