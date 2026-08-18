# Handoffs

Create or update a handoff when a task is unfinished, blocked, partially complete, crossing sessions, accumulating large context, or leaving important uncommitted work. Completed trivial tasks do not require permanent handoffs unless continuity benefits.

Use filenames `LS-XXX-short-description.md`. Never claim a test passed unless it actually ran.

```md
# Handoff — LS-XXX

## Objective

...

## Current State

...

## Changed Files

- ...

## Important Decisions

- ...

## Completed

- ...

## Remaining

- ...

## Tests Executed

- command
- result

## Tests Not Executed

- ...

## Known Failures

- ...

## Assumptions

- ...

## Next Exact Action

...

## Do Not

- ...
```

A useful handoff captures facts and continuation instructions, not verbose reasoning or conversation history. Old completed handoffs may be archived without renumbering their task IDs.
