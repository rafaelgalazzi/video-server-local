# Media Job Manager

## Purpose

Own bounded, transport-neutral execution of resource-heavy media transforms.

## Features

- Fixed worker concurrency and bounded queued admission.
- Active-job deduplication using caller-defined opaque keys.
- Cooperative cancellation propagated to the FFmpeg process boundary.
- Admission-time temporary-byte reservations and post-job quota enforcement.
- Per-job temporary directories under a fixed LocalStream-owned namespace, explicit release, failure cleanup, and startup stale cleanup.
- Path-free job snapshots with bounded permille progress and stable failure categories.

## Important Files

- `mod.rs`: public models, queue lifecycle, quota accounting, cleanup, and tests.

## Public Interfaces

- `MediaJobManager::start`: validates configuration, creates the private work root, and removes stale entries.
- `MediaJobManager::submit`: admits or deduplicates a transform closure.
- `MediaJobManager::snapshot`, `cancel`, `open_output`, and `release`: observe, consume, and control a job without exposing paths.
- `MediaJobContext`: gives transform code its private directory, cancellation token, and progress reporter.

## Dependencies

Tokio provides bounded channels, workers, synchronization, cancellation, and filesystem operations. UUIDs are opaque job identifiers.

## Limitations and Planned Work

Reservations are conservative caller estimates and completed outputs retain their reservation until released. Remux/transcode jobs currently report lifecycle progress rather than parsed FFmpeg frame/time progress.
