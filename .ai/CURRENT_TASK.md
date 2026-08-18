# Current Task

## ID

LS-005

## Title

Loopback Direct Play with HTTP Range

## Status

Completed

## Goal

Stream a persisted media file through the embedded loopback server by opaque media ID, using bounded I/O and correct single-range HTTP responses.

## Acceptance Criteria

- `GET /api/v1/media/{id}/stream` resolves only persisted media IDs from the current approved library.
- Raw filesystem paths never appear in URLs, response bodies, or errors.
- The core revalidates that the resolved file remains inside the approved library before opening it.
- Full responses stream with `200 OK`; valid single byte ranges stream with `206 Partial Content`.
- Unsatisfiable or malformed ranges return `416 Range Not Satisfiable` with `Content-Range: bytes */{size}`.
- File bodies use bounded streaming I/O and are never loaded fully into memory.
- Missing/unavailable media returns a stable safe error without internal details.
- The server remains loopback-only; LAN exposure and authentication remain out of scope.
- Contract, range parsing, containment, concurrency, and response-body tests pass.
- API, security, status, test matrix, and source documentation are current.

## Completed

- Added current-library opaque-ID-to-location lookup behind the core boundary.
- Added canonical root/file containment validation immediately before asynchronous file opening.
- Added a streaming domain with safe content types and single byte-range normalization.
- Added full `200`, partial `206`, invalid `416`, missing `404`, and capacity `503` behavior with stable safe errors.
- Added bounded Tokio file response bodies and an eight-stream core-owned concurrency limit.
- Added range, containment, response contract, path non-disclosure, and capacity tests.
- Updated the API, security model, architecture map, test matrix, source READMEs, and project status.

## Tests Last Executed

- `npm run verify` — PASS; format, lint, typecheck, 3 files / 8 frontend tests, and production build passed.
- `cargo fmt --all --check` — PASS.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — PASS.
- `cargo test --workspace` — PASS; 20 core/workspace tests passed.
- `cargo check --workspace` — PASS.
- `git diff --check` — PASS.

## Tests Not Yet Executed

- Interactive playback in a real browser/video element.
- Large-file and long-running stream soak tests.
- LAN binding, authentication/pairing, multipart ranges, conditional requests, transcoding, and non-Windows platforms.

## Known Problems

- None confirmed for the LS-005 scope.

## Assumptions

- Only one byte range per request is supported; multipart ranges are rejected with `416`.
- MIME type is derived from the scanner's supported extension allowlist.
- Streaming remains loopback-only until pairing/authentication protects LAN access.

## Next Exact Step

Define LS-006 for a Vue playback UI that builds opaque-ID stream URLs from the reported loopback server address, or explicitly prioritize pairing/authentication foundations first.
