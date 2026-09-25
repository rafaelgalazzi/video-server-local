# Media Tool Boundary

## Purpose

Discover and invoke FFmpeg tools without shell interpolation while keeping process lifetime and captured output bounded.

## Features

- Explicit absolute path construction for packaged distributions and environment configuration with `LOCALSTREAM_FFPROBE_PATH` / `LOCALSTREAM_FFMPEG_PATH`.
- Desktop adapters can supply bundled sidecar paths; those explicit distribution paths take precedence. Unpackaged development and headless launchers use `LOCALSTREAM_FFPROBE_PATH` / `LOCALSTREAM_FFMPEG_PATH`, then `PATH`.
- Executable identity validation through a bounded `-version` invocation.
- Structured `OsString` arguments, timeout, cooperative cancellation, kill-on-drop, and bounded stdout/stderr capture.

## Important Files

- `mod.rs`: discovery models, process runner, safe errors, and boundary tests.
- `probe.rs`: bounded ffprobe invocation and normalized path-free metadata parsing.

## Public Interfaces

- `MediaToolPaths::from_paths`: creates an absolute packaged tool-path pair.
- `MediaToolPaths::discover`: resolves and validates both required tools.
- `ProcessRunner::run`: executes one bounded structured process request.
- `ProcessRequest`: owns the executable, arguments, timeout, and output limits.

## Dependencies

Tokio owns asynchronous child processes and pipes. `tokio-util` provides cancellation tokens.

## Limitations

The Windows x64 portable packager embeds a pinned FFmpeg 9.0.2 essentials distribution and copies its exact notices; Linux, macOS, and headless packaging still require an explicit acquisition and licensing policy. Tool-version policy is identity-based for now; minimum supported versions will be set from compatibility evidence.

## Planned Work

`media_jobs/` owns transform queues, quotas, deduplication, cancellation, and stale-job cleanup. Remux and transcode modules build concrete structured FFmpeg work on both boundaries.
