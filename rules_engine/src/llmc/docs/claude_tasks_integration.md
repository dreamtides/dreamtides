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

---

## Implementation Plan

This section describes the milestones for implementing Claude Tasks integration
in the LLMC codebase. Each milestone builds on the previous ones and can be
developed and tested incrementally.

### Milestone 1: Task File Module

**Goal:** Create a new module for reading and writing Claude task JSON files
with proper atomicity guarantees.

**Current State:** LLMC has no code for interacting with Claude task files. The
existing `state.rs` module demonstrates the atomic write pattern (temp file +
rename) that should be reused. The `State::save()` function at line 265 shows
the pattern: serialize to JSON, write to temp file with PID suffix, rename
atomically.

**Changes Required:** Create a new module `auto_mode/claude_tasks.rs` containing
the `ClaudeTask` struct matching the JSON schema (id, subject, description,
status, blocks, blockedBy, owner, metadata). Implement `ClaudeTask::load(path)`
that reads and parses a single task file, returning `Result<ClaudeTask>` with
errors for invalid JSON or missing required fields. Implement
`ClaudeTask::save(path)` using the atomic write pattern from `state.rs`: write
to `<id>.json.tmp.<pid>`, fsync, then rename. Add helper functions
`get_priority()` and `get_label()` that extract values from the metadata object
with appropriate defaults (priority 3, no label). Define the `TaskStatus` enum
with variants Pending, InProgress, and Completed, with serde serialization to
match the JSON string values.

### Milestone 2: Configuration Changes

**Goal:** Update `AutoConfig` to replace `task_pool_command` with the new task
integration settings.

**Current State:** The `AutoConfig` struct in `auto_mode/auto_config.rs` (lines
8-24) contains `task_pool_command: Option<String>`, `concurrency: u32`, and
`post_accept_command: Option<String>`. The `ResolvedAutoConfig` struct (lines
50-71) merges CLI and TOML config. CLI argument parsing in `cli.rs` includes
`--task-pool-command` override.

**Changes Required:** Remove the `task_pool_command` field from `AutoConfig` and
`ResolvedAutoConfig`. Remove the `--task-pool-command` CLI flag from `cli.rs`.
Add new fields to `AutoConfig`: `task_list_id: Option<String>` (required when
auto mode is used), `tasks_root: Option<String>` (defaults to `~/.claude/tasks`
via a helper function), and `context_config_path: Option<String>` (defaults to
`.claude/llmc_task_context.toml` relative to repo.source). Update
`ResolvedAutoConfig::resolve()` to validate that `task_list_id` is present when
auto mode is enabled, returning an error if missing. Add a helper method
`get_task_directory()` that combines `tasks_root` and `task_list_id` into the
full path. Update the TMUX session startup code in `commands/up.rs` to set the
`CLAUDE_CODE_TASK_LIST_ID` environment variable when launching worker sessions.

### Milestone 3: Task Context System

**Goal:** Implement the context injection system for label-based prologue and
epilogue text.

**Current State:** Prompt construction happens in two places:
`auto_orchestrator.rs` has `build_auto_prompt()` (lines 513-535) for auto mode,
and `commands/start.rs` has `build_full_prompt()` (lines 309-339) for manual
starts. Both construct a preamble with worktree path, repo root, and standard
instructions, then append the task content.

**Changes Required:** Create a new module `auto_mode/task_context.rs` containing
the `TaskContextConfig` struct with a HashMap of label names to `LabelContext`
structs (each having optional prologue and epilogue strings). Implement
`TaskContextConfig::load(path)` that reads and parses the TOML file, returning
an error for invalid TOML syntax. Add a `resolve(label: Option<&str>)` method
that looks up the label, falls back to "default" if not found, and returns the
resolved prologue and epilogue (empty strings if neither exists). Modify
`build_auto_prompt()` to accept the resolved prologue and epilogue, inserting
the prologue after the standard preamble and the epilogue after the task
description. The context config file should be loaded once at daemon startup and
stored in the orchestrator context, with missing file treated as empty config
(not an error).

### Milestone 4: Task Discovery and Selection

**Goal:** Replace the task pool command execution with filesystem scanning and
the multi-factor selection algorithm.

**Current State:** Task assignment happens in `auto_orchestrator.rs` in
`process_idle_workers()` (lines 401-456). For each idle worker, it calls
`execute_task_pool_command()` from `task_pool.rs` (lines 41-84), which runs a
shell command and interprets exit codes. The task content comes from stdout.
There is no current selection algorithm since the external command handles
selection.

**Changes Required:** Create `auto_mode/task_discovery.rs` with a
`discover_tasks(task_dir: &Path)` function that scans the directory, loads all
`.json` files using the task file module, and returns `Result<Vec<ClaudeTask>>`.
Any file that fails to parse triggers an immediate error return (fail-fast). Add
`build_dependency_graph(tasks: &[ClaudeTask])` that constructs an in-memory
graph and detects circular dependencies, returning an error listing cycle
members if found. Add `get_eligible_tasks(tasks, graph)` that filters to pending
tasks with no owner and all blockedBy dependencies completed. Create
`auto_mode/task_selection.rs` with `select_task(eligible: &[ClaudeTask],
active_labels: &HashSet<String>)` implementing the three-factor algorithm:
filter by distinct labels (keeping unlabeled tasks), sort by priority ascending,
then by numeric ID ascending, return the first task. Replace the
`execute_task_pool_command()` call in `process_idle_workers()` with calls to
discover, filter, select, returning `None` when no eligible tasks exist.

### Milestone 5: Task Lifecycle Updates

**Goal:** Implement task claiming, completion, and failure handling with proper
atomicity and race prevention.

**Current State:** The orchestrator assigns tasks in `assign_task_to_worker()`
(lines 465-510), which pulls master, builds the prompt, sends `/clear`, and
stores the prompt in `pending_task_prompt`. Task completion is handled in
`process_completed_workers()` (lines 538-781), which accepts changes and resets
the worker. Worker failures are detected in `auto_failure.rs` with
`detect_transient_failures()` and `attempt_recovery()`.

**Changes Required:** Create `claim_task(task: &mut ClaudeTask, worker_name:
&str, task_path: &Path)` that sets status to InProgress and owner to worker
name, saves atomically, re-reads the file, and verifies the owner matches. If
verification fails (race lost), return a specific error variant that the caller
handles by selecting a different task. Update `assign_task_to_worker()` to call
`claim_task()` before building the prompt, retrying with a different task on
race loss. Create `complete_task(task_id: &str, task_dir: &Path)` that loads the
task, sets status to Completed, clears owner, and saves atomically. Call this
from `process_completed_workers()` after successful accept. Create
`release_task(task_id: &str, task_dir: &Path)` that resets status to Pending and
clears owner. Call this from all failure paths: worker crash detection in
`handle_session_end()`, error state transitions in `apply_transition()`, and
daemon shutdown cleanup. Store the active task ID in the worker record (new
field `active_task_id: Option<String>`) so failure handlers know which task to
release.

### Milestone 6: State File Changes

**Goal:** Update the State struct to track active task IDs and remove obsolete
backoff fields.

**Current State:** The `State` struct in `state.rs` (lines 123-155) contains
`source_repo_dirty_retry_after_unix`, `source_repo_dirty_backoff_secs`, and
`source_repo_dirty_retry_count` for source repo dirty backoff. The
`WorkerRecord` struct (lines 38-121) has `auto_retry_count` for transient
failure retry tracking. There are no task-pool-specific backoff fields currently
(the research showed the backoff is for source repo dirty state, not task pool
failures).

**Changes Required:** Add `active_task_ids: HashMap<String, String>` to the
`State` struct, mapping worker name to task ID. This is populated when a task is
claimed and cleared when the task is completed or released. Add
`active_task_id: Option<String>` to `WorkerRecord` for per-worker tracking (this
duplicates the State map but makes failure handling simpler since worker records
are passed to transition functions). Update `apply_transition()` in `worker.rs`
to clear `active_task_id` when transitioning to Idle, Error, or Offline. The
existing source repo dirty backoff fields remain unchanged since they handle a
different scenario (dirty master repo during accept). The `auto_retry_count`
field remains for transient failure recovery. No fields are actually removed
since the backoff research showed these track source repo state, not task pool
command failures.

### Milestone 7: Error Handling Integration

**Goal:** Ensure all task-related errors trigger daemon shutdown per the
fail-fast philosophy.

**Current State:** The orchestration loop in `run_orchestration_loop()` (lines
100-396) handles errors by setting `shutdown_error` and breaking the loop. The
`process_idle_workers()` function returns `Result<bool>` where errors cause
shutdown. Hard failures are handled in `auto_failure.rs` with
`HardFailure::WorkerRetriesExhausted` triggering shutdown.

**Changes Required:** Define a new error type `TaskError` in
`auto_mode/claude_tasks.rs` with variants: `ParseError { path, source }`,
`MissingField { path, field }`, `WriteError { path, source }`,
`CircularDependency { task_ids }`, `MissingDependency { task_id, missing_id }`.
Ensure all task file operations return `Result<_, TaskError>`. In
`process_idle_workers()`, convert `TaskError` to `anyhow::Error` with context
including the task file path and error details. This propagates up to the
orchestration loop where it triggers shutdown. Update the overseer remediation
prompt builder in `overseer_mode/remediation_prompt.rs` to include task-specific
context when the shutdown was caused by a task error: extract the task ID and
path from the error, include a snippet of the malformed file if it was a parse
error, and suggest specific remediation actions (fix JSON syntax, remove
circular dependency, etc.).

### Milestone 8: Integration and Testing

**Goal:** Wire all components together, add integration tests, and perform
manual validation.

**Current State:** LLMC has an existing test infrastructure with integration
tests. The `tests/` directory contains test modules. The daemon can be run in
isolated mode using `LLMC_ROOT` environment variable to create independent test
instances.

**Changes Required:** Create `tests/claude_tasks_integration.rs` with
integration tests using temporary directories for task files. Test task
discovery: create a temp directory with valid task JSON files, call the
discovery function through the public API (or a thin test wrapper), verify all
tasks are returned. Test eligibility: create tasks with various status/owner/
blockedBy combinations, verify filtering works correctly. Test selection: create
multiple eligible tasks with different priorities and labels, verify selection
order. Test claiming with simulated races: use multiple threads to claim the
same task, verify exactly one succeeds. Test failure handling: set up a worker
with an active task, simulate crash, verify task is released to pending. Test
context injection: create a context config file, verify prompts include correct
prologue/epilogue. For manual testing, create a `~/.claude/tasks/test-project/`
directory with sample tasks, configure LLMC with `task_list_id = "test-project"`,
run `llmc up`, and verify tasks are discovered, claimed, and completed in the
expected order.

### Milestone 9: Cleanup and Documentation

**Goal:** Remove obsolete code, update documentation, and finalize the
migration.

**Current State:** The `task_pool.rs` module contains all the shell command
execution logic. The `auto_config.rs` module has the `--task-pool-command` CLI
handling. Various log messages reference "task pool". The `llmc.md` design
document describes the task pool command system.

**Changes Required:** Delete `auto_mode/task_pool.rs` entirely once all
functionality is migrated and tested. Remove any imports of this module
throughout the codebase. Update log messages in `auto_orchestrator.rs` to
reference "task discovery" instead of "task pool". Update the auto mode logger
in `auto_logger.rs` to log task file operations instead of command executions.
Update `rules_engine/docs/llmc.md` to document the new task integration system,
removing references to `task_pool_command` and adding sections on task file
format, context injection, and the new configuration options. Add a
`.claude/llmc_task_context.toml` example file to the dreamtides repository with
appropriate labels for the project's different areas (rules engine, client,
etc.). Update the CLAUDE.md instructions to mention task management with Claude
Code's native task format instead of lattice.
