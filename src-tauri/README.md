# Tauri Application

## Purpose

Provide the Tauri 2 desktop shell, lifecycle, permissions, configuration, and thin adapters into the reusable Rust core.

## Features

The shell creates the main window, exposes `app_info`, and provides an approved-folder picker that delegates scanning to the core.

## Important Files

- `tauri.conf.json`: desktop shell and build configuration.
- `capabilities/default.json`: baseline main-window permissions.
- `icons/app-icon.svg`: editable source for generated platform icons.
- `src/lib.rs`: application builder and command registration.
- `src/main.rs`: desktop executable entry point.

## Public Interfaces

Tauri command `app_info` returns core-owned application metadata. `select_and_scan_library` performs native folder selection and delegates the scan to `LocalStreamCore`. Commands must remain thin adapters.

## Dependencies

Tauri 2 and the workspace-local `localstream-core` crate.

## Current Limitations

No persistence, HTTP server, playback integration, mobile project, or release packaging configuration exists.

## Planned Work

Add native capabilities only with explicit permissions and delegate behavior to the core.
