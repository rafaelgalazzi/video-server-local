# LocalStream API

## Implemented

- `GET /api/v1/health`

  Returns service identity, application version, API version, status, and whether LAN access is enabled.

- `GET /api/v1/library`

  Returns the current path-free `LibraryScan` JSON or `null` when no library exists. Media entries contain opaque `id`, `title`, `extension`, and `sizeBytes` fields.

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

## Planned Convention

Versioned REST endpoints use the `/api/v1/` prefix. Streaming and event endpoints must be documented here when their contracts are implemented. HTTP handlers remain thin adapters to reusable Rust services.

Do not expose raw filesystem paths. Public media access must use opaque identifiers, be limited to explicitly approved libraries, enforce the pairing/authentication model, prevent traversal, and use bounded streaming I/O.

## Planned

- Authentication and pairing routes.
- Opaque-ID Direct Play/HTTP Range routes.
- Static browser UI hosting.
- Event transport.

Planned routes are not contracts until implemented, tested, and recorded here.
