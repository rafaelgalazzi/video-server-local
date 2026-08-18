# LocalStream Core Crate

## Purpose

Provide framework-independent LocalStream domain services reusable by Tauri, HTTP, headless, CLI, and future platform adapters.

## Features

The initial crate exposes immutable application identity information to prove the adapter boundary.

## Important Files

- `src/lib.rs`: public core facade and application information model.

## Public Interfaces

- `LocalStreamCore`: core service facade.
- `AppInfo`: serializable application identity value.

## Dependencies

Serde for transport-neutral serialization. The crate intentionally does not depend on Tauri.

## Current Limitations

Media, storage, HTTP, discovery, pairing, streaming, and FFmpeg services are not implemented.

## Planned Work

Introduce domain modules only with their implementing tasks; avoid speculative generic abstractions.
