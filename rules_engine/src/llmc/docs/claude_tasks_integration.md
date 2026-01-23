# Claude Tasks Integration Design

## Overview

This document specifies the migration of LLMC's auto mode task discovery
mechanism from the external `task_pool_command` shell command to direct
integration with Claude Code's native task file format. This change eliminates
the dependency on external tooling (lattice/lat) and provides tighter
integration with the Claude Code ecosystem.

The integration enables LLMC to automatically discover, claim, execute, and
complete tasks stored in the `~/.claude/tasks` directory hierarchy. LLMC will
directly read and write task JSON files rather than spawning Claude Code
instances for task management, providing lower latency and more predictable
behavior.

---

## Task File Format

### Directory Structure

Tasks are stored in the global `~/.claude/tasks` directory, which is shared
across all Claude Code sessions on the machine. Tasks are organized into
subdirectories identified by a task list ID. Each task is a single JSON file
named with a numeric identifier.

The directory structure follows the pattern:
`~/.claude/tasks/<task_list_id>/<id>.json`

Claude Code creates task list directories automatically when tasks are filed. By
default, Claude Code uses session UUIDs as task list IDs, but the
`CLAUDE_CODE_TASK_LIST_ID` environment variable can override this to use a
specific named directory.

### Task List Configuration

Each LLMC project uses a single, named task list to store all its tasks. The
task list ID is configured in `config.toml` under the `[auto]` section as
`task_list_id`. This value serves as both the directory name under
`~/.claude/tasks/` and the environment variable value passed to workers.

For example, with `task_list_id = "dreamtides"`:
- Tasks are stored in `~/.claude/tasks/dreamtides/`
- Workers are started with `CLAUDE_CODE_TASK_LIST_ID=dreamtides`
- All workers and the overseer share this same task list

This isolation ensures that multiple LLMC instances managing different projects
can coexist on the same machine without interfering with each other's tasks.

### Worker Task List Binding

When LLMC starts auto worker Claude Code sessions, it sets the
`CLAUDE_CODE_TASK_LIST_ID` environment variable to the configured
`task_list_id` value. This ensures that:

1. All workers share the same task list
2. Tasks created by workers via TaskCreate appear in the project's task
   directory where LLMC can discover them
3. The overseer session uses the same task list, so remediation tasks are
   visible to auto mode

Without this binding, workers would create tasks in session-specific UUID
directories that LLMC would never discover.

### JSON Schema

Each task file contains a JSON object with the following fields:

**Required Fields:**
- `id`: String. The task identifier, matching the filename without extension.
- `subject`: String. A brief imperative title describing what needs to be done.
- `description`: String. Detailed requirements and context for completing the
  task.
- `status`: String. One of `pending`, `in_progress`, or `completed`.
- `blocks`: Array of strings. Task IDs that cannot start until this task
  completes.
- `blockedBy`: Array of strings. Task IDs that must complete before this task
  can start.

**Optional Fields:**
- `activeForm`: String. Present continuous form of the subject for display
  during execution.
- `owner`: String. The worker name currently assigned to this task.
- `metadata`: Object. Arbitrary key-value pairs for extensibility.

### Metadata Extensions

LLMC uses the `metadata` object to store priority and label information until
Claude Code provides native support for these features.

**Priority:** Stored as `metadata.priority` with integer values from 0 to 4,
where 0 is highest priority and 4 is lowest. Tasks without a priority value are
treated as priority 3 (low).

**Label:** Stored as `metadata.label` as a string value. Tasks may have at most
one label. Labels are used for context injection and concurrency optimization.
Tasks without a label are treated as unlabeled.

---

## Task Discovery

### Scan Process

When LLMC auto mode needs to find work for an idle worker, it performs a
filesystem scan of the configured task list directory:
`~/.claude/tasks/<task_list_id>/`

Each discovered `.json` file in this directory is parsed as a task. Files that
fail to parse as valid JSON or lack required fields are logged as errors and
skipped. The scan happens at the start of each orchestration cycle, ensuring
LLMC sees newly created tasks and status changes made by external processes
(including tasks created by Claude Code via TaskCreate).

Tasks in other directories (other projects, other Claude Code sessions) are not
scanned and do not affect LLMC operation.

### Eligibility Criteria

A task is eligible for assignment when all of the following conditions are met:

1. The task status is `pending`.
2. The task has no owner assigned, or the owner field is empty/null.
3. The `blockedBy` array is empty, or all tasks referenced in `blockedBy` have
   status `completed`.
4. The task file is readable and parseable.

Tasks that are `in_progress` with an owner matching a currently active auto
worker are considered claimed and are not reassigned, even if the worker has
restarted. Tasks that are `in_progress` but have an owner that does not match
any active auto worker may be considered orphaned and require special handling
as described in the failure handling section.

### Dependency Resolution

Before determining task eligibility, LLMC builds an in-memory graph of task
dependencies by scanning all tasks and their `blocks` and `blockedBy` fields. A
task is only eligible if every task ID in its `blockedBy` array resolves to a
task with status `completed`.

If a `blockedBy` reference points to a task ID that does not exist (file not
found), LLMC treats this as a blocking condition and the task remains
ineligible. This prevents execution of tasks that depend on deleted or corrupted
dependency specifications.

Circular dependencies are detected during the graph construction phase. If a
circular dependency is detected, all tasks in the cycle are logged as errors and
treated as ineligible until the cycle is manually broken.

---

## Task Selection Algorithm

When multiple eligible tasks exist, LLMC selects tasks using a multi-factor
algorithm designed to maximize throughput and minimize conflicts.

### Selection Factors in Order

1. **Distinct Label Preference**: When assigning a task to a worker, LLMC
   prefers tasks whose label differs from all tasks currently being worked on by
   other auto workers. This reduces the likelihood of merge conflicts when
   workers modify similar areas of the codebase. If no distinct-label tasks are
   available, this factor is ignored.

2. **Priority**: Among remaining candidates, LLMC selects tasks with the lowest
   priority value (highest urgency). Priority 0 tasks are selected before
   priority 1, and so on.

3. **Creation Order**: Among tasks with equal priority, LLMC selects the task
   with the lowest numeric ID. Since IDs are assigned sequentially, lower IDs
   correspond to older tasks, implementing FIFO ordering within priority levels.

### Distinct Label Selection Detail

The distinct label algorithm maintains awareness of which labels are currently
being worked on across all auto workers. When selecting a task for worker N:

1. Collect the set of labels from tasks currently assigned to all other auto
   workers (workers 1 through N-1 and N+1 through max).
2. Filter eligible tasks to those whose label is not in this set, or which have
   no label.
3. If this filtering produces an empty set, disable the distinct label
   preference and consider all eligible tasks.
4. Apply priority and creation order selection to the filtered (or unfiltered)
   set.

Tasks without labels are considered "distinct" from all labeled tasks and from
each other. This means unlabeled tasks can always be assigned regardless of what
other workers are doing, but labeled tasks will prefer to avoid duplication.

### Concurrency Safety

The selection algorithm runs atomically within a single orchestration cycle.
LLMC holds exclusive access to task state during selection, preventing race
conditions between workers. The filesystem is the source of truth, and LLMC
reads the latest state at the start of each cycle.

---

## Task Lifecycle Management

### Claiming a Task

When LLMC assigns a task to a worker, it immediately writes the updated task
file with:
- `status` changed from `pending` to `in_progress`
- `owner` set to the worker name (e.g., `auto-1`)

This write happens before sending the task prompt to the worker, ensuring that
if the daemon crashes between claiming and prompting, the task remains claimed
and can be recovered.

### Task Completion

When a worker's changes are successfully accepted (merged to master), LLMC
updates the task file with:
- `status` changed from `in_progress` to `completed`
- `owner` cleared (set to null or removed)

The task file is not deleted. Completed tasks remain on disk for historical
reference and to satisfy dependency checks from other tasks.

### Task Failure

If a worker fails to complete a task (enters error state, daemon shuts down,
etc.), the task handling depends on the failure type:

**Recoverable failures** (worker crash, session restart): The task remains
`in_progress` with the worker as owner. When the worker recovers, it can resume
or be reassigned the same task.

**Hard failures** (daemon shutdown for remediation): Tasks remain `in_progress`.
After remediation, the daemon restarts and workers can resume their assigned
tasks based on the owner field.

**Explicit rejection** (human intervention): If a human resets a worker or marks
a task as failed, LLMC sets the task status back to `pending` and clears the
owner, making it eligible for reassignment.

### No Changes Handling

If a worker completes its task but produces no code changes (status becomes
`no_changes`), LLMC still marks the task as `completed`. The task was executed
successfully; it simply didn't require code modifications.

---

## Context Injection System

### Prologue and Epilogue Concept

The context injection system allows additional text to be prepended (prologue)
or appended (epilogue) to task descriptions based on the task's label. This
enables label-specific instructions, coding standards, or context without
duplicating text across multiple task files.

### Configuration Location

Context injection rules are defined in a TOML file stored within the project
repository at `.claude/task_context.toml`. This location ensures the
configuration is version-controlled alongside the codebase and can be customized
per-project.

### Configuration Format

The configuration file defines context rules keyed by label name. Each label
entry may specify a prologue, an epilogue, or both. The configuration also
supports a default entry that applies to tasks without a label or with labels
not explicitly configured.

The file structure uses TOML tables where each table name is a label, containing
optional `prologue` and `epilogue` string fields. A special `[default]` table
provides fallback content.

### Content Resolution

When building a task prompt, LLMC resolves context in this order:

1. Look up the task's label in the configuration file.
2. If found, use that label's prologue and epilogue values.
3. If not found, use the `[default]` prologue and epilogue values.
4. If no default exists, use empty strings for both.

### Prompt Assembly

The final prompt sent to the worker is assembled as:

1. Standard LLMC task preamble (worktree location, instructions, etc.)
2. Resolved prologue (if non-empty)
3. Task subject and description from the task file
4. Resolved epilogue (if non-empty)

Line breaks are inserted between sections to ensure clean formatting.

### Default Context

If a task has no label set in its metadata, the context injection system uses
the `[default]` table from the configuration file. This provides baseline
instructions that apply to all tasks regardless of their specific focus area.

Projects that do not need label-based differentiation can rely entirely on the
default context, applying the same prologue and epilogue to every task.

---

## Configuration Changes

### Removed Configuration

The following configuration options are removed:

- `[auto].task_pool_command`: No longer needed; tasks are discovered from
  filesystem.
- Any CLI flags related to task pool command override.

### New Configuration

The `[auto]` section gains the following options:

**`task_list_id`** (required for task integration, string): The task list
identifier used as the directory name within `~/.claude/tasks/`. LLMC only
discovers and manages tasks within this directory. Example: `dreamtides`. This
value is also set as `CLAUDE_CODE_TASK_LIST_ID` when starting worker sessions.

**`tasks_root`** (optional, string): Override the default `~/.claude/tasks`
root directory. Defaults to `~/.claude/tasks`. Useful for testing or
non-standard configurations.

**`orphan_claim_timeout_secs`** (optional, integer): How long a task can remain
`in_progress` with an owner that doesn't match any active worker before being
considered orphaned. Defaults to 3600 (1 hour).

**`context_config_path`** (optional, string): Override the path to the task
context configuration file. Defaults to `.claude/task_context.toml` relative to
`repo.source`. This file remains in the project repository since context
injection rules are project-specific.

### Migration Path

Existing LLMC installations using `task_pool_command` continue to function until
explicitly migrated. The presence of `task_pool_command` in configuration takes
precedence; the new task discovery system only activates when
`task_pool_command` is absent and `task_list_id` is configured.

A migration involves:
1. Removing `task_pool_command` from `config.toml`
2. Adding `task_list_id = "<project-name>"` to the `[auto]` section
3. Creating the `~/.claude/tasks/<project-name>/` directory
4. Migrating existing tasks from the previous system (lattice) to task JSON
   files within that directory
5. Creating `.claude/task_context.toml` in the project repository if
   label-based context injection is desired

---

## Failure Handling

### Task File Access Failures

**File not readable**: If a task file cannot be read (permissions, disk error),
log a warning and skip the task. Do not treat this as a hard failure; other
tasks may still be processable.

**Invalid JSON**: If a task file contains malformed JSON, log a warning with the
filename and parse error. Skip the task and continue.

**Missing required fields**: If a task file lacks required fields (`id`,
`subject`, `description`, `status`, `blocks`, `blockedBy`), log a warning and
skip the task.

**Write failures**: If LLMC cannot write an updated task file (claim,
completion), treat this as a hard failure and initiate daemon shutdown. The
overseer can investigate permissions or disk issues.

### Dependency Resolution Failures

**Missing dependency**: If a task references a `blockedBy` task ID that doesn't
exist, the dependent task remains ineligible. Log an info-level message
indicating the unresolved dependency.

**Circular dependency**: If circular dependencies are detected, log an error
listing all task IDs in the cycle. Mark all tasks in the cycle as ineligible. Do
not shut down; continue processing non-circular tasks.

### Orphaned Task Handling

A task is considered orphaned when:
- Status is `in_progress`
- Owner field is set to a value
- No active auto worker has that name
- The task has been in this state longer than `orphan_claim_timeout_secs`

When an orphaned task is detected:
1. Log a warning indicating the orphaned task and its age
2. Reset the task to `pending` status with no owner
3. The task becomes eligible for reassignment in the next cycle

This handles cases where a worker was removed, renamed, or the daemon was
reconfigured without cleaning up task claims.

### Concurrent Modification

External processes (human editors, other tools) may modify task files while LLMC
is running. LLMC handles this by:

1. Re-reading task files at the start of each orchestration cycle
2. Accepting external status changes (e.g., human marking a task complete)
3. Detecting if a task was reassigned externally and yielding to the external
   assignment

If LLMC attempts to write a task file but the file has been modified since it
was read (detected via mtime comparison), LLMC re-reads the file and reconsiders
the operation. If the re-read shows the task is no longer eligible (already
claimed, completed, etc.), LLMC abandons the operation and moves on.

### Context Configuration Failures

**Missing context file**: If `.claude/task_context.toml` does not exist, proceed
with empty prologue and epilogue for all tasks. Log an info-level message on
first access.

**Invalid TOML**: If the context configuration file contains invalid TOML, log
an error and proceed with empty context. Do not shut down.

**Missing label entry**: If a task's label is not found in the configuration and
no default exists, use empty context. This is expected behavior, not an error.

---

## Logging Strategy

### Log Locations

Task-related logging is written to `logs/tasks.log` within the LLMC root
directory, separate from the existing auto mode and daemon logs. This provides a
dedicated log for task lifecycle events.

### Event Categories

**Task Discovery Events**: Logged at debug level. Includes scan start/end,
number of tasks found, parse errors, eligibility determination results.

**Task Assignment Events**: Logged at info level. Includes which task was
assigned to which worker, selection algorithm factors (distinct label used,
priority, ID).

**Task Completion Events**: Logged at info level. Includes task ID, worker name,
outcome (completed, no_changes), duration from claim to completion.

**Task Failure Events**: Logged at warn or error level depending on severity.
Includes task ID, failure type, any relevant error messages.

**Context Injection Events**: Logged at debug level. Includes which label was
used, whether default was applied, prologue/epilogue lengths.

**Orphan Detection Events**: Logged at warn level. Includes task ID, claimed
owner, time since claim.

### Structured Logging

All task-related log entries include structured fields for machine parsing:
- `task_id`: The task identifier
- `task_list_id`: The configured task list identifier
- `worker_name`: The worker involved (if applicable)
- `event_type`: Categorization of the log event
- `timestamp_unix`: Unix timestamp of the event

### Log Rotation

Task logs follow the same rotation policy as other LLMC logs: size-based
rotation with retention of recent files. The specific parameters are inherited
from the existing logging configuration.

---

## Testing Strategy

### Unit Testing

**Task parsing**: Test JSON deserialization for valid tasks, tasks with optional
fields omitted, tasks with metadata extensions, and malformed JSON. Verify error
messages for parse failures.

**Eligibility determination**: Test the eligibility check logic with various
combinations of status, owner, and blockedBy states. Include edge cases: empty
blockedBy array, all blockers completed, some blockers incomplete, missing
blocker tasks.

**Selection algorithm**: Test priority ordering, ID-based tiebreaking, and
distinct label selection. Create scenarios with multiple eligible tasks having
various priority and label combinations.

**Dependency graph construction**: Test cycle detection with simple cycles (A
blocks B, B blocks A) and complex cycles involving multiple tasks. Test handling
of missing task references.

**Context resolution**: Test label lookup, default fallback, and missing
configuration file handling.

### Integration Testing

**Filesystem operations**: Test task file discovery in the task list directory,
handling of non-task files in the directory, and concurrent modification
detection.

**Claim/release cycle**: Test the full lifecycle of claiming a task, executing
it, and marking it complete. Verify file contents at each stage.

**Orphan recovery**: Test detection and recovery of orphaned tasks after
simulated daemon restart without the original worker.

**Multi-worker scenarios**: Test distinct label selection with multiple workers,
verifying that workers receive tasks with different labels when available.

**Environment variable binding**: Verify that worker sessions are started with
`CLAUDE_CODE_TASK_LIST_ID` set correctly, and that tasks created by workers via
TaskCreate appear in the expected task list directory.

### Mock Task Pool for Comparison Testing

During migration, maintain the ability to run both the old task pool command
system and the new direct integration in parallel (in separate test instances).
This enables comparison testing to verify behavioral equivalence.

### Manual Testing Scenarios

**Happy path**: Create several tasks with varying priorities and labels. Start
auto mode with multiple workers. Verify tasks are claimed and completed in
expected order.

**Dependency chain**: Create tasks A, B, C where B depends on A and C depends on
B. Verify execution order respects dependencies.

**Conflict avoidance**: Create multiple tasks with the same label and verify
workers prefer to work on differently-labeled tasks when available.

**External modification**: While auto mode is running, manually edit a task file
to mark it complete. Verify LLMC detects the change and does not attempt to
reassign.

**Context injection**: Create tasks with various labels and verify the correct
prologue/epilogue text appears in worker prompts.

---

## Removal of Lattice Integration

### Scope of Removal

This migration replaces all lattice (`lat`) functionality used for task
management in LLMC auto mode. The `task_pool_command` configuration, which
typically invoked `lat ready` or similar commands, is removed entirely.

### Commands Not Affected

LLMC commands that interact with workers directly (`llmc start`, `llmc review`,
`llmc accept`, etc.) continue to function unchanged. These commands do not
depend on the task discovery mechanism.

The overseer system continues to function, using the same failure detection and
remediation approach. The only change is that task pool command failures are no
longer possible since that command no longer exists.

### Data Migration

Existing lattice tasks must be manually migrated to the Claude tasks format.
This involves:

1. For each open lattice task, create a corresponding JSON file in
   `~/.claude/tasks/<task_list_id>/<id>.json`
2. Map lattice fields to Claude task fields: title to subject, body to
   description, status mapping, dependency IDs
3. Set priority and label in metadata if the lattice task had equivalent
   concepts
4. Verify dependency references resolve correctly in the new system

A migration script may be provided as a convenience but is not part of the core
LLMC implementation.

---

## State File Changes

### New State Fields

The `State` struct gains the following fields to track task-related information:

**`active_task_ids`**: Map from worker name to task ID currently being worked
on. Used to populate the distinct label set during task selection.

**`task_claim_times`**: Map from task ID to Unix timestamp when the task was
claimed. Used for orphan detection.

### Removed State Fields

No state fields are removed. Existing fields related to task pool (backoff
counters, etc.) become unused but are retained for backward compatibility during
migration.

### State Persistence

Task state is primarily stored in the task JSON files themselves, not in LLMC's
`state.json`. The state file only caches information needed for fast
decision-making within a single daemon run. If the state file is lost, LLMC can
reconstruct task assignments by scanning task files and matching owners to
workers.

---

## Overseer Integration

### Failure Context

When the daemon shuts down due to a task-related failure, the overseer's
remediation prompt includes:

- The task ID that caused the failure
- The task file path
- The specific failure type (write failure, parse error, etc.)
- Recent entries from `logs/tasks.log`

### Remediation Actions

The overseer may be instructed (via `remediation_prompt` in configuration) to:

- Fix malformed task JSON files
- Resolve circular dependencies by editing blockedBy fields
- Clear orphaned task claims
- Restore missing task files from version control

### Recovery After Remediation

After successful remediation, the daemon restarts and resumes normal task
discovery. Tasks that were `in_progress` are either resumed by their assigned
worker (if the worker still exists) or eventually recovered via orphan
detection.

---

## Performance Considerations

### Scan Frequency

Task discovery scans happen once per orchestration cycle (configured via
`patrol_interval_secs`). With typical intervals of 30 seconds and task counts in
the hundreds, filesystem scan overhead is negligible.

### Caching

LLMC may cache parsed task data within a single orchestration cycle to avoid
re-reading files multiple times. The cache is invalidated at the start of each
cycle to ensure external modifications are detected.

### Large Task Counts

For repositories with thousands of tasks, the linear scan may become noticeable.
Future optimization could include:
- Indexing completed tasks separately from pending tasks
- Watching the filesystem for changes instead of full scans
- Caching task metadata in a local database

These optimizations are not included in the initial implementation but the
architecture should not preclude them.

---

## Security Considerations

### Task File Trust

Task files are assumed to be trusted input created by authorized users. LLMC
does not sanitize or escape task descriptions before sending them to workers.
Malicious task content could potentially inject unintended instructions into
worker prompts.

Mitigation: Task files should be treated with the same trust level as code in
the repository. Since tasks live in `~/.claude/tasks`, access is controlled by
filesystem permissions on the user's home directory.

### Filesystem Access

LLMC reads and writes files only within the configured task list directory
(`~/.claude/tasks/<task_list_id>/`) and the LLMC root. It does not scan other
task lists or follow symlinks outside these directories.

### Metadata Injection

The metadata field is passed through without validation. Arbitrary keys can be
set. LLMC only interprets specific keys (`priority`, `label`) and ignores
others. Unknown keys do not affect behavior.
