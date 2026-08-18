# LS-020 Handoff — Pairing-attempt rate limiting

## Objective

Provide bounded, fail-closed rate-limit decisions for unauthenticated pairing begin and claim attempts before exposing encrypted routes.

## State

Completed on 2026-08-18. Changes remain uncommitted in the existing working tree.

## Implemented

- Independent fixed-window policies: begin allows 5/source and 40/global per 60 seconds; claim allows 10/source and 80/global.
- Source identity comes only from `SocketAddr`, ignores ports, and normalizes IPv4-mapped IPv6 to IPv4.
- Decisions expose only `Allowed` or safe `retry_after_seconds`; counters and capacity are private.
- A shared source map is capped at 1,024 entries and stale entries are removed using monotonic `Instant` values.
- New sources fail closed while capacity is exhausted; existing sources remain governed by their policy.
- Poisoned mutex state fails closed for the full policy window.
- `LocalStreamCore::check_pairing_attempt` makes the limiter reusable by transport adapters.

## Verified

- `cargo test -p localstream-core rate_limit --locked` — PASS; 6 deterministic policy tests.
- `cargo test --workspace --locked` — PASS; 53 Rust tests.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` — PASS.
- `cargo check --workspace --locked` — PASS.
- `cargo fmt --all --check` — PASS.

## Not Verified

- HTTP response mapping and real-socket enforcement; no pairing route exists yet.
- Behavior under production-scale distributed traffic.

## Next Exact Action

Implement LS-021 JSON pairing request/claim endpoints only in the separate HTTPS router. Insert the actual accepted peer socket address, enforce body limits and LS-020 decisions, preserve local approval, return credentials only after a valid approved single-use claim, and add real TLS tests.
