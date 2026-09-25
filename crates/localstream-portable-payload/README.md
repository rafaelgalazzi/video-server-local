# Portable Payload

## Purpose

Create and activate the bounded resource payload appended to a single-file Windows portable executable.

## Public Interfaces

- `create_portable_executable`: validates a Windows x64 PE base executable, copies it, and appends a deterministic tar payload plus base/payload ranges and SHA-256 hashes.
- `activate`: verifies both range hashes and the manifest read from the embedded payload, safely extracts allowlisted files, repairs changed extractions, and returns the full-digest runtime directory.
- `PortableResource`, `PortableBuildSummary`, and `PortableError`: typed build and runtime contracts.

## Dependencies

The standard library, `tar`, `sha2`, and `thiserror`; tests use `tempfile`.

## Security and Limits

The format accepts only regular files under the exact FFmpeg/ffprobe names, `web/`, and `licenses/ffmpeg/`. It rejects absolute/parent paths, alternate separators, duplicate entries, unsupported entry types, symlinks/junctions, excessive counts, files above 512 MiB, and total payloads above 1 GiB. Build inputs must be regular files, required resources must be present, and sources are rehashed after archiving to detect mid-build changes. Extraction is staged and atomically renamed under an application-owned directory. Later launches compare both the extracted files and local manifest copy against the manifest embedded in the verified payload.

SHA-256 detects corruption and modification; it is not a publisher signature or authenticity proof.

## Current Limitations

The initial policy is Windows x64 and uses `.exe` media-tool names. Updating a portable executable creates a new content-addressed runtime directory; old versions are not automatically deleted because they may be in use.

## Planned Work

LS-055 may add explicit portable cleanup/update UX and clean-machine qualification. The archive format must remain backward-compatible or provide migration before an incompatible change.
