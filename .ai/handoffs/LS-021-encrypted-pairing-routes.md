# LS-021 Handoff — Encrypted pairing request and claim routes

## Objective

Expose bounded native pairing begin and one-time credential claim only through the separate HTTPS lifecycle.

## State

Completed on 2026-08-18. Changes remain uncommitted in the existing working tree.

## Implemented

- Strict 2 KiB JSON endpoints for pairing requests and claims in `encrypted_router` only.
- The TLS accept loop inserts the actual accepted `SocketAddr`; rate middleware ignores forwarding headers by construction.
- `429` responses include integer `Retry-After` and a stable safe envelope.
- Request responses include the expiring request/claim secrets and verification code needed for native comparison and local approval.
- Claims issue one bearer credential only after approval; preapproval, unknown request, bad secret, and replay share the same safe claim failure response.
- JSON unknown fields and malformed input fail safely; oversized bodies return a safe `413`.
- The original trusted-local HTTP router has no pairing endpoints.

## Verified

- `cargo test -p localstream-core encrypted_pairing --locked -- --nocapture` — PASS; 2 real-TLS route flows.
- `cargo test -p localstream-core trusted_local_http_router_does_not_expose_pairing_routes --locked` — PASS.
- `cargo test --workspace --locked` — PASS; 56 Rust tests.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` — PASS.
- `cargo check --workspace --locked` — PASS.

## Not Verified

- Browser session cookies; LS-021 returns native bearer credentials only.
- Non-loopback use, desktop startup integration, or production proxy/network behavior.

## Next Exact Action

Implement LS-022 database-backed opaque browser sessions bound to peers/capabilities. Add a browser claim endpoint that sets an `HttpOnly; Secure; SameSite=Strict` cookie, authenticate safe library/media requests from it, enforce expiry and peer revocation, and prove that no long-term secret enters JSON or URLs.
