# Current Task

## ID

LS-011

## Title

Bearer authentication and library-read authorization

## Status

Completed

## Goal

Create a separately testable authenticated Axum router that protects library metadata and media streams with strict bearer credentials and `library.read`, while leaving the active desktop loopback router and listener unchanged.

## Acceptance Criteria

- A separate authenticated router exposes public health and protected library/stream routes.
- Exactly one strict `Authorization: Bearer <token>` header is accepted.
- Missing, malformed, unknown, and revoked credentials return the same safe `401` body and `WWW-Authenticate: Bearer` header.
- Credential-store failures return the existing generic `500` without authentication details.
- Successful authentication inserts safe peer identity into request extensions and requires `library.read`.
- Valid credentials can read library metadata and stream byte ranges through existing thin handlers.
- The current `router` and `start_local_server` behavior remain unauthenticated loopback-only for the desktop webview.
- Negative and positive contract tests cover header parsing, invalid/revoked tokens, library access, and Range streaming.
- API, security, server, test-matrix, and project documentation remain explicit that encrypted transport is still required.

## Relevant Files

- `crates/localstream-core/src/server/mod.rs`
- `crates/localstream-core/src/auth/mod.rs`
- `docs/api/README.md`
- `docs/security/README.md`
- `docs/development/TEST_MATRIX.md`

## Completed

- Added a separately composed authenticated router without changing the active desktop listener.
- Enforced one strict bearer header, active credentials, and `library.read` on library and stream routes.
- Added uniform safe unauthorized responses and generic handling for credential-store failures.
- Inserted safe authenticated peer identity into request extensions.
- Added positive and negative contract tests for authentication, library access, and ranged streaming.
- Updated API, security, server, test-matrix, deferred-decision, and project documentation.
- Completed frontend and Rust verification.

## In Progress

- Nothing.

## Remaining

- Nothing for LS-011.

## Assumptions

- The authenticated router is a dormant reusable foundation and is not served by `start_local_server` in LS-011.
- `library.read` is the only current peer capability; middleware still checks it explicitly.
- Remote browser media elements cannot attach arbitrary bearer headers; the eventual encrypted browser session/signed-stream strategy remains a later design decision.
- ADR-0006 continues to prohibit LAN binding and plaintext credential transport.

## Next Exact Step

Start LS-012 by documenting the authenticated encrypted LAN server identity and transport design before implementing or enabling any remote listener.
