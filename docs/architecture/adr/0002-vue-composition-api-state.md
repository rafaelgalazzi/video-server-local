# ADR-0002 — Use Vue Composition API state without Pinia

## Status

Accepted

## Context

The initial interface needs shared and reusable state without premature global-state infrastructure.

## Decision

Use Vue 3 Composition API primitives: `ref`, `reactive`, `computed`, composables, and `provide` / `inject` where appropriate. Do not use Pinia.

## Alternatives Considered

- Pinia from project initialization.
- Options API component state.

## Consequences

State remains close to features with fewer dependencies. Composables need clear ownership and testing. A state-management library may be introduced only through a future explicit ADR.
