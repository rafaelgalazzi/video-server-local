# Authentication Domain

## Purpose

Own revocable peer credential generation and verification independently of Tauri, HTTP, and future headless adapters.

## Features

- Cryptographically random 256-bit bearer credentials with a recognizable non-secret prefix.
- One-time plaintext return; only SHA-256 token digests are persisted.
- Safe peer identity with an explicit `library.read` capability.
- Missing, invalid, and revoked credential outcomes.
- Persistent revocation through the core-owned SQLite database.
- Safe active-peer summaries containing no credential or filesystem data.
- In-memory pairing requests limited to 32 active entries and two-minute monotonic lifetimes.
- Cryptographic request IDs and 256-bit claim secrets plus six-digit human verification codes.
- Explicit approval/rejection and single-use credential claiming with replay tombstones.
- Fixed-window pairing-attempt limits per normalized source IP and globally, with bounded source memory.
- Persistent 24-hour browser sessions with digest-only storage and peer-bound revocation.

## Important Files

- `mod.rs`: credential lifecycle, safe peer models, validation, and authentication errors.
- `pairing.rs`: bounded request lifecycle, local decisions, expiration, and single-use claim behavior.
- `rate_limit.rs`: separate begin/claim policies, safe retry decisions, normalization, cleanup, and fail-closed state.
- `session.rs`: opaque browser session issuance, validation, expiry pruning, and safe cookie constants.

## Public Interfaces

- `LocalStreamCore::issue_peer_credential`: trusted-local credential issuance foundation.
- `LocalStreamCore::authenticate_peer`: bearer credential verification.
- `LocalStreamCore::revoke_peer`: persistent revocation by opaque peer ID.
- `LocalStreamCore::trusted_peers`: safe active-peer administration view.
- `LocalStreamCore` pairing methods: begin, list pending, approve, reject, and claim.
- `LocalStreamCore::check_pairing_attempt`: reusable transport-facing limiter decision based on the actual peer socket address.
- `LocalStreamCore::authenticate_browser_session`: validates digest-only sessions and their active peer/capability binding.

## Dependencies

Operating-system randomness through `getrandom`, URL-safe Base64 encoding, SHA-256 digests, constant-time claim-secret comparison, UUID peer identifiers, and the private database domain.

## Limitations

Native and browser pairing claim endpoints exist only in the separate loopback HTTPS lifecycle and use the limiter plus local approval. Browser sessions survive restart, expire after 24 hours, and are invalidated with their peer. Pairing requests intentionally disappear on restart. LAN binding and unsafe cookie-authenticated methods remain unavailable.

## Planned Work

Add strict HTTPS origin/fetch-metadata policy and transport resource limits before LAN exposure.
