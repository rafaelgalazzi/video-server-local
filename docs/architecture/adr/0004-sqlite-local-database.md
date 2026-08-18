# ADR-0004 — Use SQLite for local persistence

## Status

Accepted

## Context

Each LocalStream node needs durable library, settings, peer, and playback state without an external database service.

## Decision

Use SQLite as the node-local persistent database. Access it through reusable Rust modules rather than UI or transport code.

## Alternatives Considered

- An external database server.
- Flat JSON files.
- In-memory-only state.

## Consequences

Deployment remains embedded, local-first, and cross-platform. Schema migrations, concurrency behavior, backup/recovery, and database tests must be established during implementation.
