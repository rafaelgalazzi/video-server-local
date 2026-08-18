# Tauri Adapter Source

## Purpose

Contains desktop entry points and thin Tauri command adapters.

## Features and Interfaces

- `lib.rs`: initializes the database-backed core and embedded server, then registers thin status, library, approved-folder, pairing-decision, and trusted-peer administration adapters.
- `main.rs`: invokes the library runner.

## Dependencies

Tauri and `localstream-core`.

## Current Limitations and Planned Work

Pairing commands can list pending requests and approve or reject them locally; they cannot create requests or claim credentials. Peer commands expose safe active summaries and idempotent revocation. Remote encrypted transport does not exist.
