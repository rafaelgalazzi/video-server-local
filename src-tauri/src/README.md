# Tauri Adapter Source

## Purpose

Contains desktop entry points and thin Tauri command adapters.

## Features and Interfaces

- `lib.rs`: registers application-info and approved-folder scan adapters and constructs managed core state.
- `main.rs`: invokes the library runner.

## Dependencies

Tauri and `localstream-core`.

## Current Limitations and Planned Work

Only the architecture-boundary command exists. New commands must delegate to core services and receive adapter-focused tests where practical.
