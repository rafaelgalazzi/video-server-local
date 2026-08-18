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

## Important Files

- `mod.rs`: router, thin handlers, lifecycle handle, response models, and contract tests.

## Public Interfaces

- `start_local_server`: binds loopback and spawns the server.
- `ServerHandle`: reports safe address information and triggers graceful shutdown on drop.
- `router`: public for reuse by a future headless distribution and integration tests.

## Dependencies

Axum and Tokio. Handlers call `LocalStreamCore` rather than duplicating domain/database logic.

## Current Limitations

The server is deliberately unreachable from other LAN devices. Pairing/authentication, configurable LAN binding, static web hosting, CORS policy, and request rate limiting are not implemented. Direct Play is limited to eight concurrent streams.

## Planned Work

Add a playback UI while remaining loopback-only, then add pairing and authentication before enabling LAN binding.
