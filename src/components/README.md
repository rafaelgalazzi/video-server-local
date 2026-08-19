# Frontend Components

## Purpose

Present accessible UI from typed props and events. Components do not own filesystem, networking, database, or streaming business logic.

## Features

The application shell groups these components into focused Library & playback, Network, and Access tabs in native mode. Browser mode keeps its bootstrap and library flow visible without native-only settings.

- `FoundationStatus.vue` renders native-core availability and retry state.
- `MediaLibraryPanel.vue` renders folder-selection, scan feedback, empty state, safe media summaries, and API-aware Play actions.
- `PlaybackPanel.vue` renders the selected title, native video controls, accessible audio/subtitle selection, bounded text tracks, explicit bitmap limitations, loading/error feedback, and close action.
- `NodeIdentityPanel.vue` keeps advanced node ID, fingerprint, trust export, and identity-reset controls in an expandable disclosure.
- `PairingRequestsPanel.vue` renders safe pending-device metadata, matching-code guidance, expiry, and explicit Allow/Reject actions.
- `TrustedPeersPanel.vue` renders safe active-device metadata and a distinct revocation confirmation step.
- `ServerStatus.vue` renders the embedded API address and its loopback/LAN availability state.
- `BrowserBootstrapPanel.vue` renders remote bootstrap, pairing-required, authenticated, disconnected, and retry states.
- `LanServerPanel.vue` renders explicit private-address configuration and safe status.

## Public Interfaces

Components expose Vue props and events documented by their TypeScript declarations.

## Dependencies

Vue presentation APIs and types from `src/composables` where appropriate.

## Current Limitations

Fallback profiles are intentionally software-only and desktop-local; physical browser/device compatibility remains to be verified. Artwork, search, and playback-position persistence are not implemented.

## Planned Work

Add feature-specific component groups alongside their tests as product slices are implemented.
