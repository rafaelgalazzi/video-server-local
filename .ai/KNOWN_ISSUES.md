# Known Issues

This file is exclusively for confirmed or strongly identified bugs. Architectural improvements belong in `.ai/TECH_DEBT.md`; unimplemented features belong in `.ai/PROJECT_STATUS.md`.

## KI-001 — Raw executable omits remote-browser resources and media tools

Status: Fixed

Related task: LS-075

Affected area: Windows release packaging and LAN browser startup.

Description: Copying only the executable produced with base Tauri configuration opens the native desktop shell but does not create a portable release. The secure LAN listener expects separately bundled `resources/web`, and media operations discover FFmpeg/ffprobe from the machine environment.

Reproduction: Build with the base `bundle.active: false` configuration, copy only the release executable to another Windows user account, and attempt to enable/use the LAN browser or media fallback. The target does not have the developer account's `PATH` entries or `resources/web` directory.

Expected: A supported distribution contains every application-owned runtime file and requires no separately installed media tool.

Actual: The raw executable could launch the native UI while remote browser assets and FFmpeg/ffprobe were absent. LS-075 now builds a separate portable `LocalStream.exe` with a verified embedded payload; it was launched from an otherwise empty directory with FFmpeg removed from `PATH`, and its extracted tools plus LAN browser UI were verified. Extracted files are checked against the manifest embedded in the payload rather than a mutable local manifest, and a second physical Windows computer remains unverified.

## Entry Template

```md
## KI-001 — Short description

Status: Open / Investigating / Fixed / Deferred

Related task: LS-XXX

Affected area: ...

Description: ...

Reproduction: ...

Expected: ...

Actual: ...
```
