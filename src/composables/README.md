# Frontend Composables

## Purpose

Own reusable Vue-native state and coordinate typed backend adapters without introducing a global store.

## Features

- `useAppInfo`: loads application metadata through an injectable adapter and exposes derived runtime state.
- `useMediaLibrary`: restores persisted library state and coordinates approved-folder selection and safe scan results.
- `useServerStatus`: loads the embedded HTTP server's safe address and exposure state.
- `useNodeIdentity`: loads the trusted-local public identity and coordinates root export and confirmation-based reset.
- `usePlayback`: owns selected media, opaque-ID stream URL construction, playback state, accessible audio choices, and native preference persistence.
- `usePairingRequests`: polls trusted-local pending requests and coordinates approve/reject decisions.
- `useTrustedPeers`: loads safe peer summaries and owns confirmation-based revocation state.
- `useRuntimeBootstrap`: selects native Tauri or same-origin browser transport and exposes explicit connection states.
- `useTrustOnboarding`: gates root export on fingerprint-comparison acknowledgement.
- `useLanServer`: manages disabled-by-default explicit LAN configuration and status.

## Important Files

- `useAppInfo.ts`: implementation and public `AppInfo` contract.
- `useAppInfo.test.ts`: loading, success, and failure tests.
- `useMediaLibrary.ts`: library scan state and public result contracts.
- `useMediaLibrary.test.ts`: success, cancellation, and failure tests.
- `useServerStatus.ts`: server status state and typed response contract.
- `useServerStatus.test.ts`: loopback success and failure tests.
- `useNodeIdentity.ts` and its test: safe node-summary loading and failure containment.
- `usePlayback.ts`: Direct Play selection, URL, and event state.
- `usePlayback.test.ts`: URL safety, API availability, playback-event, and reset tests.
- `usePairingRequests.ts`: pending request state, decision orchestration, and polling lifecycle.
- `usePairingRequests.test.ts`: load, decision, failure retention, polling, and cleanup tests.
- `useTrustedPeers.ts`: active-peer load, confirmation, cancellation, and revocation orchestration.
- `useTrustedPeers.test.ts`: safe load, confirmation, success, cancellation, and failure-retention tests.

## Public Interfaces

Each composable returns refs, computed values, and explicit actions. Loaders are injectable for deterministic tests.

## Dependencies

Vue Composition API and the Tauri `invoke` adapter.

## Current Limitations

Native mode uses Tauri commands. A remotely hosted browser uses same-origin cookie requests and never receives bearer credentials. Pairing polling and administration run only in native mode. LAN endpoint changes require restart.

## Planned Work

Create domain-specific composables for server lifecycle, nodes, compatibility, and settings only as those domains are implemented.
