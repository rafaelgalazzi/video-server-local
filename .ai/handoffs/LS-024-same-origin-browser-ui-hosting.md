# LS-024 Handoff — Same-origin browser UI hosting

## Objective

Serve the production Vue application from the dormant HTTPS surface with safe SPA fallback and static-file policy, without changing desktop startup or enabling LAN binding.

## State

Completed on 2026-08-18. Changes remain uncommitted in the existing working tree.

## Implemented

- `BrowserAssets::from_directory` validates an explicit canonical web root and required `index.html`.
- Static reads re-canonicalize the selected file, require containment and a regular file, and cap each response at 8 MiB.
- The asset-aware encrypted router serves public HTML, JavaScript, CSS, images, and fonts with `nosniff` and explicit content types.
- Vite `/assets/` files receive one-year immutable caching; HTML and SPA fallbacks receive `no-cache, no-store, must-revalidate`.
- Missing client-side navigation falls back to `index.html`; missing `/assets/*`, `/api`, and `/api/*` never do.
- Traversal, encoded separators, invalid UTF-8/percent encoding, directories, symlink escapes, and oversized files fail without path disclosure.
- GET and HEAD are supported; other static methods fail closed.
- UI files are public for onboarding, while library/media APIs retain bearer or secure-session authentication and pairing routes retain Host/Origin policy.
- `start_loopback_https_server_with_assets` opts into the composed surface. Existing desktop and foundation lifecycle functions are unchanged and remain loopback-only.

## Verified

- `cargo test -p localstream-core browser_assets --offline -- --nocapture` — PASS; 3 focused route/security tests.
- `cargo test --workspace --offline` — PASS; 68 Rust tests.
- `cargo clippy --workspace --all-targets --offline -- -D warnings` — PASS.
- `cargo fmt --all -- --check` — PASS after formatting.
- `npm run verify` — PASS outside the restricted filesystem sandbox; 27 frontend tests and production Vite build.
- `git diff --check` — PASS.

## Not Verified

- Packaged/install-time asset path resolution; this remains for lifecycle and packaging work.
- Non-loopback or LAN operation; intentionally not enabled.
- Remote-browser bootstrap/pairing states; LS-025.
- Unsafe cookie-authenticated browser methods; none exist, so no CSRF token is required yet.

## Next Exact Action

Start LS-025 remote-browser application bootstrap, selecting same-origin HTTPS API behavior in browser mode while retaining the existing Tauri and trusted-local desktop flow.
