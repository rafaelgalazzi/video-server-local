# LS-014 Handoff — Node identity startup and trusted-local display

## Objective

Initialize exactly one protected node identity during desktop startup and expose only its safe public summary through the local Tauri UI.

## State

Completed on 2026-08-18. Changes remain uncommitted because the repository has no commits and all files are untracked.

## Implemented

- Desktop setup opens the platform keyring entry `desktop-default` and loads or creates the node identity before starting the loopback HTTP server.
- Startup fails closed if protected storage or identity restoration fails.
- Only a cloned `NodeIdentitySummary` is retained as Tauri managed state.
- Thin `node_identity` command exposes the node ID and full SPKI fingerprint without key or certificate material.
- `useNodeIdentity` owns typed loading, status, stale-data clearing, and contained preview errors.
- `NodeIdentityPanel` presents the public identity and fingerprint-comparison guidance on the trusted desktop.

## Changed Files

- `src-tauri/src/lib.rs`
- `src-tauri/README.md`
- `src-tauri/src/README.md`
- `src/App.vue`
- `src/styles.css`
- `src/composables/useNodeIdentity.ts`
- `src/composables/useNodeIdentity.test.ts`
- `src/components/NodeIdentityPanel.vue`
- `.ai/CURRENT_TASK.md`
- `.ai/PROJECT_STATUS.md`

## Verified

- `npm run verify` — PASS outside the restricted filesystem sandbox; format, lint, typecheck, 23 tests across 7 files, and production build.
- `cargo fmt --all` — PASS.
- `cargo test --workspace --locked` — PASS; 36 Rust tests.

## Not Verified

- Interactive startup against Windows Credential Manager.
- Apple Keychain, Linux Secret Service, mobile, or headless startup.
- TLS, certificate trust installation, remote pairing, or LAN behavior.

## Security-Critical Notes

- The managed Tauri state intentionally contains only `NodeIdentitySummary`; do not retain or expose `NodeIdentity` there.
- Keyring failure aborts setup and must never fall back to a plaintext or ephemeral identity.
- The existing loopback HTTP listener remains separate and unchanged.

## Next Exact Action

Implement LS-015 explicit identity reset with all-peer revocation and confirmation. Reset must fail closed and must not regenerate identity in-process.
