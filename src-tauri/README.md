# Tauri Application

## Purpose

Provide the Tauri 2 desktop shell, lifecycle, permissions, configuration, and thin adapters into the reusable Rust core.

## Features

The shell creates the main window, initializes the database-backed core and protected node identity, always starts loopback HTTP, and optionally starts audited TLS-only LAN serving from explicit persisted configuration. LAN failures do not prevent trusted-local startup.

## Important Files

- `tauri.conf.json`: desktop shell and build configuration.
- `capabilities/default.json`: baseline main-window permissions.
- `icons/app-icon.svg`: editable source for generated platform icons.
- `src/lib.rs`: application builder and command registration.
- `src/main.rs`: desktop executable entry point.

## Public Interfaces

Tauri commands `app_info`, `server_info`, and `node_identity` return safe runtime metadata. `current_library` loads the safe persisted view; `select_audio_track` and `select_subtitle` delegate validated preference persistence to the core. Commands must remain thin adapters.

## Dependencies

Tauri 2 and the workspace-local `localstream-core` crate.

## Current Limitations

Release installer behavior, firewall UX, mobile projects, and physical second-device trust onboarding remain unverified.

## Planned Work

Add native capabilities only with explicit permissions and delegate behavior to the core.
