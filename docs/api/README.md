# LocalStream API

## Implemented

- `GET /api/v1/health`

  Returns service identity, application version, API version, status, and whether LAN access is enabled.

- `GET /api/v1/library`

  Returns the current path-free `LibraryScan` JSON or `null` when no library exists. Media entries contain opaque `id`, `title`, `extension`, and `sizeBytes` fields.

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

The server currently binds to `127.0.0.1` on an ephemeral port. The Tauri `server_info` adapter reports the active base URL. This is not yet a LAN-accessible API.

## Authenticated Router Foundation

`authenticated_router` is implemented for tests and future encrypted transports but is not served by the active desktop listener. Its health route is public. Library and stream routes require exactly one `Authorization: Bearer <credential>` header and an active `library.read` peer. Missing, malformed, unknown, and revoked credentials return the same `401` envelope and `WWW-Authenticate: Bearer`; internal credential-store failures remain generic `500` errors.

Bearer authentication does not make plaintext HTTP safe. ADR-0007 selects separate future credential transports: native clients use bearer authorization over pinned TLS, while the same-origin browser UI and native media requests use a revocable secure HttpOnly session cookie. DD-007 records why credentials and signed secrets must not appear in media URLs.

## Planned Convention

Versioned REST endpoints use the `/api/v1/` prefix. Streaming and event endpoints must be documented here when their contracts are implemented. HTTP handlers remain thin adapters to reusable Rust services.

Do not expose raw filesystem paths. Public media access must use opaque identifiers, be limited to explicitly approved libraries, enforce the pairing/authentication model, prevent traversal, and use bounded streaming I/O.

## Planned

- Expiring, rate-limited pairing request/confirmation routes with explicit local approval.
- Static browser UI hosting.
- Event transport.

Planned routes are not contracts until implemented, tested, and recorded here.

The core can run bounded, expiring, explicitly approved pairing requests and issue/revoke peer credentials, but no pairing or credential HTTP API is exposed. ADR-0006 prohibits transmitting pairing claim secrets or bearer credentials over plaintext LAN HTTP or changing the bind address before encrypted transport and route-authorization gates are satisfied.

ADR-0007 defines the future private-PKI HTTPS origin and browser-session policy. It is a design contract, not an implemented route or permission to bind on the LAN.
