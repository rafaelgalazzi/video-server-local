# Tauri Adapter Source

## Purpose

Contains desktop entry points and thin Tauri command adapters.

## Features and Interfaces

- `lib.rs`: activates the verified portable payload, initializes packaged media-tool paths, the database-backed core, protected node identity, and embedded server, then registers thin trusted-local adapters.
- `portable.rs`: resolves the current executable and delegates payload activation to the reusable payload crate.
- `main.rs`: invokes the library runner.

## Dependencies

Tauri and `localstream-core`.

## Current Limitations and Planned Work

Node-identity commands expose only a cloned public summary and a restart-required reset result. Portable activation failure is converted to a safe status, disables packaged media/LAN assets without consulting machine executables, and keeps trusted-local startup available. Root export uses a native save dialog and writes public DER directly without returning certificate bytes or a path to Vue. Portable media files are resolved only from the verified extracted payload; automatic trust installation, firewall changes, and remote certificate download do not exist.
