# AI Agent Workflow

This workflow supplements the mandatory rules in `AGENTS.md`.

## Session Start

```text
Read governance files
        |
Inspect git state
        |
Read current task
        |
Read module docs
        |
Read relevant ADRs and tests
        |
Understand existing implementation
```

Confirm whether unfinished work exists before starting anything new. Search for existing equivalent abstractions before adding one.

## Implementation

```text
Small cohesive change
        |
Add or update tests
        |
Run targeted checks
        |
Record assumptions/failures
        |
Continue
```

Keep work continuation-safe. If an intermediate state cannot compile, explain why and give the next exact repair action in `.ai/CURRENT_TASK.md`.

## Completion

```text
Format
  |
Lint and typecheck
  |
Test and build/check
  |
Self-review git diff
  |
Update canonical docs and task status
  |
Create handoff if unfinished
```

Only report checks that actually ran. Identify each as verified, not verified, not applicable, or unable to execute. For important bug fixes, reproduce and add a regression test where practical before implementing the fix.
