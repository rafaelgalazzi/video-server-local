# LocalStream Core Crate

## Purpose

Provide framework-independent LocalStream domain services reusable by Tauri, HTTP, headless, CLI, and future platform adapters.

## Features

The crate exposes application identity information and scans explicitly approved video-library directories.

## Important Files

- `src/lib.rs`: public core facade and application information model.
- `src/media/`: approved-directory scanner and safe media models.

## Public Interfaces

- `LocalStreamCore`: core service facade.
- `AppInfo`: serializable application identity value.
- `LocalStreamCore::scan_library`: approved-directory scan entry point.

## Dependencies

Serde for transport-neutral serialization, walkdir for controlled traversal, UUID for opaque IDs, and thiserror for typed errors. The crate intentionally does not depend on Tauri.

## Current Limitations

Persistence, compatibility inspection, audio, HTTP, discovery, pairing, streaming, and FFmpeg services are not implemented.

## Planned Work

Introduce domain modules only with their implementing tasks; avoid speculative generic abstractions.
