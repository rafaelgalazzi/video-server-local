# ADR-0006 — Revocable peer bearer credentials

## Status

Accepted

## Context

LocalStream must authenticate explicitly approved LAN peers without an Internet identity provider. LAN membership and source IP are not identities. Credentials must survive restarts, support revocation, avoid plaintext-at-rest storage, and remain reusable by desktop, headless, CLI, and future mobile cores.

Bearer credentials do not provide transport confidentiality. Sending one over plaintext LAN HTTP would allow interception and credential reuse, so credential storage alone is not sufficient to authorize LAN binding.

## Decision

- The reusable Rust core owns peer credential generation, verification, capability assignment, persistence, and revocation.
- A credential contains 256 random bits from the operating system and is encoded as URL-safe Base64 with an `ls_peer_` identifier prefix.
- Plaintext is returned exactly once to the trusted caller. SQLite stores only its SHA-256 digest, peer ID, display name, capability, creation time, and revocation state.
- The initial and only capability is `library.read`. Capabilities fail closed when an unknown stored value is encountered.
- Revocation retains the digest so a revoked token can be distinguished and rejected across restarts.
- Credential issuance remains a trusted-local core operation until an explicit user-approved, expiring, replay-resistant pairing protocol calls it.
- Current loopback routes remain unchanged. No bearer credential may traverse an unprotected LAN channel.

LAN binding is prohibited until all of these are implemented and tested:

1. An encrypted channel with authenticated server identity.
2. Explicit local user approval for each pairing request.
3. Expiration, replay protection, and rate limiting for pairing attempts.
4. Authentication and capability enforcement on every non-public route.
5. Peer listing and revocation controls.
6. Negative authorization, origin/CORS, malformed-input, and restart tests.

## Alternatives Considered

- Trusting LAN source addresses: rejected because local networks and peer devices may be hostile.
- Persisting plaintext tokens: rejected because database disclosure would immediately expose reusable credentials.
- Short numeric codes as long-term credentials: rejected because they lack sufficient entropy; a code may only verify an expiring pairing exchange.
- Internet accounts or a hosted identity provider: rejected because primary LAN operation must remain local-first.
- Enabling plaintext HTTP with bearer tokens: rejected because passive LAN observers could steal reusable credentials.

## Consequences

Credential mechanics and revocation can be tested before exposing a network attack surface. Database compromise does not reveal directly reusable bearer strings, although offline verification of a stolen candidate remains possible; 256-bit random tokens make guessing infeasible. Client-side secret storage, pairing endpoints, channel protection, middleware, rate limits, and LAN binding remain separate required work. Lost plaintext credentials cannot be recovered and must be revoked/reissued.
