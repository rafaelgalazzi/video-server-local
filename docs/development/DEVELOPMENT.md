# Development

The frontend and native project foundation is scaffolded. Commands are marked when they could not be verified in the current environment.

## Prerequisites

- Node.js 22.13 or newer and npm. The 2026-09-25 frontend verification used Node.js 24.13.0.
- Rust stable meeting the workspace MSRV in `Cargo.toml`.
- Current [Tauri 2 system prerequisites](https://v2.tauri.app/start/prerequisites/) for the target OS.
- Windows portable release builds additionally require PowerShell and temporary Internet access on the first build to fetch the checksum-verified FFmpeg archive; the resulting application does not download resources at runtime.

## Installing Dependencies

Run `npm install` from the repository root. `package-lock.json` is canonical. Cargo resolves Rust dependencies from the workspace manifests; commit `Cargo.lock` after Cargo first generates it.

## Starting Vue

Run `npm run dev`. The Vite preview listens on `http://localhost:1420`; it cannot invoke native commands and shows a non-fatal preview state.

## Starting Tauri

Run `npm run tauri dev`. This was verified on Windows on 2026-08-18.

## Running Rust Tests

Run `cargo test --workspace`. Also use `cargo test -p localstream-core` for targeted core tests. The workspace command passed on Windows on 2026-08-18.

## Running Frontend Tests

Run `npm run test` for Vitest once or `npm run test:watch` during development.

## Type Checking

Run `npm run typecheck`.

## Linting

Run `npm run lint`.

## Formatting

Run `npm run format` to write frontend and documentation formatting, or `npm run format:check` to check it. Run `cargo fmt --all --check` for Rust.

## Building

Run `npm run build` for the frontend production bundle. `npm run tauri build` still produces the unbundled native executable and is not a portable release; bundling remains disabled in the base configuration.

For the single-file Windows x64 portable executable, run:

```powershell
npm run release:windows
```

The command builds the raw Tauri executable, downloads the pinned FFmpeg 9.0.2 essentials archive into `target/media-tools`, verifies its SHA-256 digest and required media features, and invokes the repository-local packager. The result is `target/release/portable/LocalStream.exe`; no setup executable is produced.

On first launch, `LocalStream.exe` verifies the base and payload hashes plus the embedded manifest, then atomically extracts its browser UI, FFmpeg/ffprobe, and exact GPL notices into a full-digest directory under the normal LocalStream application-data folder. Later launches verify that extraction against the embedded manifest before use and repair it atomically if it changed. The verified portable file is 227,000,920 bytes (216.48 MiB) and requires Microsoft Edge WebView2 Runtime on Windows, but not an installer, Internet access, Node.js, Rust, SQLite, FFmpeg, ffprobe, or a Visual C++ redistributable. A portable activation failure is reported in the native footer and disables packaged media/LAN assets rather than falling back to machine executables.

Do not distribute the raw `target/x86_64-pc-windows-msvc/release/localstream-app.exe`. The canonical command pins that target triple so the packager cannot consume a stale host or ARM64 binary. The raw executable does not contain the browser UI or media tools required by the secure remote listener and media fallback. The pinned media source is Gyan's FFmpeg `9.0.2` release-essentials archive at `https://www.gyan.dev/ffmpeg/builds/packages/ffmpeg-9.0.2-essentials_build.zip`, SHA-256 `60f467265b1e312373dbcd92200c2618a74850f98d3d078e94296bb3fa2047ba`, corresponding to FFmpeg source revision `946fcce07b`. The packager includes the archive's exact GPL license and build README in the portable payload. See [ADR-0010](../architecture/adr/0010-self-contained-windows-release.md) for the packaging and licensing decision.

## Connecting Another Device

LAN serving remains disabled on every new computer. In **Network**, select one detected private address, keep the default HTTPS port unless it is unavailable, save the configuration, and restart LocalStream. Export the node root from **Access**, compare its complete fingerprint, and install it into the other device's trusted-root store. Open the exact active endpoint shown by LocalStream, then approve the browser's pairing code.

If Windows Firewall blocks `LocalStream.exe`, review its private-network rule on the computer hosting the server. Do not bypass the certificate warning or bind a wildcard/public address.

## Canonical Verification

Run `npm run verify` for frontend format checking, linting, type checking, unit tests, and a production build. It passed with 50 tests on 2026-09-25.

Run the Rust gates separately:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

These Rust commands passed on Windows with Rust 1.98.1 on 2026-09-25, including 125 unit tests across the application, core, and portable-payload crates.

## Troubleshooting

- If Vitest/esbuild reports access denied while resolving `vitest.config.ts`, rerun verification with the workspace execution permission required by the host sandbox.
- If a documented command is absent, treat the documentation as stale and update it with the implementation.
- Run commands from the repository root unless a directory README says otherwise.
- Do not delete lockfiles or uncommitted work to resolve dependency problems.
- Record unresolved failures in the active task and handoff, including the exact command and output summary.
