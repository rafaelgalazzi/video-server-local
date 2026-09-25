# ADR-0010 — Single-file Windows portable release

## Status

Accepted

## Context

The native application was built with Tauri bundling disabled. A raw `target/release` executable embeds the desktop frontend, but it does not contain the separately served browser resources under `resources/web` and it does not contain FFmpeg or ffprobe. The core therefore depended on media tools installed in the build user's `PATH`, and an enabled LAN listener could fail closed when the remote browser assets were absent.

The requested Windows distribution is one portable executable with no setup or installer. Tauri still uses Microsoft Edge WebView2 on Windows, and bundling a private WebView2 runtime inside the same file would require a much larger self-extracting distribution and manual runtime security updates. The user selected the smaller portable profile, which permits an installed WebView2 Runtime but does not permit separately installed Node.js, Rust, SQLite, FFmpeg, or ffprobe.

## Decision

- Keep Vue 3, Tauri 2, and Rust. Do not migrate to Electron or add another browser runtime.
- Distribute Windows x64 as `target/release/portable/LocalStream.exe`, built by the canonical `npm run release:windows` command.
- Append a bounded tar payload and a trailer containing the base range, payload range, SHA-256 of both ranges, and a per-file manifest to the raw Tauri executable. The payload contains the remote browser UI, checksum-pinned FFmpeg/ffprobe executables, and the exact FFmpeg license/build-information files.
- On startup, the desktop adapter verifies the trailer, both range hashes, and the manifest read from the embedded payload before trusting an existing extraction. SHA-256 provides corruption detection, not publisher authenticity; signing remains a separate release gate.
- Extraction accepts only regular allowlisted files, rejects traversal/duplicates/symlinks, enforces entry and total-size limits, stages into a process-specific directory, verifies every extracted file against the embedded manifest, and atomically publishes a full-digest runtime directory under Tauri application data.
- Pass the extracted absolute FFmpeg/ffprobe paths into the reusable core. Core operations retain the bounded structured process boundary and executable identity check. Development and non-portable launchers retain environment-variable and `PATH` discovery.
- On later launches, verify the existing content-addressed extraction and repair it atomically if files are missing or changed. Do not automatically delete old content-addressed versions because another process may still be using them.
- If portable activation fails, retain trusted-local desktop startup, disable packaged media and LAN browser assets without falling back to machine executables, and expose a safe failure code in the native UI.
- Do not enable LAN serving automatically. The portable executable does not bypass certificate warnings, install trust, or change Windows Firewall. The user must select a concrete private address, save it, restart, verify and install the root certificate on the client, and approve pairing.
- Do not distribute the raw `target/x86_64-pc-windows-msvc/release/localstream-app.exe`; it is only the build input.

## Alternatives Considered

- Electron: rejected because it adds a second large Chromium runtime and does not improve the selected Tauri/WebView2 profile.
- NSIS or another setup executable: rejected because the requested artifact must run by copying one file.
- A raw Tauri executable: rejected because it omits separately served browser resources and packaged media tools.
- Embedding a fixed WebView2 runtime in the portable file: deferred because it would add roughly 250 MB, require manual security servicing, and exceed the user's selected smaller portable profile.
- Runtime downloads: rejected because the portable application must not depend on Internet access.
- Linking FFmpeg through FFI: unchanged and rejected for this task by ADR-0008.

## Consequences

`LocalStream.exe` is 216.48 MiB and needs roughly 211 MiB of additional application-data storage after first launch. First launch performs base/payload hashing, manifest reading, and extraction; later launches verify the extracted copy against the embedded manifest before use. Updating the executable creates a new full-digest runtime directory, so old portable payload directories require explicit future cleanup UX.

A target computer needs Windows 10/11 x64 and Microsoft Edge WebView2 Runtime, but does not need an installer, Internet access, Node.js, Rust, SQLite, FFmpeg, ffprobe, or a Visual C++ redistributable. Windows firewall approval and explicit certificate trust remain visible operating-system steps.

The bundled FFmpeg build is GPLv3 because it includes libx264 and other GPL components. Release operators must preserve the exact notices/source information and complete legal review before public or commercial distribution. Clean-machine portability, firewall behavior, physical second-device trust, and playback remain part of broader LS-055 qualification.
