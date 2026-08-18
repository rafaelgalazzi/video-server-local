# LocalStream

LocalStream is a planned cross-platform, local-first media application for browsing and streaming explicitly approved media libraries across a LAN without depending on cloud infrastructure.

The stack is Vue 3 + TypeScript for the interface, Tauri 2 for the native shell, and a reusable Rust core for media, persistence, networking, streaming, and security. The initial project foundation now exists; media-library behavior is not yet implemented.

## Start Here

AI agents must begin with [AGENTS.md](AGENTS.md) and `.ai/SESSION_START.md`. Human contributors should also review:

- [Project status](.ai/PROJECT_STATUS.md)
- [Architecture map](docs/architecture/ARCHITECTURE_MAP.md)
- [Development workflow](docs/development/DEVELOPMENT.md)
- [Test matrix](docs/development/TEST_MATRIX.md)
- [Architecture decisions](docs/architecture/adr/README.md)

## Product Direction

The first target is an end-to-end desktop-to-browser flow: approve a media folder, scan it into SQLite, display it in Vue, serve it over the LAN with Axum, and Direct Play a compatible file with HTTP Range support. Discovery, pairing, distributed libraries, and transcoding follow in later milestones.

The detailed product vision remains in:

- `LocalStream_AI_Project_Blueprint.md`
- `LocalStream_Code_Quality_Testing_Standard.md`

These are planning inputs. Accepted architecture decisions live in ADRs, and actual implementation state lives in `.ai/PROJECT_STATUS.md`.

## Development

For the shortest setup and startup instructions, read [RUN.md](RUN.md).

Install dependencies with `npm install`, launch the web preview with `npm run dev`, and run all available frontend checks with `npm run verify`. Native commands require a Rust toolchain and Tauri system prerequisites; use `npm run tauri dev` after installing them. See [DEVELOPMENT.md](docs/development/DEVELOPMENT.md) for exact verified and unverified workflows.
