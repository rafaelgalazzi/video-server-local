# Current Task

## ID

LS-024

## Title

Same-origin browser UI hosting

## Status

Completed

## Goal

Serve an explicitly configured production Vue build from the dormant authenticated HTTPS router without changing desktop startup or enabling LAN binding.

## Acceptance Criteria

- The HTTPS router serves `index.html` and immutable production assets with correct content types.
- Client-side navigation falls back to `index.html`.
- `/api/` paths never use the SPA fallback, including unknown API routes.
- Static paths reject traversal, encoded separators, malformed encoding, and non-file asset paths safely.
- Static file reads are bounded and do not expose filesystem paths in errors.
- HTML uses revalidation/no-store policy; fingerprinted assets use long-lived immutable caching.
- Public UI assets do not weaken library/media authentication or pairing origin checks.
- The active desktop HTTP listener and HTTPS loopback-only lifecycle remain unchanged.
- Focused route/security tests and full affected-stack checks pass.

## Relevant Files

- `crates/localstream-core/src/server/mod.rs`
- `crates/localstream-core/src/server/README.md`
- `docs/api/README.md`
- `docs/security/README.md`
- `.ai/PROJECT_STATUS.md`

## Completed

- LS-023 strict HTTPS origin policy and transport resource limits.
- Added explicit validated production asset roots with an 8 MiB per-file limit and containment checks.
- Added public HTML/assets, correct content types, safe cache headers, HEAD support, and SPA fallback.
- Reserved `/api` and `/api/*` from fallback while preserving API authentication and pairing policy.
- Added an opt-in asset-aware loopback HTTPS lifecycle without changing desktop startup or LAN exposure.
- Added focused router/security/lifecycle coverage and verified the current Vite production build.

## In Progress

- Nothing.

## Remaining

- Nothing for LS-024.

## Assumptions

- Packaged-asset path resolution belongs to a future lifecycle/platform adapter; the reusable core receives an explicit validated asset root.
- Static files are capped at 8 MiB each; current Vite production output is substantially smaller.
- Asset filenames under `/assets/` are content-fingerprinted by Vite and may use immutable caching; HTML and SPA fallbacks must revalidate.

## Next Exact Step

Start LS-025 from `.ai/IMPLEMENTATION_ROADMAP.md`: select the same-origin HTTPS API in a remote browser while retaining Tauri commands and the trusted-local loopback desktop flow.
