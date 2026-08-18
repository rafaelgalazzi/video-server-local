# HTTP Server

## Purpose

Expose versioned HTTP adapters backed by the reusable Rust core while keeping server lifecycle and network policy explicit.

## Features

- In-process Axum server with graceful shutdown.
- Ephemeral loopback binding for the pre-pairing security phase.
- `GET /api/v1/health` service and capability response.
- `GET /api/v1/library` path-free current-library response.
- Stable JSON error envelope.
- Opaque-ID Direct Play route with full and single-range responses.
- Separate authenticated-router policy with strict bearer parsing and `library.read` enforcement.

## Important Files

- `mod.rs`: router, thin handlers, lifecycle handle, response models, and contract tests.

## Public Interfaces

- `start_local_server`: binds loopback and spawns the server.
- `ServerHandle`: reports safe address information and triggers graceful shutdown on drop.
- `router`: public for reuse by a future headless distribution and integration tests.
- `authenticated_router`: dormant protected health/library/stream composition for future encrypted transports.

## Dependencies

Axum and Tokio. Handlers call `LocalStreamCore` rather than duplicating domain/database logic.

## Current Limitations

The active server is deliberately unreachable from other LAN devices and continues to use the trusted-local router. A separately tested authenticated router exists, but encrypted transport, remote pairing routes, configurable LAN binding, static web hosting, CORS policy, and request rate limiting are not implemented. Direct Play is limited to eight concurrent streams.

## Planned Work

Design encrypted server identity and remote pairing/session transport while remaining loopback-only. Satisfy every ADR-0006 gate before enabling the authenticated router on a listener.
