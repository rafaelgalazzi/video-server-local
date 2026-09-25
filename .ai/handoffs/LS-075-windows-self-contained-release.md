# LS-075 Handoff — Single-file Windows portable release

## Status

Completed on 2026-09-25. The final diff remains uncommitted; preserve all working-tree changes.

## Objective

Replace the raw executable distribution with one copy-and-run Windows x64 `LocalStream.exe` that carries the remote browser resources and FFmpeg/ffprobe without an installer or target-computer development tools, while preserving explicit LAN activation and certificate trust.

## Root Cause Identified

`src-tauri/tauri.conf.json` had `bundle.active: false`. The desktop frontend was embedded, so the native window opened, but the remote HTTPS listener reads separately packaged browser assets, which were not present beside a copied raw executable. FFmpeg/ffprobe were also resolved only from environment/`PATH`; the development computer happened to have them installed through WinGet.

LAN configuration is correctly disabled by default. Even with a portable payload, the user must select a private address, save it, restart, verify/install the root certificate, and approve pairing. Windows Firewall can still block `LocalStream.exe` and is not changed silently.

## Architecture

- The framework remains Vue 3 + Tauri 2 + Rust. Electron was not introduced.
- `localstream-portable-payload` is a framework-independent Rust crate that creates and activates the bounded tar payload and integrity trailer.
- `tools/portable-packager` is a separate Cargo package. Keeping it outside the Tauri package prevents Cargo auto-bin discovery from changing the application target.
- `npm run release:windows` pins `--target x86_64-pc-windows-msvc` for both the Tauri build and the packager, so a stale or ARM64 host binary can never be packaged. The packager also validates the PE machine and optional-header magic before appending.
- The Tauri adapter activates the current executable's payload under the normal app-data directory, passes the extracted absolute tool paths to the core, and serves remote assets from `<portable-root>/web`.
- Portable activation failure keeps trusted-local desktop startup, disables packaged media/LAN assets with `open_with_disabled_media_tools` instead of consulting machine executables, and reports a safe failure code in the native footer.
- The selected smaller profile still requires Microsoft Edge WebView2 Runtime.

## Payload Controls

- Trailer: base range size, payload range size, SHA-256 of the base range, and SHA-256 of the payload range.
- SHA-256 manifest for every extracted resource, read from the embedded payload rather than the mutable extracted copy.
- Allowlist: exact `ffmpeg.exe` / `ffprobe.exe`, `portable-manifest.sha256`, `web/**`, and `licenses/ffmpeg/**`; build inputs must be regular files and the required set must be complete.
- Rejects traversal, absolute/alternate-separator paths, duplicate entries, non-regular entries, symlinks/junctions, more than 2,048 entries, individual files above 512 MiB, and payloads above 1 GiB.
- Build sources are hashed before archiving and rehashed afterwards, so a mid-build change fails the release instead of producing a mismatched manifest.
- Process-specific staging plus atomic rename into a full-digest directory; existing extractions are reverified against the embedded manifest on every launch and repaired atomically.
- SHA-256 is corruption detection, not a publisher signature. Signing remains LS-055 work.
- Raw `target/x86_64-pc-windows-msvc/release/localstream-app.exe` is a build input, not a distributable; nested portable builds are rejected.

## Pinned Media Distribution

- Version: FFmpeg `9.0.2` release essentials.
- Download: `https://www.gyan.dev/ffmpeg/builds/packages/ffmpeg-9.0.2-essentials_build.zip`
- SHA-256: `60f467265b1e312373dbcd92200c2618a74850f98d3d078e94296bb3fa2047ba`
- FFmpeg source revision advertised by the publisher: `946fcce07b`.
- License: GPLv3 build; the archive's exact `LICENSE` and `README.txt` are included in the payload.
- Build preparation requires PowerShell and first-build access to the archive URL. Runtime never downloads resources.

## Verified

- `npm run media-tools:prepare` — PASS; prepared verified FFmpeg/ffprobe 9.0.2 inputs.
- `npm run verify` — PASS; formatting, lint, typecheck, 50 frontend tests, and production build.
- `cargo fmt --all -- --check` — PASS.
- `cargo check --workspace --all-targets` — PASS.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — PASS.
- `cargo test --workspace` — PASS; 125 unit tests, including verified first extraction/reuse, rewritten-local-manifest repair, modified base rejection, malformed ranges, nested-build rejection, symlink rejection with a valid outer hash, tampered trailer rejection, and path allowlisting.
- `npm run release:windows` — PASS; produced `target/release/portable/LocalStream.exe` at 227,000,920 bytes (216.48 MiB) with a 211,453,440-byte payload from a 15,547,392-byte x64 PE base.
- Final executable SHA-256: `AE05B1330131F3557B9091C521EDE9F6D44DB99BEA8ED46824F3279894802DB3`; payload SHA-256 `85abaf1ecc389fc97955529db48d52419d27dcf41aa7a6ef7014e54133a6c78a`.
- Copy-only runtime test — PASS: copied only `LocalStream.exe` to an empty approved-temp directory, launched with `PATH` reduced to Windows system directories, and verified:
  - the application remained running and was LAN-ready in 4.1 s on first extraction;
  - FFmpeg/ffprobe 9.0.2 executed from the extracted full-digest directory;
  - the configured private HTTPS listener was active on the machine's current private address and port;
  - `/api/v1/health` returned LocalStream v1 with `lanAvailable: true`;
  - `/` returned the LocalStream browser UI;
  - a second launch reused the verified extraction and was LAN-ready in 2.4 s.
- The temporary test executable, extracted payload, and abandoned installer test were removed after verification, and the pre-existing LAN configuration file was restored byte-for-byte.

## Not Yet Verified

- A second physical Windows computer, especially WebView2-missing behavior, firewall prompt/rule, certificate trust, pairing, and playback.
- Old content-addressed payload cleanup/update UX.
- Signing and non-Windows portable packages.

## Next Exact Action

Resume LS-032 and draft the discovery protocol ADR. LS-055 retains physical machine, lifecycle, signing, and cleanup qualification.
