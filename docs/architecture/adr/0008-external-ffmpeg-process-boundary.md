# ADR-0008 — External FFmpeg process boundary

## Status

Accepted

## Context

MKV inspection and browser-compatible fallback require ffprobe and FFmpeg. Linking libav would add a large unsafe FFI and platform-packaging surface before compatibility behavior is measured. Unbounded or shell-based invocation would violate project security and resource invariants.

## Decision

Invoke external `ffprobe` and `ffmpeg` through one reusable Rust process boundary. Arguments are structured operating-system strings and never interpolated into a shell command. Every invocation sets a timeout, cancellation token, and captured-output limit; children are killed when those bounds are reached or the owner is dropped.

Development builds resolve tool names from `PATH`. Deployments may set absolute paths through `LOCALSTREAM_FFPROBE_PATH` and `LOCALSTREAM_FFMPEG_PATH`. Both executables are identity-checked with a bounded `-version` call. ADR-0010 now selects pinned FFmpeg/ffprobe payload files and their license notices for the Windows portable release; other distributions must define their own acquisition and licensing policy before shipping.

## Alternatives Considered

- Link libav through FFI.
- Construct shell command strings.
- Download executables automatically at runtime.
- Require fixed global executable paths.

## Consequences

Later probing and transform services reuse one testable, platform-isolated boundary without Internet dependence or unsafe media bindings. Packaged releases must provide compatible binaries and license material or require explicit administrator installation. Minimum supported versions remain to be selected from Phase A.1 evidence; the provisional Windows portable payload policy is recorded in ADR-0010.
