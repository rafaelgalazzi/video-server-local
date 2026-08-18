# Current Task

## ID

LS-012

## Title

Authenticated encrypted LAN identity and transport design

## Status

Completed

## Goal

Define the offline-capable TLS identity, client trust, browser origin, credential transport, and listener gates required before LocalStream can expose authenticated routes beyond loopback.

## Acceptance Criteria

- A new ADR defines persistent node identity and authenticated TLS without requiring an Internet service.
- Native-client certificate pinning and browser certificate trust are treated as distinct onboarding profiles.
- Static browser assets and APIs use one HTTPS origin.
- Browser authentication works with native media elements without exposing bearer credentials to JavaScript or URLs.
- CSRF, origin, certificate rotation, key storage, discovery, and listener lifecycle requirements are explicit.
- The order of implementation is split into safe, independently testable follow-up tasks.
- No LAN listener, TLS dependency, certificate, secret, or production route is introduced by this design task.

## Relevant Files

- `docs/architecture/adr/0007-private-pki-and-https-origin.md`
- `docs/architecture/adr/README.md`
- `docs/architecture/ARCHITECTURE_MAP.md`
- `docs/security/README.md`
- `docs/api/README.md`
- `.ai/DEFERRED_DECISIONS.md`
- `.ai/PROJECT_STATUS.md`

## Completed

- Reviewed ADR-0006, LS-011, the security model, API policy, architecture map, product plan, and deferred browser credential decision.
- Confirmed that a zero-install browser cannot automatically trust a private offline server identity.
- Selected an offline private-PKI model with native pinning and explicit browser trust onboarding.
- Accepted ADR-0007 with separate native and browser trust profiles, same-origin HTTPS, secure browser sessions, lifecycle policy, and test gates.
- Resolved DD-007 in favor of secure HttpOnly same-origin sessions rather than browser-visible bearer tokens or signed media URLs.
- Updated the architecture map, API policy, security model, project status, and ADR index.
- Verified documentation formatting and whitespace.

## In Progress

- Nothing.

## Remaining

- Nothing for LS-012.

## Assumptions

- Primary LAN operation must continue without Internet access.
- Browser trust installation is an explicit local administrative action, not something an untrusted web page may automate.
- LS-012 is design-only and does not authorize binding beyond loopback.

## Next Exact Step

Start LS-013 by implementing a reusable persistent node-root identity service and protected-storage boundary with restart/failure tests, while keeping every HTTP listener loopback-only.
