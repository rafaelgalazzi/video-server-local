# LocalStream Project Status

## Last Updated

2026-08-19

## Current Milestone

Phase A.1: MKV track support and browser compatibility before discovery.

## Working

- Two detailed planning documents describe product direction and quality expectations.
- Repository-local governance, continuation, architecture, security, API, and development documentation exists.
- Vue 3 + TypeScript + Vite frontend scaffold and responsive foundation screen.
- Composition API backend state with success/failure unit coverage.
- Tauri 2 shell configured with a thin `app_info` command.
- Framework-independent `localstream-core` Rust crate with a unit test.
- Frontend format, lint, typecheck, test, build, and combined verification scripts.
- Approved-folder Rust scanner for supported video candidates with opaque IDs and no returned paths.
- Native folder-picker adapter and Vue media-library states.
- Embedded SQLite schema, atomic library snapshot persistence, and startup restoration.
- Migration, restart restoration, and new/deleted media reconciliation tests.
- Embedded Axum server with graceful lifecycle, versioned health/library routes, and safe JSON errors.
- Loopback-only opaque-ID Direct Play with containment validation, HTTP single-range responses, bounded asynchronous I/O, and an eight-stream concurrency limit.
- Vue displays the actual loopback API address and exposure status.
- Vue media rows can open an accessible native-controls player using the versioned opaque-ID stream URL, with loading and compatibility-error states.
- Core peer credentials use 256-bit OS randomness, digest-only SQLite persistence, explicit `library.read` capability, restart-safe authentication, and revocation.
- ADR-0006 and the security threat model define pairing, encrypted transport, authorization, rate-limit, and revocation gates that must precede LAN binding.
- Bounded two-minute pairing requests use cryptographic IDs/claim secrets, human verification codes, explicit local decisions, and single-use replay protection.
- Thin Tauri commands can list, approve, or reject pending pairing requests without exposing creation or credential claiming.
- Vue polls pending pairing requests only after native availability and presents accessible code comparison, expiry, Allow/Reject, and retry states.
- Active trusted peers can be listed through path/token-free metadata and revoked persistently through a confirmation-based local UI.
- A dormant authenticated Axum router strictly validates bearer credentials, enforces `library.read`, inserts safe peer identity, and returns uniform unauthorized responses.
- ADR-0007 defines persistent private-PKI node identity, native certificate pinning, explicit browser trust onboarding, and same-origin secure browser sessions.
- A reusable node-root service generates P-256 CA material, derives stable safe identity from SPKI, persists private PKCS#8 only through a platform secret-store boundary, and fails closed on corruption or storage failure.
- Desktop startup serially restores the protected node identity before the loopback server and exposes only its public node ID and root fingerprint through a trusted-local command and UI.
- Explicit local identity reset revokes every active peer before protected-root deletion, fails closed on deletion errors, and requires UI confirmation and restart.
- The node root issues fresh-key 30-day P-256 server leaves with validated DNS/IP SANs, server-only usage, and a leaf-first certificate chain.
- Leaf material converts directly into a TLS 1.3/1.2 Rustls server configuration with HTTP/1.1 ALPN, no client certificate request, and negative trust/name handshake coverage.
- A separate authenticated HTTPS lifecycle binds only ephemeral loopback, rejects plaintext/wrong trust, serves protected routes, and shuts down gracefully; desktop startup remains unchanged.
- Trusted-local root export writes public DER only to a user-selected file after existing-only identity reload and startup-summary verification, with explicit fingerprint/trust guidance and no automatic installation.
- Pairing begin/claim attempts have separate per-source/global monotonic rate policies, normalized actual-socket IPs, bounded stale-cleaned memory, safe retry decisions, and fail-closed state.
- HTTPS-only native pairing routes enforce actual-socket rate limits and strict body bounds, require local approval, issue one bearer credential, reject replay uniformly, and remain absent from trusted-local HTTP.
- Approved browser claims set a 24-hour `__Host-` secure HttpOnly strict-same-site cookie with no response secret; digest-only sessions survive restart and fail uniformly on expiry, malformed cookies, capability errors, or peer/identity revocation.
- The loopback HTTPS surface strictly validates one configured Host, requires exact same-origin Origin and safe Fetch Metadata on pairing POSTs before rate limiting, ignores forwarded authority metadata, caps TLS connections at 64, and times out stalled handshakes after five seconds.
- The dormant HTTPS surface can optionally serve a validated production Vue directory with bounded canonical reads, correct content types, safe cache headers, SPA fallback, strict API precedence, and unchanged protected-route authentication.
- Vue selects native Tauri or remote same-origin HTTPS mode with explicit bootstrap, pairing-required, authenticated, disconnected, and retry states.
- Trusted-local browser trust onboarding requires full-fingerprint acknowledgement and documents platform procedures without warning bypass or automatic installation.
- Native peer trust binds node ID, verified root fingerprint, endpoint hints, and bearer credentials behind an isolated platform-protected store with explicit replacement/deletion.
- LAN configuration is disabled by default, persists one validated explicit private address/port and optional DNS name, and rejects loopback, wildcard, multicast, and public addresses.
- TLS leaves retain the stable root identity, renew before expiry, and rotate for SAN/address changes.
- Desktop startup composes packaged UI, identity, TLS, authenticated routes, sessions, limits, and configuration fail-closed while preserving trusted-local startup on LAN failure.
- Audited activation can bind exactly one private interface only after every Phase A security evidence gate; isolated-interface TLS/auth/downgrade coverage passes.
- FFmpeg/ffprobe discovery validates configured tools and runs structured no-shell processes with bounded output, timeout, cancellation, and kill-on-drop behavior.
- Async persisted scans probe normalized container, duration, video, complete audio, and text/bitmap subtitle metadata; corrupt items fail individually, opaque track mappings remain private, and schema-v4 metadata survives restart.
- Schema-v5 audio preferences validate opaque tracks against the current item, resolve private source indices, survive restart/unchanged rescans, reset when tracks change, and have accessible desktop playback controls.

## In Progress

- LS-070 embedded subtitle selection is the next dependency in Phase A.1.

## Not Started

- Node discovery, pairing, trust, and distributed libraries.
- FFmpeg remux/transcoding and transform concurrency management.
- Automated tests, CI, packaging, and platform verification.

## Known Major Limitations

- Release bundling and installer behavior is unknown / not verified.
- The current local Node.js 22.12 environment is below one transitive lint dependency's declared minimum of 22.13, although verification executed successfully.
- The standalone scanner remains extension-based; desktop persisted scans inspect compatibility metadata with ffprobe.
- Rescans replace the full stored snapshot rather than updating incrementally.
- HTTP and Direct Play are loopback-only on an ephemeral port until pairing/authentication is implemented.
- Direct Play supports one byte range per request; multipart ranges and conditional caching are not implemented.
- Playback compatibility is still delegated to the embedded browser; compatibility decisions and transcoding fallback are not implemented.
- Pairing requests are intentionally memory-only and disappear on restart.
- Native client secret storage is not implemented.
- Physical second-device certificate installation, browser onboarding, and playback are not verified.
- Unsafe authenticated browser methods do not exist; CSRF tokens must be added before any are introduced.
- The verified HTTPS lifecycle is not connected to desktop startup or LAN; interactive certificate export/install and real OS keyring operation remain untested.
- LS-014 through LS-031 changes remain uncommitted in the working tree.

## Next Major Goal

Start LS-070 embedded subtitle selection. Complete compatibility and fallback tasks through LS-049 before Phase B.

The dependency-ordered remaining backlog is maintained in [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md). The committed completion target is the release-ready desktop LAN MVP through LS-060; post-MVP work is gated and must not silently resolve deferred architecture decisions.
