---
name: breaking-down-tasks-codex
description: Break a design doc or project plan into durable Codex tasks stored in .codex/tasks with explicit dependencies managed through .codex/scripts/task.py. Use when asked to decompose implementation work, plan task graphs, or create actionable engineering tasks from specs.
---

# Breaking Down Design Docs into Codex Tasks

Decompose technical documents into small, independent implementation tasks that
can be completed in one Codex session. Persist all tasks under `.codex/tasks`
using `.codex/scripts/task.py`.

## Workflow

### Phase 1: Research in Parallel

Spawn 2-4 lightweight exploration agents in parallel to gather:

- codebase entry points and impacted files
- existing patterns and conventions for similar features
- likely type/function dependencies
- test locations and validation patterns

Wait for all exploration outputs before creating tasks.

### Phase 2: Design the Task Graph

Build task breakdown from research and the source design doc:

1. Identify logical units of work.
2. Order tasks by foundation first.
3. Add only necessary dependencies.
4. Draft task descriptions with concrete acceptance criteria.

### Phase 3: Persist Tasks

Initialize task store if needed:

```bash
.codex/scripts/task.py init
```

Create tasks:

```bash
.codex/scripts/task.py add \
  --title "Add DamageEffect enum variant and parser support" \
  --markdown-file /tmp/task.md
```

Add dependency edges:

```bash
.codex/scripts/task.py update T0004 --add-blocker T0002
```

Validate:

```bash
.codex/scripts/task.py validate
```

## Task Ordering

Use oldest-first execution as a soft dependency mechanism:

1. Foundational data types, enums, and traits.
2. Core logic implementations.
3. Integration wiring across systems.
4. Tests-only, cleanup, and polish.

## Dependency Strategy

Add explicit blockers only when required:

- two tasks modify the same file or function area
- task B uses new type/function introduced by task A
- task B extends behavior implemented in task A

Do not add blockers for conceptual linkage alone when code paths are separate.

Default to fewer dependencies for better parallelism.

## Task Quality Template

Use this template for each task body passed to `task.py add`:

```markdown
## Context
[2-3 sentences: project domain, feature goal, and where this task fits.]

## Objective
[1-2 sentences: exact outcome for this task.]

## Key Files
Read these files for context before starting:
- `path/to/file.rs` - why this file matters
- `path/to/pattern.rs` - pattern to follow

## Requirements
1. Concrete change one
2. Concrete change two
3. Concrete change three

## Acceptance Criteria
- [ ] Verifiable task-specific criterion
- [ ] Verifiable task-specific criterion
- [ ] Code compiles: `just check`
- [ ] Formatting applied: `just fmt`
- [ ] Lints and tests pass: `just review`
- [ ] Changes committed with descriptive message
```

## Sizing Rules

- aim for 1-3 files changed per task
- cap near 200 LOC changed per task
- split tasks touching more than 5 files
- split tasks that require long instructions to explain

## Token Efficiency Rules for Agents

- use `.codex/scripts/task.py ready` and `.codex/scripts/task.py list` first
- call `.codex/scripts/task.py get <id> --body` only for selected tasks
- prefer `--json` for orchestrator parsing and chaining
- avoid loading all task markdown bodies into context at once
