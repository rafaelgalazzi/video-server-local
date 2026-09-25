# Current Task

## ID

LS-075

## Title

Single-file Windows portable release baseline

## Status

Completed

## Goal

Produce one copy-and-run Windows x64 `LocalStream.exe` that carries the remote browser resources and FFmpeg/ffprobe without an installer or target-computer development tools, while preserving explicit LAN activation and certificate trust.

## Completed

- Confirmed the raw Tauri executable was not a portable release: base bundling was disabled, remote browser assets were not packaged, and media operations resolved FFmpeg/ffprobe from the machine environment.
- Kept Vue 3 + Tauri 2 + Rust; Electron was explicitly rejected because it would add a second browser runtime.
- Added a framework-independent, bounded portable payload format with a content hash, per-file manifest, allowlisted paths, atomic extraction, tamper detection, and content-addressed application-data storage.
- Added a separate repository-local packager so the Tauri package still has exactly one native binary target.
- Added a checksum-verified FFmpeg 9.0.2 essentials preparation script, required encoder/filter checks, and exact license/build-information staging.
- Added explicit absolute media-tool paths to the reusable core and verified portable-root discovery at the thin Tauri adapter.
- Expanded first-run connection, certificate trust, pairing, endpoint, no-address, and firewall guidance without enabling LAN automatically.
- Built `target/release/portable/LocalStream.exe` and tested the executable by itself with FFmpeg removed from `PATH`; first-run extraction, extracted tools, private HTTPS health, and browser UI were verified.

## Verification

- `npm run media-tools:prepare` — PASS; prepared verified FFmpeg/ffprobe 9.0.2 inputs.
- `npm run verify` — PASS; formatting, lint, typecheck, 50 frontend tests, and production build.
- `cargo fmt --all -- --check` — PASS.
- `cargo check --workspace --all-targets` — PASS.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — PASS.
- `cargo test --workspace` — PASS; 125 unit tests.
- `npm run release:windows` — PASS; produced a 227,000,920-byte (216.48 MiB) `LocalStream.exe` with a 211,453,440-byte payload.
- Copy-only launch with a stripped `PATH` — PASS; extracted tools executed, private HTTPS health returned `lanAvailable: true`, and the browser UI was served.
- Final executable SHA-256: `AE05B1330131F3557B9091C521EDE9F6D44DB99BEA8ED46824F3279894802DB3`; payload SHA-256 `85abaf1ecc389fc97955529db48d52419d27dcf41aa7a6ef7014e54133a6c78a`.

## Follow-Up Work

- A second physical Windows computer, WebView2-missing behavior, firewall UX, certificate trust, pairing, and playback remain under LS-055 qualification.
- Old content-addressed payload directories need explicit cleanup/update UX before broad public release.
- Signing and non-Windows portable packages are not implemented.
- Resume Phase B with LS-032.

## Assumptions

- “Single EXE” means one copy-and-run portable file, not a setup executable. It necessarily extracts application-owned resources because Tauri cannot execute sidecars and serve web files from an appended payload directly.
- The selected smaller profile requires Microsoft Edge WebView2 Runtime. It is preinstalled on Windows 11 and most supported Windows 10 systems but remains an explicit platform prerequisite.
- The initial portable baseline targets Windows x64. It does not silently claim ARM64, 32-bit, Linux, or macOS support.
- The pinned Gyan essentials build is GPLv3 and includes libx264/libass. Exact notices and source/build information travel in the payload, but public/commercial distribution still requires legal review.
- LAN access remains disabled by default; certificate trust and firewall approval are explicit user actions.

## Next Exact Step

Resume LS-032 and draft the discovery protocol ADR.
