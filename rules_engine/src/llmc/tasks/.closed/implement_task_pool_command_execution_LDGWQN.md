---
lattice-id: LDGWQN
name: implement-task-pool-command-execution
description: Implement task pool command execution
task-type: feature
priority: 1
labels:
- auto-mode
- auto-overseer
- core
blocking:
- LDIWQN
blocked-by:
- LDCWQN
- LDEWQN
created-at: 2026-01-21T04:00:47.924244Z
updated-at: 2026-01-21T05:32:54.517066Z
closed-at: 2026-01-21T05:32:54.517066Z
---

## Overview

Implement the task pool command execution that fetches new tasks for auto workers by running a user-configured shell command.

## Implementation Steps

1. **Create task_pool.rs** under `src/auto_mode/`:
   - Define `TaskPoolResult` enum: `Task(String)`, `NoTasksAvailable`, `Error(String)`
   - Implement `execute_task_pool_command(command: &str) -> Result<TaskPoolResult>`

2. **Shell command execution**:
   - Execute command using the caller's shell environment
   - Inherit PATH, working directory, and environment variables
   - Capture stdout for task content
   - Capture stderr for error context
   - Handle exit codes:
     - Exit 0 + non-empty stdout = task available
     - Exit 0 + empty stdout = no tasks available (not an error)
     - Non-zero exit = error condition

3. **Logging**:
   - Create dedicated `logs/task_pool.log` file
   - Log each invocation with timestamp
   - Log full stdout and stderr output
   - Log exit code and duration

4. **Integration with daemon loop**:
   - Called once per idle auto worker per cycle
   - On successful task fetch, pass task to worker assignment logic
   - On no tasks available, skip worker this cycle
   - On error, trigger daemon shutdown

5. **Error handling**:
   - Command not found should produce clear error
   - Timeout handling (consider adding configurable timeout in future)
   - Capture and log any panic/crash in command

## Acceptance Criteria

- Task pool command executes in correct shell environment
- Empty stdout (exit 0) correctly interpreted as "no tasks"
- Non-zero exit triggers shutdown with detailed logs
- All invocations logged to dedicated log file
