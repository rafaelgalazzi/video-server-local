# Streaming Domain

## Purpose

Resolve persisted opaque media identifiers and open approved files for bounded Direct Play delivery.

## Features

- Current-library opaque-ID lookup through the core-owned database.
- Canonical path containment validation immediately before file access.
- Asynchronous file opening and metadata reads without loading media into memory.
- Single HTTP byte-range parsing for bounded response streams.
- Content types for the scanner's supported video extensions.

## Important Files

- `mod.rs`: safe Direct Play source resolution and content type selection.
- `range.rs`: single byte-range validation and normalization.

## Public Interfaces

- `LocalStreamCore::open_direct_play`: opens a trusted streaming source by opaque ID.
- `DirectPlaySource`: asynchronous file, current size, and safe content type.
- `range::parse_single_range`: normalizes one requested range against the current file size.

## Dependencies

Tokio provides asynchronous filesystem I/O. Persisted location records remain private to the core.

## Limitations

Only single byte ranges are supported. Compatibility probing, multipart ranges, conditional requests, transcoding, concurrency limits, and authenticated LAN delivery are not implemented.

## Planned Work

Add client compatibility negotiation before introducing FFmpeg-based fallback.
