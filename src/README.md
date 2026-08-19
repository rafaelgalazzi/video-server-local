# Frontend Source

## Purpose

This directory contains the Vue 3 + TypeScript presentation layer.

## Features

The responsive shell keeps product identity in a compact header. In native mode, always-visible quick-setup instructions sit above a tabbed workspace that groups library/playback, network, and access configuration. Each active tab uses a centered half-width column on desktop and the full width on smaller screens. Network configuration has one focused surface, advanced node identity controls stay collapsed, and core/version information sits in the page footer. The shell also loads framework-independent application information, presents an approved-folder video scan, plays selected items through the private loopback API, lets the local user decide pairing requests, and administers trusted devices.

## Important Files

- `main.ts`: Vue application entry point.
- `App.vue`: responsive application shell and accessible workspace tabs.
- `styles.css`: shared visual tokens, responsive workspace layout, and shell styling.
- `components/`: presentation components.
- `composables/`: reusable Vue state and backend orchestration.
- `components/PlaybackPanel.vue`: native-controls Direct Play presentation.
- `components/PairingRequestsPanel.vue`: trusted-local pairing review and decision presentation.
- `components/TrustedPeersPanel.vue`: safe active-peer listing and confirmation-based revocation.

## Public Interfaces

The frontend is mounted into `#app`. Backend access belongs in typed composables or services, never directly in presentation components.

## Dependencies

Vue 3, Vite, TypeScript, and the Tauri JavaScript API.

## Current Limitations

The current library is persisted locally in SQLite. Native mode retains Tauri administration and loopback playback; remote browser mode bootstraps through same-origin HTTPS cookies. The desktop can explicitly configure one secure LAN address, disabled by default.

## Planned Work

Add discovery and other new domains only in separately scoped tasks; compatibility-aware desktop playback is implemented.
