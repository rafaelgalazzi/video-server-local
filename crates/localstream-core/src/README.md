# Core Source

## Purpose

Contains reusable Rust domain code with no dependency on Tauri or HTTP transport frameworks.

## Features and Interfaces

`lib.rs` currently defines `LocalStreamCore::app_info` and its `AppInfo` return type, with a baseline unit test.

## Dependencies

Serde derives only.

## Current Limitations and Planned Work

This is a boundary proof, not the media core. Add responsibility-focused modules as vertical product slices require them.
