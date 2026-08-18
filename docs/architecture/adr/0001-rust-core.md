# ADR-0001 — Use Rust for the native and core backend

## Status

Accepted

## Context

LocalStream needs native filesystem and networking access, efficient bounded streaming, cross-platform packaging, and a core that can later support desktop, mobile, headless, CLI, and NAS targets.

## Decision

Use Rust for the reusable application/server core and Tauri 2 native integration. Keep domain services independent of Tauri adapters wherever practical.

## Alternatives Considered

- A separate Go backend runtime.
- Business logic implemented in TypeScript inside the UI/native shell.

## Consequences

Tauri integration has one native language and media/server code can be efficient and reusable. Contributors must manage Rust async, platform boundaries, and safe resource usage. Another backend runtime such as Go is not introduced.
