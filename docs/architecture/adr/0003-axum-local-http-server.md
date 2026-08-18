# ADR-0003 — Use Axum for the local HTTP server

## Status

Accepted

## Context

LocalStream needs a LAN API, browser UI hosting, REST endpoints, and streaming routes backed by the same reusable Rust core used by Tauri.

## Decision

Use Axum for the local Rust HTTP server. Keep handlers thin and delegate domain behavior to shared core services.

## Alternatives Considered

- A JavaScript HTTP server.
- A separate backend process in another language.
- A Tauri-command-only interface.

## Consequences

HTTP and native adapters can share Rust services and runtime infrastructure. Server lifecycle, authentication, binding, graceful shutdown, and streaming backpressure require explicit design and tests.
