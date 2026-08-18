# Current Task

## ID

LS-001

## Title

Initial Vue, Tauri, and reusable Rust-core scaffold

## Status

Verification

## Goal

Establish the first runnable project foundation from the product blueprint while preserving the accepted architecture boundaries.

## Acceptance Criteria

- A Vue 3 + TypeScript + Vite frontend exists and uses Composition API state without Pinia.
- A Tauri 2 shell exists with thin commands.
- A separate Rust core crate exposes framework-independent application information used by a Tauri adapter.
- Baseline frontend and Rust unit tests exist.
- Canonical format, lint, typecheck, test, build, and verification commands are documented and wired where tooling permits.
- Meaningful source directories contain concise README files.
- Governance status and test evidence reflect actual results.

## Relevant Files

- `package.json`
- `src/`
- `src-tauri/`
- `crates/localstream-core/`
- `docs/development/DEVELOPMENT.md`
- `docs/development/TEST_MATRIX.md`

## Completed

- Repository governance bootstrap.
- Confirmed Node.js 22.12.0 and npm 11.6.1 are available.
- Created the Vue 3 + TypeScript + Vite frontend and offline-first foundation screen.
- Created a typed Composition API adapter with two unit tests.
- Created the Tauri 2 shell with a thin `app_info` command.
- Created the standalone `localstream-core` crate and baseline Rust unit test.
- Added npm quality commands and installed a locked dependency graph.
- Ran the complete frontend verification pipeline successfully.

## In Progress

- Native verification is awaiting an installed Rust toolchain.

## Remaining

- Install a supported Rust toolchain and Tauri system prerequisites.
- Run Rust formatting, Clippy, tests, and checks.
- Launch `npm run tauri dev` and smoke-test the native adapter.
- Resolve any native-only errors, then mark LS-001 completed.

## Tests Last Executed

- `npm run verify` — PASS on 2026-08-18.
- Vitest: 1 file and 2 tests passed.
- Vite production build: PASS (16 modules transformed).

## Tests Not Yet Executed

- Rust formatting, Clippy, unit tests, and checks.
- Tauri development/build smoke test.

## Known Problems

- `rustc`, `cargo`, and `rustup` are not installed or not available on `PATH` in the current environment.
- `npm install` warns that transitive `eslint-visitor-keys@5.0.1` declares Node `^22.13.0` while the available runtime is 22.12.0. All frontend gates still passed; use Node 22.13+ for supported development.

## Assumptions

- LS-001 establishes architecture and a minimal UI-to-core adapter path; media scanning, SQLite, Axum, and streaming are separate later tasks.
- npm is the initial frontend package manager and `package-lock.json` is the canonical lockfile.
- The initial Tauri target is desktop; mobile initialization is deferred until desktop scaffolding is verified.

## Next Exact Step

Install Rust with rustup, restart the shell so Cargo is on `PATH`, and run `cargo fmt --all --check` from the repository root.
