# LS-022 Handoff — Persistent revocable browser sessions

## Objective

Establish an approved browser pairing as a persistent secure cookie session without exposing reusable secrets to JavaScript or URLs.

## State

Completed on 2026-08-18. Changes remain uncommitted in the existing working tree.

## Implemented

- Schema v3 adds browser sessions with SHA-256 token digest, peer/capability binding, creation, expiry, and revocation; v1/v2 migrations preserve existing data.
- Browser session tokens contain 256 random bits and are held only in non-serializable Rust results before `Set-Cookie`.
- Peer and session creation is one SQLite transaction; pairing is not consumed if persistence fails.
- Browser claim returns `204` with `__Host-localstream_session`, `HttpOnly`, `Secure`, `SameSite=Strict`, `Path=/`, `Max-Age=86400`, and no `Domain` or response body.
- Safe library/media GET authentication accepts exactly one native bearer or one valid browser session, rejecting ambiguous, duplicate, malformed, expired, revoked, and unknown-capability credentials uniformly.
- Peer revocation and identity reset mark bound sessions revoked transactionally; expiry pruning deletes stale rows.
- Sessions survive restart while plaintext tokens are absent from SQLite.

## Verified

- `cargo test -p localstream-core browser --locked -- --nocapture` — PASS; persistence and real-TLS cookie tests.
- `cargo test -p localstream-core migrates_version --locked` — PASS; v1/v2 migration tests.
- `cargo test --workspace --locked` — PASS; 62 Rust tests.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` — PASS after correction.
- `cargo check --workspace --locked` — PASS.
- `cargo fmt --all` — PASS.

## Not Verified

- Actual browser cookie behavior, static UI hosting, or browser trust installation.
- Unsafe cookie-authenticated methods; none are exposed yet, so CSRF tokens are intentionally not added in LS-022.
- Non-loopback operation or desktop HTTPS startup.

## Next Exact Action

Implement LS-023 strict HTTPS request authority/origin and Fetch Metadata checks for pairing POSTs, plus bounded concurrent accepted TLS connections and handshake timeout. Keep browser cookie use limited to safe GET requests and remain loopback-only.
