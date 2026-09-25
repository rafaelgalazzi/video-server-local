# Scripts

Repository automation has one maintained entry point:

- `prepare-windows-media-tools.ps1` prepares the ignored Windows x64 FFmpeg and ffprobe inputs for the portable packager. Run it through `npm run media-tools:prepare`, or use the canonical `npm run release:windows` command.

The preparation script requires Windows PowerShell. It downloads FFmpeg 9.0.2 release-essentials once into `target/media-tools`, verifies the archive SHA-256, checks the required x264/AAC encoders and subtitle filters, verifies both executable identities, and copies the exact license/build-information files into payload staging. Build directories and files are checked for Windows reparse points, and the script does not recursively delete a pre-existing linked path. Re-running the script revalidates and recreates the inputs from the cached verified archive.

The Rust implementation in `crates/localstream-portable-payload` and `tools/portable-packager` creates and activates the final `LocalStream.exe`; no PowerShell script constructs the portable executable itself.

A future verification entry point may coordinate frontend formatting, linting, type checking, tests, Rust formatting, Clippy, tests, and build checks after those tools and manifests exist. Do not add a placeholder script that reports success without performing real checks.
