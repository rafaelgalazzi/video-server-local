# Media Tool Boundary

## Purpose

Discover and invoke FFmpeg tools without shell interpolation while keeping process lifetime and captured output bounded.

## Features

- Explicit path configuration with `LOCALSTREAM_FFPROBE_PATH` and `LOCALSTREAM_FFMPEG_PATH`.
- Development fallback to `ffprobe` and `ffmpeg` on `PATH`.
- Executable identity validation through a bounded `-version` invocation.
- Structured `OsString` arguments, timeout, cooperative cancellation, kill-on-drop, and bounded stdout/stderr capture.

## Important Files

- `mod.rs`: discovery models, process runner, safe errors, and boundary tests.

## Public Interfaces

- `MediaToolPaths::discover`: resolves and validates both required tools.
- `ProcessRunner::run`: executes one bounded structured process request.
- `ProcessRequest`: owns the executable, arguments, timeout, and output limits.

## Dependencies

Tokio owns asynchronous child processes and pipes. `tokio-util` provides cancellation tokens.

## Limitations

Release packaging does not yet bundle FFmpeg. Operators must install compatible tools on `PATH` or configure absolute paths. Tool-version policy is identity-based for now; minimum supported versions will be set from compatibility evidence.

## Planned Work

LS-044 will consume this boundary for normalized ffprobe metadata. LS-046 will add transform queues, quotas, deduplication, and stale-job cleanup.
