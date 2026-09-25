# Generated Windows Sidecars

`scripts/prepare-windows-media-tools.ps1` places the verified Windows FFmpeg and ffprobe executables in this directory during a portable release build.

The files are intentionally not committed. Run `npm run release:windows`; the preparation step downloads the pinned archive, verifies its SHA-256 digest, checks the required encoders and filters, and writes target-suffixed inputs for the portable packager here.
