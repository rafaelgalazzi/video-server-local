# Frontend Source

## Purpose

This directory contains the Vue 3 + TypeScript presentation layer.

## Features

The initial shell displays product identity and loads framework-independent application information through a backend adapter.

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

There is no media library, routing, persistence, server control, or playback implementation.

## Planned Work

Add the first approved-folder and library flow in a separately scoped task.
