# ADR-0007 — Private PKI and same-origin HTTPS

## Status

Accepted

## Context

ADR-0006 prohibits bearer credentials on plaintext LAN HTTP. LocalStream must authenticate the server as well as the peer, work without an Internet identity provider, support native clients and ordinary browsers, and keep browser media playback compatible with native `<video>` requests that cannot add an arbitrary authorization header.

A private self-signed identity can encrypt traffic, but encryption alone does not authenticate the intended node. Native clients can pin a verified certificate or public-key fingerprint. Browsers instead rely on their platform trust store and must not be expected to bypass certificate warnings. Publicly trusted certificates and public DNS would simplify browser onboarding but would make primary LAN setup or renewal depend on Internet infrastructure.

## Decision

### Node identity and private PKI

- Each server node owns a persistent, randomly generated private root CA identity and a stable node ID. Neither is shared between nodes.
- The root private key is stored through a platform credential/key-protection adapter. A headless fallback may use a permission-restricted file only after its threat model and backup behavior are documented.
- The node issues short-lived leaf certificates for its current advertised DNS name and IP addresses. Leaf renewal does not change the pinned root identity.
- Private keys, bearer credentials, session values, claim secrets, and full certificate material are never logged or exposed through general API responses.
- Loss or compromise of the root identity requires explicit identity reset, revocation of all peers and sessions, and fresh trust onboarding. Silent replacement is prohibited.

### Client trust profiles

- Native clients pin the node root public-key fingerprint during an authenticated pairing ceremony. Discovery data is only a locator and never establishes trust.
- Browser clients require an explicit administrative installation of the node root certificate into the device or browser trust store. LocalStream displays the full fingerprint through a trusted-local surface for comparison before installation.
- A certificate warning, click-through exception, unauthenticated CA download, LAN address, mDNS result, or remembered IP address is not proof of node identity.
- Public-CA and cloud-mediated identity may be reconsidered only as optional onboarding; primary LAN operation must remain independent of it.

### HTTPS origin and browser sessions

- The remote browser UI, REST API, pairing endpoints, and media routes are hosted by the same HTTPS origin. Cross-origin browser API access is denied by default.
- A successful browser pairing claim establishes an opaque, revocable, `HttpOnly`, `Secure`, `SameSite=Strict` session cookie. Long-term bearer material is not exposed to JavaScript, local storage, query strings, fragments, or media URLs.
- Native clients continue using the authorization-header credential defined by ADR-0006 over pinned TLS.
- Cookie-authenticated unsafe methods require strict `Origin` validation and an explicit CSRF token. Fetch Metadata checks provide defense in depth where available. Safe methods must not mutate state.
- Native `<video>` requests use the same-origin secure session cookie. Signed media URLs are not part of the initial design because URLs can leak through logs, history, referrers, and copied links.
- Sessions are capability-bound, expire, rotate after pairing and privilege changes, and become invalid immediately when their peer is revoked.

### Listener and lifecycle policy

- The trusted desktop loopback listener remains separate from the future HTTPS LAN listener.
- The LAN listener must fail closed if identity loading, leaf issuance, TLS configuration, authentication policy, or required limits fail. It must never fall back to plaintext.
- Binding is explicit and configurable; wildcard binding is not a hidden default. Reported addresses distinguish loopback from LAN exposure.
- Discovery advertises only the HTTPS endpoint, stable node ID, and non-secret capability/version hints. Clients still verify the pinned or platform-trusted certificate.
- Certificate renewal and network-address changes must not crash active local functionality. New connections receive a valid current leaf; lifecycle tests cover restart, rotation, shutdown, and partial failure.

### Implementation gates

LAN binding remains prohibited until all ADR-0006 gates plus the following are implemented and tested:

1. Persistent node-root generation, protected storage, reload, reset, and non-secret fingerprint APIs.
2. Leaf issuance and TLS serving with hostname/IP verification and no plaintext fallback.
3. Native pin onboarding or browser root-trust onboarding with fingerprint comparison.
4. Rate-limited encrypted pairing request and claim routes bound to the authenticated TLS identity.
5. Same-origin static hosting, secure browser sessions, CSRF/origin enforcement, and revocation.
6. Request-size, connection, pairing-attempt, and expensive-operation limits.
7. Negative tests for untrusted certificates, changed identity, expired leaves, invalid origins, session misuse, downgrade attempts, restart, and revocation.

Implementation proceeds in that order where dependencies require it. Early identity and TLS work remains loopback-testable and does not itself authorize LAN exposure.

## Alternatives Considered

- Plain HTTP with bearer tokens: rejected because passive LAN observers can steal reusable credentials.
- Ephemeral self-signed certificates: rejected because clients cannot distinguish restart from impersonation and cannot retain trust safely.
- Trust on first use without user verification: rejected because the first connection can be intercepted.
- Browser certificate-warning bypass: rejected because it trains unsafe behavior and does not provide a dependable product trust ceremony.
- Public CA plus public or dynamic DNS as the only path: rejected because primary LAN operation would depend on external infrastructure and renewal.
- Cross-origin UI and API: rejected because it expands CORS and credential risk without an MVP benefit.
- Bearer tokens in browser storage or signed media URLs: rejected for the initial browser design because script access or URL leakage increases credential exposure.

## Consequences

LocalStream gains an offline-capable identity model with explicit trust rather than treating TLS encryption as authentication. Native onboarding can use pinning, while browser onboarding requires a deliberate certificate installation step. This is less convenient than a public certificate but preserves the local-first requirement.

The server must add a platform key-protection boundary, certificate lifecycle code, HTTPS hosting, session persistence, origin/CSRF enforcement, and extensive negative tests before LAN exposure. DD-007 is resolved for the initial browser design by same-origin secure cookies; signed stream URLs remain outside the initial scope. LS-012 makes no runtime or bind change.
