# Tauri Application

## Purpose

Provide the Tauri 2 desktop shell, lifecycle, permissions, configuration, and thin adapters into the reusable Rust core.

## Features

The initial shell creates the main window and exposes the `app_info` command.

## Important Files

- `tauri.conf.json`: desktop shell and build configuration.
- `capabilities/default.json`: baseline main-window permissions.
- `icons/app-icon.svg`: editable source for generated platform icons.
- `src/lib.rs`: application builder and command registration.
- `src/main.rs`: desktop executable entry point.

## Public Interfaces

Tauri command `app_info` returns core-owned application metadata. Commands must remain thin adapters.

## Dependencies

Tauri 2 and the workspace-local `localstream-core` crate.

## Current Limitations

No folder dialog, persistence, HTTP server, media integration, mobile project, or packaging configuration exists.

## Planned Work

Add native capabilities only with explicit permissions and delegate behavior to the core.
