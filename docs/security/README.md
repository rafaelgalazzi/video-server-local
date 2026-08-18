# Security Model

This document states requirements, not verified implementation. No security-sensitive application code exists yet.

## Trust and Pairing

- LAN location does not imply trust; unpaired peers are untrusted.
- Pairing must require explicit user approval and establish revocable credentials.
- Public APIs are expected to authenticate requests according to the eventual pairing design. Exact endpoints and token storage remain to be designed and documented before implementation.

## Filesystem Exposure

- Only explicitly user-approved libraries may expose media.
- Raw paths are private and must never appear in public API payloads or URLs.
- Clients use opaque media identifiers resolved inside the trusted core.
- Canonicalization and containment checks must prevent directory traversal, symlink escapes where applicable, and identifier manipulation.

## Processes and Resources

- Invoke FFmpeg with structured argument APIs, never unsafe shell interpolation. Treat media and metadata as untrusted input.
- Stream using bounded I/O; never load entire media files into memory.
- Put concurrency, timeout, cancellation, request-size, and rate limits around expensive scans, streams, probes, and transcodes.
- Network failures and inaccessible files must be contained rather than crashing the process or exposing unrelated data.

## Privacy

Primary LAN features must work without Internet access. External metadata or remote-access features, if later adopted, must be explicit, opt-in, and documented. Logs must not leak paths, credentials, pairing secrets, or sensitive media metadata unnecessarily.

Security behavior must receive unit/integration tests and threat-focused review as it is implemented. Update this file with the actual authentication and pairing model before exposing a LAN API.
