# Technical Debt

Use this file only for implementation that currently works but should eventually be improved. Do not record missing features or confirmed bugs here.

## TD-001 — Declared Rust baseline does not match the resolved workspace

Priority: Medium

Affected module: Cargo workspace and CI/toolchain policy.

Reason: The workspace declares Rust 1.77.2, but the pre-LS-013 lockfile already resolved packages such as Axum and UUID whose published `rust-version` is newer. LS-013 pinned its new `time` dependency to 0.3.36, but a full metadata audit still reports pre-existing packages above 1.77.2. Only the installed Rust 1.97.1 toolchain was available for verification.

Preferred future direction: Decide and test a supported MSRV in CI, then either raise `workspace.package.rust-version` honestly or systematically pin the complete dependency graph to the intended older compiler.

Do not address before: The next dependency-policy or CI task; do not mix a workspace-wide downgrade into a feature task.

## Entry Template

```md
## TD-001 — Description

Priority: Low / Medium / High

Affected module: ...

Reason: ...

Preferred future direction: ...

Do not address before: ...
```
