# LocalStream Agent Instructions

This file is the mandatory entry point for every AI coding agent working in this repository.

## Before Modifying Code

Read, in order:

1. `AGENTS.md`
2. `README.md`
3. `.ai/PROJECT_STATUS.md`
4. `.ai/CURRENT_TASK.md`
5. Relevant directory `README.md` files
6. Relevant ADRs in `docs/architecture/adr/`
7. Existing tests related to the requested feature

Then run `git status` and `git diff`. Determine whether unfinished or uncommitted work already exists. Never discard uncommitted changes without explicit authorization.

## Repository Memory

> Important project knowledge must live in the repository, not only in AI conversation history.

> If another agent needs information to safely continue implementation, that information must exist in the repository.

Persist implementation-affecting knowledge in source documentation, a directory README, an ADR, `.ai/PROJECT_STATUS.md`, `.ai/CURRENT_TASK.md`, a handoff, API documentation, or security documentation. Never rely on “as discussed earlier in the conversation” unless the decision is also recorded here. Store decisions, state, results, failures, changed files, executed commands, and next actions—not chain-of-thought or conversation transcripts.

Each information type has one canonical source:

- Implementation status: `.ai/PROJECT_STATUS.md`
- Active work: `.ai/CURRENT_TASK.md`
- Confirmed bugs: `.ai/KNOWN_ISSUES.md`
- Working-but-imperfect implementation: `.ai/TECH_DEBT.md`
- Intentionally postponed decisions: `.ai/DEFERRED_DECISIONS.md`
- Architecture decisions: `docs/architecture/adr/`
- API contract: `docs/api/README.md`

Link to canonical sources instead of duplicating detail.

## Architecture Rules

### Frontend

Use Vue 3, TypeScript, the Composition API, `ref()`, `reactive()`, `computed()`, composables, and `provide()` / `inject()` where useful. Do not introduce Pinia unless a future accepted ADR explicitly changes this rule.

Vue components must not contain native filesystem, streaming, database, FFmpeg, or networking business logic.

### Native and Core

Use Tauri 2 and Rust. Rust is the main application/server core and owns the media library, filesystem access, local HTTP server, streaming, SQLite, node discovery, pairing, security, FFmpeg integration, transcoding, and appropriate platform-native integration.

Design the core for eventual reuse by LocalStream Desktop, Mobile, Headless Server, CLI, and NAS distributions. Keep platform-specific code isolated.

Tauri commands and HTTP handlers are thin adapters to shared services:

```text
Tauri Command ----+
                  |
                  v
             Core Service
                  ^
                  |
HTTP Handler -----+
```

Do not duplicate business logic between transports.

## Architectural Invariants

1. Vue must not directly access native filesystem operations.
2. Vue must not contain media streaming business logic.
3. Tauri commands must remain thin adapters.
4. HTTP handlers must remain thin adapters.
5. Business logic must live in reusable Rust services/modules.
6. Raw filesystem paths must never be exposed through public HTTP APIs.
7. Public media access must use opaque media identifiers.
8. Media files must never be fully loaded into memory for streaming.
9. Streaming must use bounded or streaming I/O.
10. Direct Play must be attempted before transcoding.
11. Unpaired LAN nodes must be considered untrusted.
12. User media must only be exposed from explicitly approved libraries.
13. FFmpeg arguments must never be constructed using unsafe shell interpolation.
14. Resource-heavy operations must have concurrency limits.
15. Network failures must not crash the application.
16. One inaccessible media file must not necessarily abort an entire library scan.
17. Platform-specific code must remain isolated from reusable core logic.
18. The core application must not require Internet access for primary LAN functionality.

## Implementation Rules

- Before creating a service, helper, utility, parser, type, adapter, repository abstraction, or network abstraction, search for equivalent functionality.
- Avoid generic dump modules such as `utils.ts`, `helpers.ts`, `utils.rs`, `common.rs`, and `misc.rs`. Prefer domain locations such as `streaming/range.rs`.
- Every meaningful source directory needs a concise README covering purpose, features, important files, public interfaces, dependencies, limitations, and planned work. Do not document generated or build directories.
- Break large work into cohesive, resumable units that compile where practical.
- Record temporary implementation assumptions in `.ai/CURRENT_TASK.md`.
- Do not reopen a deferred decision until its documented trigger is reached.

## Continuation Safety

> Any unfinished task must be resumable by another agent with no access to the previous conversation.

A handoff is incomplete unless the next agent can determine the objective, current state, changes, remaining work, verified and unverified behavior, known failures, and next exact action. Create or update a handoff when work is unfinished, blocked, partially complete, likely to cross sessions, context is becoming large, or important uncommitted work exists.

Use permanent IDs: `LS-001`, `LS-002`, and so on. Never renumber an established ID. Bug, debt, and deferred-decision IDs use `KI-`, `TD-`, and `DD-` respectively.

## Quality and Verification

Before declaring an implementation complete:

- Format relevant code.
- Run applicable frontend lint, TypeScript checks, and tests.
- Run applicable Rust formatting, checks, Clippy, and tests.
- Run applicable integration checks and builds.
- Review errors and warnings.
- Verify acceptance criteria.
- Update documentation and task state.
- Inspect `git diff`.

Never claim that tests pass, a build succeeds, or a platform works without executing the relevant verification. Reports must distinguish **Verified**, **Not verified**, **Not applicable**, and **Unable to execute**, and list exact commands when checks ran.

Before completion, review the diff for debug output, temporary code, unrelated edits, duplicate logic, machine-specific paths, hardcoded IP addresses, secrets, disabled tests or lint rules, dead code, unnecessary dependencies, broken docs, and placeholder production behavior.

For confirmed bugs, where practical: reproduce, add a failing regression test, fix, and confirm the test passes.

## Final Report

Finish coding tasks with:

```text
Implemented:
- ...

Changed:
- ...

Tests executed:
- command — PASS / FAIL

Verification not performed:
- ...

Documentation updated:
- ...

Known limitations:
- ...

Next step:
- ...
```
