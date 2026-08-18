# Frontend Components

## Purpose

Present accessible UI from typed props and events. Components do not own filesystem, networking, database, or streaming business logic.

## Features

- `FoundationStatus.vue` renders native-core availability and retry state.
- `MediaLibraryPanel.vue` renders folder-selection, scan feedback, empty state, and safe media summaries.

## Public Interfaces

Components expose Vue props and events documented by their TypeScript declarations.

## Dependencies

Vue presentation APIs and types from `src/composables` where appropriate.

## Current Limitations

Media rows are informational only; playback, artwork, search, and persistence are not implemented.

## Planned Work

Add feature-specific component groups alongside their tests as product slices are implemented.
