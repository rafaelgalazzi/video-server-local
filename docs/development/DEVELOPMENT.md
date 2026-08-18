# Development

The frontend and native project foundation is scaffolded. Commands are marked when they could not be verified in the current environment.

## Prerequisites

- Node.js 22.13 or newer and npm. Node 22.12 executed the frontend gates, but a current lint dependency does not declare support until 22.13.
- Rust stable meeting the workspace MSRV in `Cargo.toml`.
- Current [Tauri 2 system prerequisites](https://v2.tauri.app/start/prerequisites/) for the target OS.

## Installing Dependencies

Run `npm install` from the repository root. `package-lock.json` is canonical. Cargo resolves Rust dependencies from the workspace manifests; commit `Cargo.lock` after Cargo first generates it.

## Starting Vue

Run `npm run dev`. The Vite preview listens on `http://localhost:1420`; it cannot invoke native commands and shows a non-fatal preview state.

## Starting Tauri

Run `npm run tauri dev`. This was not executed on 2026-08-18 because Rust was unavailable.

## Running Rust Tests

Run `cargo test --workspace`. Also use `cargo test -p localstream-core` for targeted core tests. Not yet verified locally.

## Running Frontend Tests

Run `npm run test` for Vitest once or `npm run test:watch` during development.

## Type Checking

Run `npm run typecheck`.

## Linting

Run `npm run lint`.

## Formatting

Run `npm run format` to write frontend and documentation formatting, or `npm run format:check` to check it. Run `cargo fmt --all --check` for Rust.

## Building

Run `npm run build` for the frontend production bundle. Run `npm run tauri build` for a native build after bundle icons and platform packaging are configured; bundling is currently disabled.

## Canonical Verification

Run `npm run verify` for frontend format checking, linting, type checking, unit tests, and a production build. It passed on 2026-08-18.

Rust remains separate until the toolchain is available:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo check --workspace
```

These Rust commands are canonical but not yet verified in this repository.

## Troubleshooting

- If Vitest/esbuild reports access denied while resolving `vitest.config.ts`, rerun verification with the workspace execution permission required by the host sandbox.
- If a documented command is absent, treat the documentation as stale and update it with the implementation.
- Run commands from the repository root unless a directory README says otherwise.
- Do not delete lockfiles or uncommitted work to resolve dependency problems.
- Record unresolved failures in the active task and handoff, including the exact command and output summary.
