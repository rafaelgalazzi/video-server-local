# LS-012 Handoff — Encrypted LAN transport design

## Objective

Define an offline-capable authenticated TLS identity, client trust model, browser origin, credential transport, and implementation gates before exposing LocalStream beyond loopback.

## State

Completed on 2026-08-18. LS-012 is documentation-only and all changes are uncommitted at handoff time; preserve them.

## Implemented

- ADR-0007 selects a persistent per-node private CA with renewable hostname/IP leaf certificates.
- Native clients will pin a verified node-root public-key fingerprint.
- Browser devices require explicit root-certificate trust installation and trusted-local fingerprint comparison; certificate-warning bypass is prohibited.
- Browser assets, APIs, pairing, and media will share one HTTPS origin.
- Browser pairing will establish a revocable `HttpOnly`, `Secure`, `SameSite=Strict` cookie so native `<video>` requests work without JavaScript-visible bearer tokens or signed URLs.
- Unsafe cookie-authenticated methods require Origin and CSRF validation.
- The desktop loopback listener remains distinct from the future fail-closed HTTPS LAN listener.
- ADR-0006 and ADR-0007 gates remain mandatory before any LAN binding.

## Changed Files

- `docs/architecture/adr/0007-private-pki-and-https-origin.md`
- `docs/architecture/adr/README.md`
- `docs/architecture/ARCHITECTURE_MAP.md`
- `docs/security/README.md`
- `docs/api/README.md`
- `.ai/DEFERRED_DECISIONS.md`
- `.ai/PROJECT_STATUS.md`
- `.ai/CURRENT_TASK.md`

## Verified

- `npx prettier --write ...` — PASS; all LS-012 documentation was already formatted.
- `git diff --check` — PASS before final task/handoff documentation edits.

## Not Verified

- TLS library or platform key-store compatibility; no implementation dependency was selected.
- Certificate creation, persistence, renewal, trust installation, pinning, HTTPS serving, or browser sessions.
- LAN, browser, Tauri, and non-Windows runtime behavior.

## Security-Critical Notes

- Do not enable LAN binding merely because ADR-0007 is accepted.
- Do not fall back to plaintext when identity or TLS setup fails.
- Discovery is only a locator and never establishes trust.
- Do not expose private keys, full credentials, session values, or claim secrets in APIs, logs, URLs, or browser storage.
- Identity reset must revoke peers and sessions; silent root replacement is prohibited.
- A browser warning click-through is not an accepted trust ceremony.

## Next Exact Action

Create LS-013 for a reusable node-root identity service and protected-storage abstraction. First evaluate maintained Rust certificate/key-protection options against Windows, macOS, Linux, and headless requirements. Add deterministic generation/reload/corruption/failure tests and expose only stable node ID plus fingerprint to trusted-local adapters. Do not add TLS serving or change listener binding in LS-013.
