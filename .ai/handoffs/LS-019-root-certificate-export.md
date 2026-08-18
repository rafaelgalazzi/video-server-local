# LS-019 Handoff — Root certificate export and trust guidance

## Objective

Export the node's public root certificate from the trusted desktop with fingerprint-based trust guidance and no automatic installation or remote download.

## State

Completed on 2026-08-18. Changes remain uncommitted in the existing working tree.

## Implemented

- `NodeIdentityService::load_existing` fails if protected identity is missing and never creates a replacement.
- A reference-store implementation allows safe existing-only reload from managed platform storage.
- The Tauri export command verifies the reloaded summary matches startup-managed identity, opens a native `.cer`/`.der` save dialog, and writes only root certificate DER.
- Cancellation returns `false`; success returns `true`; no bytes, destination path, keyring details, or private material cross to Vue.
- The Vue flow requires an explicit button action and contains success, cancellation, and failure states.
- Trust guidance explains full fingerprint comparison, the authority granted by installation, removal after reset/compromise, and the absence of automatic installation.

## Verified

- `npm run verify` — PASS; formatting, lint, typecheck, 27 tests across 7 files, and production build.
- `cargo test --workspace --locked` — PASS; 47 Rust tests.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` — PASS.
- Rust and frontend formatting — PASS.

## Not Verified

- Interactive native save dialog and filesystem write.
- Actual certificate installation in Windows/macOS/Linux or browsers; installation is intentionally manual.
- Real platform protected-store operation.

## Next Exact Action

Implement LS-020 as a reusable in-memory pairing-attempt limiter with per-source and global bounds, monotonic windows, safe retry metadata, stale-entry cleanup, and deterministic tests before exposing pairing routes.
