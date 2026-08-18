# Current Task

## ID

LS-023

## Title

Strict HTTPS origin policy and transport resource limits

## Status

Completed

## Goal

Harden the separate HTTPS lifecycle with strict request authority/origin and Fetch Metadata validation for pairing POSTs, bounded concurrent TLS connections, and handshake timeouts while remaining loopback-only.

## Acceptance Criteria

- HTTPS requests require one valid Host authority matching the listener's configured origin names/address.
- Pairing POSTs require an exact same-origin `Origin`; missing, duplicate, malformed, `null`, HTTP, or foreign origins fail uniformly.
- `Sec-Fetch-Site`, when present, permits only `same-origin` or `none`; cross-site values fail.
- Forwarded host/proto/origin headers are ignored.
- Rejected origin/authority requests do not consume pairing service capacity or issue secrets.
- Concurrent accepted TLS connections are capped by a core-owned semaphore.
- TLS handshakes have a short timeout and stalled clients release capacity.
- Excess connections fail closed without reaching Axum or receiving plaintext HTTP.
- Tests cover valid/invalid origin matrix, forged forwarding headers, duplicate headers, connection saturation, timeout recovery, and unchanged safe GET behavior.
- Browser cookie authentication remains limited to safe GET routes; no CSRF token is needed until an unsafe authenticated browser method exists.
- Desktop startup, trusted-local HTTP, static hosting, and LAN binding remain unchanged.

## Relevant Files

- `crates/localstream-core/src/server/mod.rs`
- `crates/localstream-core/src/server/README.md`
- `docs/api/README.md`
- `docs/security/README.md`
- `docs/architecture/adr/0007-private-pki-and-https-origin.md`

## Completed

- LS-022 implemented persistent revocable secure browser sessions.
- HTTPS requires exactly one listener-matching Host authority.
- Pairing POSTs enforce exact same-origin Origin and safe Fetch Metadata before rate limiting or pairing work.
- Forged forwarding headers do not affect authority, origin, or source identity decisions.
- TLS accepts are capped at 64 concurrent connections with a five-second handshake timeout and fail-closed saturation.
- Focused authority/origin, saturation, and timeout-recovery tests pass.

## In Progress

- Nothing.

## Remaining

- Nothing for LS-023.

## Assumptions

- The loopback foundation allows `https://localhost:<port>` and `https://127.0.0.1:<port>` as configured origins.
- TLS connection capacity is 64 and handshake timeout is 5 seconds.
- Requests without Fetch Metadata remain possible for native clients but still require exact Origin on pairing POSTs.

## Next Exact Step

Plan LS-024 same-origin static browser UI hosting without changing the active desktop listener or enabling LAN binding.
