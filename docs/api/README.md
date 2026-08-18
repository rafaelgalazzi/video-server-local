# LocalStream API

## Implemented

No API routes or server implementation exist.

## Planned Convention

Versioned REST endpoints should use the `/api/v1/` prefix. Streaming and event endpoints must be documented here when their contracts are implemented. HTTP handlers must be thin adapters to reusable Rust services.

Do not expose raw filesystem paths. Public media access must use opaque identifiers, be limited to explicitly approved libraries, enforce the pairing/authentication model, prevent traversal, and use bounded streaming I/O.

Planned routes in product documents are not contracts until they are implemented, tested, and recorded here.
