# Claude Tasks Integration Design

## Overview

This document specifies the migration of LLMC's auto mode task discovery
mechanism from the external `task_pool_command` shell command to direct
integration with Claude Code's native task file format. This eliminates the
dependency on external tooling (lattice/lat) and provides tighter integration
with Claude Code.

LLMC automatically discovers, claims, executes, and completes tasks stored in
`~/.claude/tasks`. It directly reads and writes task JSON files rather than
spawning Claude Code instances for task management.

---

## Task File Format

### Directory Structure

Tasks are stored in `~/.claude/tasks`, organized into subdirectories by task
list ID. Each task is a single JSON file named with a numeric identifier:
`~/.claude/tasks/<task_list_id>/<id>.json`

Claude Code creates task list directories automatically. By default, Claude Code
uses session UUIDs as task list IDs, but `CLAUDE_CODE_TASK_LIST_ID` can override
this to use a specific named directory.

### Task List Configuration

Each LLMC project uses a single, named task list configured in `config.toml`
under `[auto]` as `task_list_id`. This value serves as both the directory name
and the environment variable passed to workers.

For example, with `task_list_id = "dreamtides"`:
- Tasks are stored in `~/.claude/tasks/dreamtides/`
- Workers are started with `CLAUDE_CODE_TASK_LIST_ID=dreamtides`
- All workers and the overseer share this same task list

This ensures multiple LLMC instances for different projects can coexist.

### Worker Task List Binding

LLMC sets `CLAUDE_CODE_TASK_LIST_ID` when starting worker sessions. This ensures
tasks created by workers via TaskCreate appear in the project's task directory
where LLMC can discover them.

### JSON Schema

Each task file contains a JSON object with these fields:

**Required Fields:**
- `id`: String. Task identifier matching the filename without extension.
- `subject`: String. Brief imperative title describing what needs to be done.
- `description`: String. Detailed requirements and context.
- `status`: String. One of `pending`, `in_progress`, or `completed`.
- `blocks`: Array of strings. Task IDs that cannot start until this completes.
- `blockedBy`: Array of strings. Task IDs that must complete first.

**Optional Fields:**
- `activeForm`: String. Present continuous form for display during execution.
- `owner`: String. Worker name currently assigned to this task.
- `metadata`: Object. Arbitrary key-value pairs for extensibility.

### Metadata Extensions

LLMC uses `metadata` to store priority and label information.

**Priority:** `metadata.priority` with integer values 0-4 (0 highest). Tasks
without priority are treated as priority 3.

**Label:** `metadata.label` as a string. Used for context injection and
concurrency optimization. Label values must be valid TOML table names
(alphanumeric, hyphens, underscores only).

---

## Task Discovery

### Scan Process

LLMC scans `~/.claude/tasks/<task_list_id>/` at the start of each orchestration
cycle. Each `.json` file is parsed as a task. Tasks in other directories are
not scanned.

### Eligibility Criteria

A task is eligible for assignment when:

1. Status is `pending`
2. No owner assigned (or owner field is empty/null)
3. `blockedBy` is empty or all referenced tasks have status `completed`
4. The task file is readable and parseable

### Dependency Resolution

LLMC builds an in-memory dependency graph from all tasks' `blocks` and
`blockedBy` fields. A task is eligible only if every `blockedBy` reference
resolves to a task with status `completed`.

If a `blockedBy` reference points to a nonexistent task, the dependent task
remains ineligible. Circular dependencies trigger daemon shutdown with an error
listing all task IDs in the cycle.

---

## Task Selection Algorithm

When multiple eligible tasks exist, LLMC selects using these factors in order:

1. **Distinct Label Preference**: Prefer tasks whose label differs from all
   tasks currently worked on by other auto workers. Reduces merge conflicts.

2. **Priority**: Select tasks with lowest priority value (highest urgency).

3. **Creation Order**: Select task with lowest numeric ID (FIFO within priority).

### Distinct Label Selection

When selecting for worker N:

1. Collect labels from tasks assigned to other workers
2. Filter to tasks whose label is not in this set (or have no label)
3. If empty, consider all eligible tasks
4. Apply priority and creation order

Tasks without labels are "distinct" from all labeled tasks and each other.

---

## Concurrent Access and Atomicity

Multiple LLMC daemon processes may run concurrently. All task file operations
must be safe against concurrent access.

### Atomic File Writes

All writes use the atomic pattern:

1. Write to temporary file (e.g., `<id>.json.tmp.<pid>`)
2. `fsync` the temporary file
3. Atomically rename to target filename

### Atomic File Reads

Read entire file content in single operation, parse JSON. Parse failures are
errors (no retries).

### Claim Race Prevention

When claiming a task:

1. Read task, verify `pending` status and empty owner
2. Write updated task with `in_progress` and owner set
3. Re-read task immediately
4. Verify owner matches this worker's name
5. If owner differs, another process won; abandon and select different task

Lost races are not errors. LLMC does not use file locks; atomic writes plus
claim verification provide sufficient safety.

---

## Task Lifecycle Management

### Claiming a Task

LLMC writes the updated task file before sending the prompt:
- `status` changed to `in_progress`
- `owner` set to worker name (e.g., `auto-1`)

### Task Completion

When changes are accepted (merged to master):
- `status` changed to `completed`
- `owner` cleared

Completed tasks remain on disk for dependency checks.

### Task Failure

On any worker failure (crash, error state, daemon shutdown), LLMC immediately:

1. Sets `status` back to `pending`
2. Clears the `owner` field

LLMC does not attempt to resume failed tasks. The task becomes eligible for
reassignment in the next cycle.

### No Changes Handling

If a worker completes without code changes (`no_changes` status), LLMC still
marks the task as `completed`.

---

## Context Injection System

### Prologue and Epilogue

Context injection prepends (prologue) or appends (epilogue) text to task
descriptions based on the task's label. This enables label-specific instructions
without duplicating text across task files.

### Configuration

Context rules are in `.claude/llmc_task_context.toml` (version-controlled).
Label names must be valid TOML bare keys. Sections are separated by two blank
lines:

```toml
[default]
prologue = """
Follow all coding standards in CLAUDE.md.
Run `just check` before completing.
"""
epilogue = "Remember to run tests."


[ui]
prologue = """
This task involves UI components.
Follow the component patterns in src/components/README.md.
"""
epilogue = "Ensure accessibility requirements are met."


[backend]
prologue = """
This task involves backend systems.
Follow the API patterns in src/api/README.md.
"""
```

### Content Resolution

1. Look up task's label in configuration
2. If found, use that label's prologue/epilogue
3. If not found, use `[default]` values
4. If no default, use empty strings

### Prompt Assembly

1. Standard LLMC task preamble (worktree location, instructions, etc.)
2. Resolved prologue (if non-empty)
3. Task subject and description
4. Resolved epilogue (if non-empty)

---

## Configuration Changes

### Removed Configuration

- `[auto].task_pool_command`: Tasks discovered from filesystem
- CLI flags related to task pool command

### New Configuration

**`task_list_id`** (required, string): Directory name within `~/.claude/tasks/`.
Also set as `CLAUDE_CODE_TASK_LIST_ID` for workers.

**`tasks_root`** (optional, string): Override `~/.claude/tasks` root. Useful for
testing.

**`context_config_path`** (optional, string): Override path to context config.
Defaults to `.claude/llmc_task_context.toml` relative to `repo.source`.

---

## Failure Handling

LLMC follows "fail early and let overseer resolve". Task errors trigger shutdown.

### Task File Access Errors

- **File not readable**: Log error and shut down
- **Invalid JSON**: Log error with filename and parse error, shut down
- **Missing required fields**: Log error and shut down
- **Write failures**: Log error and shut down

### Dependency Resolution Errors

- **Missing dependency**: Log error and shut down
- **Circular dependency**: Log error listing cycle, shut down

### Context Configuration Errors

- **Missing context file**: Proceed with empty prologue/epilogue (not an error)
- **Invalid TOML**: Log error and shut down

---

## Testing Strategy

All testing follows black-box methodology, testing only real public APIs using
fake implementations.

### Integration Testing with Fakes

Tests create temporary directory structure mimicking `~/.claude/tasks/` with
task JSON files, verifying behavior through public CLI or library API.

**Scenarios:**
- Task discovery finds all `.json` files
- Eligibility correctly filters by status, owner, blockedBy
- Selection respects priority and label preferences
- Claim/complete cycle updates files correctly
- Concurrent claims handled safely (simulate with threads)
- Invalid task files trigger errors
- Circular dependencies detected

### Manual Testing Scenarios

- **Happy path**: Multiple tasks with varying priorities/labels, verify order
- **Dependency chain**: A→B→C dependency, verify execution order
- **Conflict avoidance**: Same-label tasks, verify workers prefer different labels
- **External modification**: Edit task file while running, verify detection
- **Context injection**: Various labels, verify correct prologue/epilogue

---

## Removal of Lattice Integration

This migration replaces all lattice (`lat`) functionality for task management.
The `task_pool_command` configuration is removed entirely.

LLMC commands that interact with workers directly (`llmc start`, `llmc review`,
`llmc accept`, etc.) continue unchanged. The overseer system continues
functioning with the same failure detection and remediation approach.

---

## State File Changes

### New State Fields

**`active_task_ids`**: Map from worker name to task ID being worked on. Used for
distinct label selection.

### Removed State Fields

The `task_pool_backoff_*` fields are removed. These tracked retry delays when
the `task_pool_command` shell command failed, which no longer applies since we
read task files directly. Standard fail-fast error handling replaces backoff.

### State Persistence

Task state is primarily in the JSON files, not `state.json`. The state file
tracks worker-to-task mappings for distinct label selection. If state is lost,
LLMC reconstructs by scanning task files and matching owners to workers.

---

## Overseer Integration

### Failure Context

When daemon shuts down due to task failure, the overseer's remediation prompt
includes:
- Task ID and file path (if applicable)
- Failure type (parse error, missing field, write failure, etc.)
- Error message

### Remediation Actions

The overseer may:
- Fix malformed task JSON files
- Resolve circular dependencies by editing blockedBy fields
- Restore missing task files from version control
- Fix file permission issues

### Recovery After Remediation

After remediation, daemon restarts normally. Tasks that were `in_progress` have
already been reset to `pending` (per failure handling) and are eligible for
reassignment.
