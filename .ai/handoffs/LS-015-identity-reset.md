# LS-015 Handoff — Identity reset and trust revocation

## Objective

Provide an explicit trusted-local identity reset that invalidates existing trust before removing protected root material.

## State

Completed on 2026-08-18. Changes remain uncommitted in the repository's existing working tree.

## Implemented

- `NodeSecretStore::delete` extends the protected-storage boundary; the keyring adapter treats an already-missing entry as an idempotent success and all other failures as unavailable.
- `LocalStreamCore::reset_node_identity` bulk-revokes active peers before protected-root deletion.
- Store deletion failure is fail-closed: all credentials remain revoked and no replacement identity is generated.
- Desktop setup retains the keyring adapter as managed state; the thin reset command returns only the number of revoked peers.
- The Vue flow requires a separate confirmation, explains that every device must pair again, supports cancellation, retains state on failure, and requires restart after success.

## Verified

- `npm run verify` — PASS; formatting, lint, typecheck, 25 tests across 7 files, and production build.
- `cargo test --workspace --locked` — PASS; 38 Rust tests.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` — PASS.
- `cargo fmt --all` — PASS.

## Not Verified

- Real Windows Credential Manager deletion or other platform stores.
- Interactive desktop reset/restart behavior.
- Browser sessions do not exist yet; reset currently revokes all persisted peer credentials only.

## Next Exact Action

Implement LS-016 short-lived leaf issuance from `NodeIdentity`, validating DNS/IP subject alternative names and keeping private leaf material inside the core TLS boundary.
