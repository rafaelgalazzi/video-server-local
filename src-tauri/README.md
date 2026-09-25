# Tauri Application

## Purpose

Provide the Tauri 2 desktop shell, lifecycle, permissions, configuration, and thin adapters into the reusable Rust core.

## Features

The shell creates the main window, initializes the database-backed core and protected node identity, always starts loopback HTTP, and optionally starts audited TLS-only LAN serving from explicit persisted configuration. LAN failures do not prevent trusted-local startup.

## Important Files

- `tauri.conf.json`: desktop shell and base build configuration.
- `capabilities/default.json`: baseline main-window permissions.
- `icons/app-icon.svg`: editable source for generated platform icons.
- `src/lib.rs`: application builder and command registration.
- `src/main.rs`: desktop executable entry point.

## Public Interfaces

Tauri commands `app_info`, `portable_runtime_status`, `server_info`, and `node_identity` return safe runtime metadata. `current_library` loads the safe persisted view, `clear_local_database` delegates confirmed local-data clearing, and track commands delegate validated preferences. `prepare_playback`, `playback_job`, `cancel_playback`, and `release_playback` are thin adapters to the reusable local playback coordinator.

## Dependencies

Tauri 2 plus the workspace-local `localstream-core` and `localstream-portable-payload` crates. The portable payload module is framework-independent; the Tauri adapter supplies its verified extracted root without making the core depend on Tauri.

## Current Limitations

The Windows portable profile is x64-only and requires Microsoft Edge WebView2 Runtime. It extracts roughly 211 MiB into application data on first launch, does not yet clean old payload versions, and still requires physical second-device trust/firewall qualification.

## Planned Work

Add native capabilities only with explicit permissions and delegate behavior to the core.
