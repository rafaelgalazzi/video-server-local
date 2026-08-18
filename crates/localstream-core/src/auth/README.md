# Authentication Domain

## Purpose

Own revocable peer credential generation and verification independently of Tauri, HTTP, and future headless adapters.

## Features

- Cryptographically random 256-bit bearer credentials with a recognizable non-secret prefix.
- One-time plaintext return; only SHA-256 token digests are persisted.
- Safe peer identity with an explicit `library.read` capability.
- Missing, invalid, and revoked credential outcomes.
- Persistent revocation through the core-owned SQLite database.
- In-memory pairing requests limited to 32 active entries and two-minute monotonic lifetimes.
- Cryptographic request IDs and 256-bit claim secrets plus six-digit human verification codes.
- Explicit approval/rejection and single-use credential claiming with replay tombstones.

## Important Files

- `mod.rs`: credential lifecycle, safe peer models, validation, and authentication errors.
- `pairing.rs`: bounded request lifecycle, local decisions, expiration, and single-use claim behavior.

## Public Interfaces

- `LocalStreamCore::issue_peer_credential`: trusted-local credential issuance foundation.
- `LocalStreamCore::authenticate_peer`: bearer credential verification.
- `LocalStreamCore::revoke_peer`: persistent revocation by opaque peer ID.
- `LocalStreamCore` pairing methods: begin, list pending, approve, reject, and claim.

## Dependencies

Operating-system randomness through `getrandom`, URL-safe Base64 encoding, SHA-256 digests, constant-time claim-secret comparison, UUID peer identifiers, and the private database domain.

## Limitations

No HTTP pairing endpoints, approval UI, network rate limiting, LAN binding, or transport encryption exist. Pairing requests intentionally disappear on restart. Credentials must not cross a LAN until channel protection and authenticated routes are implemented.

## Planned Work

Add the local approval UI, then design encrypted request/claim routes with rate limiting before LAN exposure.
