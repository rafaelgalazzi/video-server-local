# Repository-Local AI Memory

The `.ai` directory contains concise operational memory that lets an AI agent safely continue work without prior conversation history. It is not a reasoning log and must not contain chain-of-thought, prompts, or conversational transcripts.

- `PROJECT_STATUS.md`: canonical project-wide implementation status.
- `CURRENT_TASK.md`: the single active task, its evidence, assumptions, and next action.
- `IMPLEMENTATION_ROADMAP.md`: dependency-ordered permanent LS backlog.
- `KNOWN_ISSUES.md`: confirmed or strongly identified bugs only.
- `TECH_DEBT.md`: working implementation that should later improve.
- `DEFERRED_DECISIONS.md`: intentionally postponed decisions and their reopening triggers.
- `handoffs/`: session-to-session records for unfinished work.

Keep entries factual, concise, dated where helpful, and backed by repository or command evidence. Use permanent identifiers and never renumber existing records.
