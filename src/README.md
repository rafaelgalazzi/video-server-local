# Frontend Source

## Purpose

This directory contains the Vue 3 + TypeScript presentation layer.

## Features

The shell displays product identity, loads framework-independent application information, and presents an approved-folder video scan through typed backend adapters.

## Important Files

- `main.ts`: Vue application entry point.
- `App.vue`: initial application shell.
- `styles.css`: shared visual tokens and shell styling.
- `components/`: presentation components.
- `composables/`: reusable Vue state and backend orchestration.

## Public Interfaces

The frontend is mounted into `#app`. Backend access belongs in typed composables or services, never directly in presentation components.

## Dependencies

Vue 3, Vite, TypeScript, and the Tauri JavaScript API.

## Current Limitations

The scanned library is session-only. Routing, persistence, server control, and playback are not implemented.

## Planned Work

Add the first approved-folder and library flow in a separately scoped task.
