# Portable Executable Packager

## Purpose

Create `target/release/portable/LocalStream.exe` by appending a verified, bounded resource payload to the raw Tauri Windows x64 executable.

## Inputs

- `target/x86_64-pc-windows-msvc/release/localstream-app.exe`: native Tauri application; its PE header must confirm x64.
- `dist/`: browser UI, stored as `web/` in the portable payload.
- `src-tauri/binaries/ffmpeg-x86_64-pc-windows-msvc.exe` and the matching ffprobe executable.
- `src-tauri/generated/ffmpeg/`: exact license and build-information files prepared by the media-tool script.

## Interface and Security

The packager delegates archive creation to the framework-independent `localstream-portable-payload` crate. That module allowlists resource paths, records per-file SHA-256 values, hashes the complete payload, bounds entry count/size, rejects symlinks and traversal, and writes atomically through a partial output.

Run it through the canonical `npm run release:windows` command rather than directly; the raw Tauri binary and prepared resources must exist first.
