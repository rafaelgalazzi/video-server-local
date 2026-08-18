# Authentication Domain

## Purpose

Own revocable peer credential generation and verification independently of Tauri, HTTP, and future headless adapters.

## Features

- Cryptographically random 256-bit bearer credentials with a recognizable non-secret prefix.
- One-time plaintext return; only SHA-256 token digests are persisted.
- Safe peer identity with an explicit `library.read` capability.
- Missing, invalid, and revoked credential outcomes.
- Persistent revocation through the core-owned SQLite database.

## Important Files

- `mod.rs`: credential lifecycle, safe peer models, validation, and authentication errors.

## Public Interfaces

- `LocalStreamCore::issue_peer_credential`: trusted-local credential issuance foundation.
- `LocalStreamCore::authenticate_peer`: bearer credential verification.
- `LocalStreamCore::revoke_peer`: persistent revocation by opaque peer ID.

## Dependencies

Operating-system randomness through `getrandom`, URL-safe Base64 encoding, SHA-256 digests, UUID peer identifiers, and the private database domain.

## Limitations

No HTTP pairing endpoints, user approval UI, rate limiting, LAN binding, or transport encryption exist. Credentials must not cross a LAN until channel protection and the explicit pairing protocol are implemented.

## Planned Work

Add expiring, replay-resistant pairing requests that require local user approval before calling credential issuance.
