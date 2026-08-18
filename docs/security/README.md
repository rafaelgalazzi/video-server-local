# Security Model

This document combines implemented safeguards with requirements for unimplemented network and playback features.

## Trust and Pairing

- LAN location does not imply trust; unpaired peers are untrusted.
- Pairing must require explicit user approval and establish revocable credentials.
- Current loopback routes are trusted-local adapters and remain unauthenticated.
- The core credential foundation issues 256-bit random bearer tokens only to trusted local callers. Plaintext is returned once; SQLite stores a SHA-256 digest, safe peer metadata, `library.read` capability, and revocation state.
- The in-memory pairing service limits active requests to 32, expires them after two monotonic minutes, uses collision-checked cryptographic request IDs and 256-bit claim secrets, compares claim digests in constant time, requires a matching six-digit approval code, and makes claims single-use.
- Trusted-local Tauri adapters can only list, approve, or reject requests. They cannot create requests or claim credentials.
- The local Vue interface polls pending requests after native startup, warns users to compare codes, and exposes explicit Allow/Reject actions with retryable failure state.
- Trusted-peer administration exposes only opaque ID, display name, `library.read`, and creation time. Revocation requires a separate confirmation step, persists across restart, and immediately invalidates authentication.
- A dormant authenticated Axum router accepts exactly one strict bearer header, authenticates through the core, inserts safe peer identity, and protects library/stream routes with `library.read`.
- Missing, malformed, unknown, and revoked credentials receive the same safe `401`; the active desktop listener still uses the loopback-only local router.
- ADR-0007 requires a persistent private node CA: native clients pin its verified public-key fingerprint, while browser devices explicitly install it into their trust store after trusted-local fingerprint comparison.
- The LS-013 core service generates/restores a P-256 CA key through an injected secret-store boundary, derives stable public identity from its SPKI, and fails closed for corrupt or unavailable protected storage. Its platform adapter targets Windows Credential Manager, Apple Keychain, and Linux Secret Service.
- Desktop startup restores that identity before starting the loopback server and retains only its public summary. Explicit local reset first revokes all active peer credentials, then deletes the protected root; deletion failure leaves peers revoked and never creates a silent replacement.
- The node identity can issue fresh P-256 TLS server keys with 30-day root-signed leaves for at most 16 validated explicit DNS/IP names. Wildcards, duplicates, malformed names, client-auth usage, and CA usage are rejected or absent by construction; no listener consumes the material yet.
- Issued leaf material is consumed directly into Rustls configured for TLS 1.3/1.2, no client certificates, and HTTP/1.1 ALPN. In-memory handshakes verify trusted-root/name success and wrong-root/name rejection; no socket consumes the configuration yet.
- A separate authenticated HTTPS lifecycle is verified on ephemeral loopback sockets for trusted health/library requests, bearer enforcement, wrong-root/name rejection, plaintext downgrade rejection, graceful shutdown, and listener release. Desktop startup still uses only its original trusted-local HTTP listener.
- Trusted-local root export reloads only an existing identity, verifies it matches the startup summary, and writes public DER through a native save dialog without returning bytes or paths to Vue. Guidance requires full-fingerprint comparison; LocalStream does not install trust automatically or offer a remote CA download.
- Pairing-attempt rate limiting defines independent begin/claim windows, per-source and global quotas, actual-socket IP normalization, 1,024-source bounded memory, stale cleanup, safe retry seconds, and fail-closed poisoned state.
- HTTPS-only pairing routes apply those limits before strict 2 KiB JSON parsing, require the existing trusted-local approval, issue one native bearer credential, and return uniform failed-claim/replay responses. Forged forwarding headers do not affect the source bucket; the active HTTP listener has no pairing routes.
- Browser claims consume the same approved single-use exchange and return no secret body, setting only a `__Host-` 24-hour `HttpOnly; Secure; SameSite=Strict; Path=/` cookie. SQLite stores its SHA-256 digest and peer/capability/expiry/revocation binding; peer revocation and identity reset invalidate sessions transactionally.
- The HTTPS router requires exactly one configured Host on all requests. Pairing POSTs require one exact same-origin Origin and, when supplied, `Sec-Fetch-Site: same-origin` or `none`; all invalid variants fail uniformly before rate limits or pairing state. Forwarded authority metadata is ignored.
- The HTTPS accept loop holds a semaphore permit for each connection, caps concurrency at 64, limits TLS handshakes to five seconds, and drops excess or stalled connections without plaintext fallback.
- The future browser UI, API, and media routes share one HTTPS origin. Browser pairing establishes a revocable `HttpOnly`, `Secure`, `SameSite=Strict` session cookie; credentials are not placed in JavaScript storage or media URLs.
- Client secret storage, same-origin static UI hosting, and LAN activation are not yet implemented.

### Threat Model

| Boundary or asset            | Threat                                                 | Implemented mitigation                                                                                                  | Remaining gate                                                                 |
| ---------------------------- | ------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| LAN listener                 | Unpaired or hostile peers request library/control data | Listener remains loopback-only                                                                                          | Authenticate every non-public route before LAN binding                         |
| Pairing flow                 | Guessing, replay, approval bypass, or request flooding | Bounded expiring requests, explicit local UI decisions, 256-bit claim secrets, and replay tombstones; no remote surface | Encrypted and rate-limited request/claim routes                                |
| Bearer credential in transit | Passive capture and replay                             | Credentials never traverse the LAN; ADR-0007 defines pinned/private-PKI TLS                                             | Implement protected node identity and TLS without plaintext fallback           |
| SQLite credential store      | Database disclosure exposes reusable secrets           | Only SHA-256 digests are stored; tokens have 256 random bits                                                            | Protect application data using platform controls and avoid backups/log leakage |
| Trusted client               | Client secret theft or loss                            | Local peer listing and confirmation-based persistent revocation                                                         | Platform-appropriate secure client storage                                     |
| Authorization                | Valid peer invokes excessive capabilities              | Separate router enforces strict bearer authentication and `library.read` with negative tests                            | Activate only behind encrypted transport; add future capabilities fail-closed  |
| Local adapter                | Untrusted remote input invokes credential issuance     | Tauri and Vue expose only list/approve/reject; issuance requires an approved high-entropy claim                         | Keep request creation/claim off the Tauri surface                              |

### Planned Route Policy

- Health/capability discovery may expose only minimal non-sensitive information.
- Pair-request endpoints may be unauthenticated only when bounded by request-size limits, expiration, replay protection, and rate limits.
- Pair confirmation must require a pending request plus explicit local approval.
- Library metadata and media streams require an active `library.read` credential on LAN.
- Control, settings, scanning, filesystem selection, peer administration, and credential issuance must never be remotely available under `library.read`.
- Missing, invalid, revoked, and insufficient credentials must fail safely without revealing which secret component was wrong.

## Filesystem Exposure

- Only explicitly user-approved libraries may expose media.
- Raw paths are private and must never appear in public API payloads or URLs.
- Clients use opaque media identifiers resolved inside the trusted core.
- Canonicalization and containment checks must prevent directory traversal, symlink escapes where applicable, and identifier manipulation.

### Implemented in LS-002

- A user must choose the scan root through the native folder picker.
- The core canonicalizes the approved root and does not follow directory symlinks during traversal.
- Scan responses contain UUID media identifiers and display metadata only; filesystem paths remain inside Rust.
- Files are not opened or loaded during discovery; only directory entries and file metadata are read.
- Individual traversal/metadata failures are skipped and counted rather than exposing their paths or aborting successful entries.

Persistence and future streaming routes must resolve opaque IDs against trusted internal records and revalidate containment before file access.

### Implemented in LS-003

- Approved root and media paths are stored only in the core-owned SQLite database under the OS application-data directory.
- Vue restoration receives the same path-free `LibraryScan` model used after a live scan.
- Snapshot replacement is transactional, so a failed write cannot expose a partially updated library.
- The database schema rejects unknown future versions instead of silently downgrading them.

### Implemented in LS-004

- The embedded HTTP server binds only to `127.0.0.1` while pairing/authentication is absent.
- HTTP library responses reuse the path-free core model and are contract-tested against filesystem path disclosure.
- Handler failures return a generic stable JSON envelope without database or filesystem details.
- The UI explicitly reports that LAN access is unavailable rather than presenting loopback as a shareable address.

### Implemented in LS-005

- Streaming accepts only opaque IDs persisted for the current approved library; request values are never interpreted as paths.
- The approved root and selected file are canonicalized immediately before access, and files outside the root are rejected.
- Tokio file streams are bounded to the requested byte count and never load a complete media file into memory.
- A core-owned semaphore limits Direct Play to eight concurrent streams; excess requests receive a safe `503` response.
- Missing, invalid, outside-root, and unavailable files do not disclose internal filesystem details.
- The streaming route remains loopback-only while pairing/authentication is absent.

LAN binding must not be enabled until requests are authenticated under an implemented trust model.

Credential persistence alone does not satisfy this gate. ADR-0006 requires encrypted transport, explicit approval, protected routes, rate limits, revocation controls, and negative security tests before any non-loopback bind.

LS-011 established the protected-route and negative-authentication foundation. LS-013 through LS-023 subsequently added the separate loopback TLS and browser-session foundation, but the active desktop listener remains trusted-local HTTP and LAN binding is still prohibited.

ADR-0007 resolves the intended transport and initial browser-session architecture. LS-013 through LS-023 implement loopback-tested encrypted native/browser pairing, revocable secure sessions, strict pairing origin checks, and transport resource limits. LAN binding remains prohibited until static hosting, client-side trust/secret handling, and remaining negative-test gates are complete. CSRF tokens become required if unsafe authenticated browser methods are introduced.

## Processes and Resources

- Invoke FFmpeg with structured argument APIs, never unsafe shell interpolation. Treat media and metadata as untrusted input.
- Stream using bounded I/O; never load entire media files into memory.
- Put concurrency, timeout, cancellation, request-size, and rate limits around expensive scans, streams, probes, and transcodes.
- Network failures and inaccessible files must be contained rather than crashing the process or exposing unrelated data.

## Privacy

Primary LAN features must work without Internet access. External metadata or remote-access features, if later adopted, must be explicit, opt-in, and documented. Logs must not leak paths, credentials, pairing secrets, or sensitive media metadata unnecessarily.

Security behavior must receive unit/integration tests and threat-focused review as it is implemented. Update this file with the actual authentication and pairing model before exposing a LAN API.
