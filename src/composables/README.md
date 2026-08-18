# Frontend Composables

## Purpose

Own reusable Vue-native state and coordinate typed backend adapters without introducing a global store.

## Features

- `useAppInfo`: loads application metadata through an injectable adapter and exposes derived runtime state.
- `useMediaLibrary`: restores persisted library state and coordinates approved-folder selection and safe scan results.
- `useServerStatus`: loads the embedded HTTP server's safe address and exposure state.
- `usePlayback`: owns selected media, opaque-ID stream URL construction, and playback state.

## Important Files

- `useAppInfo.ts`: implementation and public `AppInfo` contract.
- `useAppInfo.test.ts`: loading, success, and failure tests.
- `useMediaLibrary.ts`: library scan state and public result contracts.
- `useMediaLibrary.test.ts`: success, cancellation, and failure tests.
- `useServerStatus.ts`: server status state and typed response contract.
- `useServerStatus.test.ts`: loopback success and failure tests.
- `usePlayback.ts`: Direct Play selection, URL, and event state.
- `usePlayback.test.ts`: URL safety, API availability, playback-event, and reset tests.

## Public Interfaces

Each composable returns refs, computed values, and explicit actions. Loaders are injectable for deterministic tests.

## Dependencies

Vue Composition API and the Tauri `invoke` adapter.

## Current Limitations

Browser preview cannot call native commands and intentionally shows a non-fatal preview state. The desktop adapter restores the current SQLite-backed library on startup.

## Planned Work

Create domain-specific composables for server lifecycle, nodes, compatibility, and settings only as those domains are implemented.
