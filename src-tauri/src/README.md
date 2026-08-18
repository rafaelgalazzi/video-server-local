# Tauri Adapter Source

## Purpose

Contains desktop entry points and thin Tauri command adapters.

## Features and Interfaces

- `lib.rs`: initializes the database-backed core, protected node identity, and embedded server, then registers thin trusted-local adapters.
- `main.rs`: invokes the library runner.

## Dependencies

Tauri and `localstream-core`.

## Current Limitations and Planned Work

Node-identity commands expose only a cloned public summary and a restart-required reset result. Root export uses a native save dialog and writes public DER directly without returning certificate bytes or a path to Vue. Automatic trust installation and remote certificate download do not exist.
