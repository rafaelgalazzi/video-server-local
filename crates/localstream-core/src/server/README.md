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
- Separate loopback-only HTTPS lifecycle using node-issued Rustls configuration and graceful shutdown.
- HTTPS-only rate-limited pairing request/claim routes backed by the existing local approval service.
- Browser claims that set an opaque secure HttpOnly same-site session cookie and expose no secret body.
- Immutable listener-derived Host/Origin policy, pairing Fetch Metadata checks, bounded TLS connections, and handshake timeouts.

## Important Files

- `mod.rs`: router, thin handlers, lifecycle handle, response models, and contract tests.

## Public Interfaces

- `start_local_server`: binds loopback and spawns the server.
- `ServerHandle`: reports safe address information and triggers graceful shutdown on drop.
- `router`: public for reuse by a future headless distribution and integration tests.
- `authenticated_router`: dormant protected health/library/stream composition for future encrypted transports.
- `start_loopback_https_server`: foundation-only authenticated HTTPS lifecycle that remains disconnected from desktop startup.
- `encrypted_router`: authenticated routes plus bounded pairing begin/claim endpoints for encrypted transports only.
- `HttpsServerHandle`: reports loopback-only HTTPS metadata and supports awaited graceful shutdown.

## Dependencies

Axum, Hyper, Tokio, Tokio-Rustls, and Rustls. Handlers call `LocalStreamCore` rather than duplicating domain/database logic.

## Current Limitations

The active desktop server is deliberately unreachable from other LAN devices and continues to use the trusted-local HTTP router. The separate HTTPS lifecycle exposes native and browser pairing on loopback but is not started by Tauri. It accepts at most 64 TLS connections, allows five seconds for each handshake, validates one configured Host on every request, and requires exact same-origin Origin plus safe optional `Sec-Fetch-Site` on pairing POSTs. Browser cookies authenticate only safe GET library/media routes. Configurable LAN binding, static web hosting, unsafe cookie methods, and CSRF tokens are not implemented. Direct Play is limited to eight concurrent streams.

## Planned Work

Add same-origin static browser UI hosting while remaining loopback-only. Satisfy every ADR gate before any LAN binding.
