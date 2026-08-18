# LS-023 Handoff — Strict HTTPS origin policy and transport resource limits

## Objective

Harden the separate loopback HTTPS lifecycle against cross-origin pairing requests, authority confusion, connection exhaustion, and stalled TLS handshakes without activating it in desktop startup or binding to the LAN.

## State

Completed on 2026-08-18. Changes remain uncommitted in the existing working tree.

## Implemented

- Listener-derived immutable policy allows exactly `localhost:<port>` and the bound loopback IP authority/origin.
- Every encrypted request requires exactly one matching Host header.
- Pairing POSTs require exactly one matching HTTPS Origin; missing, duplicate, malformed, `null`, plaintext, and foreign values fail through one safe `403` response.
- Optional `Sec-Fetch-Site` accepts only `same-origin` or `none`.
- Origin enforcement runs outside pairing rate middleware, so rejected requests do not consume rate or pairing-service capacity.
- `Forwarded` and `X-Forwarded-*` metadata is ignored for authority, origin, and source identity.
- A core-owned semaphore caps accepted TLS connections at 64; each permit remains held for the full connection.
- TLS handshakes time out after five seconds. Saturated and stalled connections fail closed without plaintext fallback.
- Browser session authentication remains restricted to safe GET routes.

## Verified

- `cargo test -p localstream-core server::tests --locked` — PASS; 20 server tests including real-TLS origin, saturation, and timeout recovery.
- `cargo clippy -p localstream-core --all-targets --all-features --locked -- -D warnings` — PASS.
- `cargo fmt --all` — PASS.
- `cargo test --workspace --locked` — PASS; 64 Rust tests.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` — PASS.
- `cargo check --workspace --locked` — PASS.
- `cargo fmt --all --check` — PASS.
- `npm run verify` — PASS; formatting, lint, typecheck, 27 frontend tests, and production build.
- `git diff --check` — PASS.

## Not Verified

- Non-loopback or LAN operation; intentionally not enabled.
- Static browser UI hosting or browser trust installation.
- Unsafe cookie-authenticated browser methods; none exist, so no CSRF token is implemented.
- Production-scale load characteristics beyond deterministic reduced-capacity tests.

## Next Exact Action

Plan LS-024 same-origin static browser UI hosting while keeping the desktop listener and LAN binding unchanged.
