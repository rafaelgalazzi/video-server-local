# LocalStream API

## Implemented

- `GET /api/v1/health`

  Returns service identity, application version, API version, status, and whether LAN access is enabled.

- `GET /api/v1/library`

  Returns the current path-free `LibraryScan` JSON or `null` when no library exists. Media entries contain opaque `id`, `title`, `extension`, `sizeBytes`, `probeStatus`, optional `metadata`, and optional `selectedAudioTrackId` fields. Available metadata contains normalized container, duration, video dimensions/codec, and complete audio/subtitle track summaries. Track IDs are opaque; source stream indices and filesystem paths are never returned. `probeStatus` is `available`, `not_probed`, or `unavailable`, allowing one corrupt or inaccessible item to remain safely listed without aborting the library scan.

Audio preference mutation is currently a trusted-local Tauri command rather than an HTTP route. Adding a cookie-authenticated unsafe browser method requires the project’s CSRF-token gate first. The core validates that the opaque track belongs to the current media item and privately resolves it to the ffprobe source index for later compatibility/remux/transcode decisions.

- `GET /api/v1/media/{id}/stream`

  Streams a persisted item from the current approved library by opaque ID. Without `Range`, it returns `200 OK` and the complete file through bounded I/O. A valid single `bytes` range returns `206 Partial Content` with `Accept-Ranges`, `Content-Range`, `Content-Length`, and a video content type. Malformed, multipart, or unsatisfiable ranges return `416 Range Not Satisfiable` and `Content-Range: bytes */{size}`. Unknown IDs return `404` without filesystem details.

Errors use:

```json
{
  "error": {
    "code": "internal_error",
    "message": "The request could not be completed."
  }
}
```

The trusted-local server binds to `127.0.0.1` on an ephemeral port. Separately, an explicitly enabled and security-audited HTTPS server may bind one configured private LAN address after restart. It never falls back to plaintext.

## Authenticated Router Foundation

`authenticated_router` is implemented for tests and future encrypted transports but is not served by the active desktop listener. Its health route is public. Library and stream routes require exactly one `Authorization: Bearer <credential>` header and an active `library.read` peer. Missing, malformed, unknown, and revoked credentials return the same `401` envelope and `WWW-Authenticate: Bearer`; internal credential-store failures remain generic `500` errors.

Bearer authentication does not make plaintext HTTP safe. ADR-0007 selects separate future credential transports: native clients use bearer authorization over pinned TLS, while the same-origin browser UI and native media requests use a revocable secure HttpOnly session cookie. DD-007 records why credentials and signed secrets must not appear in media URLs.

## Encrypted Pairing Foundation

The separate loopback HTTPS lifecycle adds these native-client contracts; they are absent from the active trusted-local HTTP listener:

- `POST /api/v1/pairing/requests`

  Accepts up to 2 KiB of strict JSON containing only `displayName`. Returns `requestId`, `claimSecret`, `verificationCode`, and `expiresInSeconds`. The same verification code appears in the trusted-local approval UI.

- `POST /api/v1/pairing/claims`

  Accepts strict JSON containing only `requestId` and `claimSecret`. After explicit local approval it returns one `peer` summary and `bearerToken`; the request is then permanently replay-protected in the in-memory lifetime. Failed claims return one uniform `pairing_claim_failed` response.

Both endpoints use the accepted socket IP for separate begin/claim per-source and global limits. `X-Forwarded-For` is ignored. Limited responses use `429`, an integer `Retry-After`, and the standard error envelope. These routes do not authorize LAN binding and are not the browser credential mechanism.

Every HTTPS request requires exactly one Host matching the listener's configured `localhost` or loopback-IP authority. Pairing POSTs additionally require exactly one matching HTTPS `Origin`. Missing, duplicate, malformed, `null`, plaintext, and foreign origins return the same safe `403`; optional `Sec-Fetch-Site` permits only `same-origin` or `none`. `Forwarded` and `X-Forwarded-*` headers are ignored. These checks run before pairing rate or service capacity is consumed.

- `POST /api/v1/pairing/browser-claims`

  Accepts the same approved single-use request ID and claim secret, returns `204 No Content`, and sets `__Host-localstream_session` with `HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=86400`. It returns no token, peer, or credential JSON and sets no `Domain` attribute. The session cookie authenticates safe library/media GET requests and is invalidated by expiry or peer revocation.

Browser sessions are persisted only as SHA-256 digests. Supplying both bearer and session credentials, duplicate cookie headers, duplicate session cookie names, malformed tokens, expired/revoked sessions, and unknown capabilities all fail through the uniform authorization response.

## Planned Convention

Versioned REST endpoints use the `/api/v1/` prefix. Streaming and event endpoints must be documented here when their contracts are implemented. HTTP handlers remain thin adapters to reusable Rust services.

Do not expose raw filesystem paths. Public media access must use opaque identifiers, be limited to explicitly approved libraries, enforce the pairing/authentication model, prevent traversal, and use bounded streaming I/O.

## Same-Origin Browser Application

The optional HTTPS asset router serves the production Vue `index.html` and `/assets/` files from one explicitly configured, canonicalized directory. UI files are public so an unpaired browser can render onboarding; protected library and media routes still require a valid bearer credential or secure browser-session cookie.

Client-side navigation falls back to `index.html`, but `/api` and `/api/*` always retain API routing and never return the SPA shell. Missing `/assets/*`, traversal attempts, malformed encoding, files outside the configured root, directories, and files larger than 8 MiB return `404` without path details. Generated `/assets/` responses use one-year immutable caching; HTML and navigation fallbacks use `no-cache, no-store, must-revalidate`. Responses include `X-Content-Type-Options: nosniff` and extension-specific content types.

## Planned

- Remote-browser bootstrap and unsafe-method CSRF protection when an unsafe authenticated method is introduced.
- Event transport.

Planned routes are not contracts until implemented, tested, and recorded here.

The core can run bounded, expiring, explicitly approved pairing requests and its separate HTTPS lifecycle can issue native bearer credentials. ADR-0006 prohibits exposing these routes over plaintext or changing the bind address before every remaining transport/session gate is satisfied.

ADR-0007's private-PKI loopback HTTPS origin, encrypted pairing, browser-session foundation, and optional same-origin production UI hosting are implemented and tested. This is not permission to bind on the LAN; remote bootstrap, client storage, and the remaining release gates are still absent.
