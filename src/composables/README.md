# Frontend Composables

## Purpose

Own reusable Vue-native state and coordinate typed backend adapters without introducing a global store.

## Features

- `useAppInfo`: loads application metadata through an injectable adapter and exposes derived runtime state.
- `useMediaLibrary`: coordinates approved-folder selection and safe scan-result state.

## Important Files

- `useAppInfo.ts`: implementation and public `AppInfo` contract.
- `useAppInfo.test.ts`: loading, success, and failure tests.
- `useMediaLibrary.ts`: library scan state and public result contracts.
- `useMediaLibrary.test.ts`: success, cancellation, and failure tests.

## Public Interfaces

Each composable returns refs, computed values, and explicit actions. Loaders are injectable for deterministic tests.

## Dependencies

Vue Composition API and the Tauri `invoke` adapter.

## Current Limitations

Browser preview cannot call native commands and intentionally shows a non-fatal preview state. Library results are session-only until SQLite persistence is implemented.

## Planned Work

Create domain-specific composables for libraries, server lifecycle, nodes, playback, and settings only as those domains are implemented.
