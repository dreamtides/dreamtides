---
lattice-id: LDBWQN
name: auto-overseer-design
description: Technical design for llmc auto mode and overseer command
created-at: 2026-01-21T01:25:55.793081Z
updated-at: 2026-01-21T01:25:55.793113Z
---

# Auto Mode and Overseer Technical Design

## Overview

This document specifies two new LLMC features for autonomous operation:

- **`llmc up --auto`**: A daemon mode that autonomously assigns tasks to
  workers, accepts completed work, and shuts down gracefully on errors
- **`llmc overseer`**: A higher-level supervisor that monitors the daemon,
  detects failures, and uses Claude Code to remediate issues

These features transform LLMC from a human-in-the-loop system to a fully
autonomous task execution pipeline.

---

## Part 1: Auto Mode (`llmc up --auto`)

### Behavioral Changes

- Normal `llmc up` philosophy: "stay alive at all costs" - absorb errors,
  continue running
- Auto mode philosophy: "execute graceful shutdown on any error" - preserve
  worktree state, terminate cleanly
- Auto mode bypasses the human review step (`llmc review`); changes are accepted
  automatically
- Auto workers are segregated from manual workers in status display and cannot
  receive `llmc start` tasks

### Configuration

#### New TOML Section: `[auto]`

- `task_pool_command` (required)
  - Shell command that prints a new task description to stdout
  - Should mark tasks as "in progress" internally so subsequent calls return
    different tasks
  - Exit code 0 with empty stdout: no tasks available, daemon waits
  - Exit code non-zero: error condition, triggers shutdown
  - Command runs in caller's shell environment ($PATH, working directory, env
    vars)
  - Stdout logged to `logs/task_pool.log`

- `concurrency` (optional, default: 1)
  - Number of auto workers to run simultaneously
  - Auto workers are created on demand if they don't exist (named `auto-1`,
    `auto-2`, etc.)

- `post_accept_command` (optional)
  - Shell command invoked after successfully rebasing a worker's changes onto
    master
  - May be long-running (tests, validation, deployment)
  - Daemon blocks until completion before proceeding
  - Exit code non-zero: error condition, triggers shutdown
  - Stdout logged to `logs/post_accept.log`

- `restart_cooldown_secs` (optional, default: 60)
  - Minimum time daemon must stay healthy after start before errors no longer
    trigger termination
  - Used by overseer to detect failure spirals

- Inherits all `[defaults]` options (model, skip_permissions, allowed_tools,
  etc.)
  - Per-worker overrides not supported for auto workers

#### CLI Flags

- `--auto`: Enable auto mode
- `--task-pool-command <CMD>`: Override config, required if not in config
- `--concurrency <N>`: Override config
- `--post-accept-command <CMD>`: Override config
- Flags take precedence over TOML configuration

### Auto Workers

#### Naming and Creation

- Auto workers named `auto-1`, `auto-2`, ..., `auto-N` (N = concurrency)
- Created automatically if missing when daemon starts
- Use `llmc add auto-1 --excluded-from-pool` semantics internally
- Worktrees created at `.worktrees/auto-1`, etc.
- TMUX sessions: `llmc-auto-1`, etc.

#### Status Display

- `llmc status` shows auto workers in a separate section: "Auto Workers"
- Each auto worker shows: name, status, current task (truncated), time in state
- Summary line: "Auto Mode: N workers, M tasks completed, L errors"

#### Command Restrictions

- `llmc start` rejects auto workers with clear error message
- All other commands work normally: `attach`, `peek`, `reset`, `nuke`, `pick`,
  etc.
- `llmc nuke --all` removes auto workers; `llmc reset --all` resets them

### Daemon Loop

#### Startup Sequence

1. Validate required configuration (task_pool_command present)
2. Write registration record to `.llmc/daemon.json`:
   - `pid`: Process ID
   - `start_time_unix`: Unix timestamp
   - `instance_id`: Random UUID
   - `log_file`: Path to daemon log
3. Create/verify auto workers exist
4. Start heartbeat thread (updates `.llmc/auto.heartbeat` every 5 seconds)
5. Enter main loop

#### Main Loop (Per Cycle)

1. Check for shutdown signal (Ctrl-C)
2. For each idle auto worker:
   - Execute `task_pool_command`
   - If stdout empty: skip worker this cycle
   - If non-zero exit: log error, initiate shutdown
   - If stdout non-empty: assign task to worker (same as `llmc start`)
3. For each worker that has completed (needs_review or no_changes):
   - If no_changes: reset worker to idle, continue
   - Execute accept workflow:
     - Rebase onto master
     - If conflict: log error, initiate shutdown
     - Squash commits, strip attribution
     - Fast-forward merge to master
   - If `post_accept_command` configured:
     - Execute command
     - If non-zero exit: log error, initiate shutdown
   - Reset worker to idle
4. Run patrol (session health, rebasing in-flight workers, etc.)
5. Sleep patrol_interval_secs

#### Error Handling

- Any unexpected error triggers shutdown sequence:
  - Log detailed error with context (worker, state, command output)
  - Set `shutdown_requested` flag
  - Allow current operations to drain (with timeout)
  - Execute `llmc down` logic: graceful Ctrl-C to workers, then kill
  - Preserve all worktree state (no cleanup)
  - Exit with non-zero code

- Error conditions that trigger shutdown:
  - `task_pool_command` returns non-zero
  - `post_accept_command` returns non-zero
  - Worker enters error state
  - Rebase conflict during accept
  - State file corruption
  - TMUX session unexpectedly missing
  - Hook IPC failures

#### Heartbeat Mechanism

- Background thread updates `.llmc/auto.heartbeat` every 5 seconds
- File contains: `{"timestamp_unix": N, "instance_id": "UUID"}`
- Atomic write (temp file + rename)
- Heartbeat thread survives log I/O issues
- Missing/stale heartbeat indicates daemon hang

### Logging

- All existing logging continues
- New dedicated log files:
  - `logs/task_pool.log`: All task pool command invocations and output
  - `logs/post_accept.log`: All post accept command invocations and output
  - `logs/auto.log`: Auto-specific daemon events (task assignments, accepts,
    errors)
- Each log entry includes timestamp, worker name (if applicable), and full
  command output

---

## Part 2: Overseer (`llmc overseer`)

### Behavioral Overview

- Long-running supervisor process that manages the entire auto pipeline
- Starts daemon, monitors health, terminates on failure, remediates via Claude
  Code
- Designed for unattended multi-day operation
- Single point of control for autonomous LLMC operation

### Architecture

#### Components

1. **Overseer Process** (main process)
   - Starts daemon via shell command
   - Monitors daemon health via logs and heartbeat
   - Detects failures and coordinates remediation
   - Prints high-level status updates to stdout

2. **Daemon Process** (`llmc up --auto`)
   - Standard auto daemon as specified above
   - No awareness of overseer

3. **Overseer Claude Code Session**
   - Separate TMUX session: `llmc-overseer`
   - Claude Code running in main project directory (not a worktree)
   - Used for remediation when daemon fails
   - Restarted if it crashes

### Status Display

- `llmc status` includes new "Overseer" section when active:
  - Overseer state: Running, Remediating, Terminated
  - Daemon PID and uptime
  - Current remediation attempt (if any)
  - Last N status events

### Startup Sequence

1. Validate configuration
2. Create overseer TMUX session if missing
3. Start overseer Claude Code process (if not running)
4. Start daemon: execute `llmc up --auto` via shell (not direct Rust call)
   - Important: Uses caller's $PATH so modified `llmc` binaries are picked up
5. Watch for daemon registration (`.llmc/daemon.json`)
6. Enter monitor loop

### Monitor Loop

#### Health Checks (Every N Seconds)

1. **Process Identity Verification**
   - Read `.llmc/daemon.json`
   - Verify PID matches expected
   - Verify `start_time_unix` matches
   - Verify `instance_id` matches
   - If mismatch: daemon restarted unexpectedly, investigate

2. **Heartbeat Check**
   - Read `.llmc/auto.heartbeat`
   - If file missing or timestamp stale (>30 seconds): daemon hung

3. **Log Monitoring**
   - Tail daemon log file
   - Parse for ERROR and WARN level entries
   - Track error frequency and patterns

4. **Progress Tracking**
   - Monitor for task completions
   - Detect stalled state (no progress for extended period)

#### Failure Detection

- Missing heartbeat for >30 seconds
- Daemon process terminated (PID no longer exists)
- PID reuse detection (same PID, different start time or instance ID)
- Repeated errors in logs (>N errors in M seconds)
- Stalled progress (no task completions for configurable period)

### Daemon Termination Protocol

When failure detected:

1. Log failure details with full context
2. Wait grace period (10 seconds) - daemon may self-recover
3. Re-verify failure persists and process identity matches
4. Send SIGTERM to daemon PID
5. Wait grace period (30 seconds) for graceful shutdown
6. If still running: send SIGKILL
7. Verify daemon fully terminated
8. Enter remediation mode

### Remediation

#### Remediation Trigger

- Daemon terminated (by overseer or self-terminated with error)
- Overseer enters remediation mode

#### Prompt Construction

1. Start with user-configured remediation prompt from TOML:
   - `[overseer].remediation_prompt`: High-level instructions and context
   - Example: "You are debugging LLMC. Check logs, fix issues, test changes."

2. Append structured error context:
   - Failure type (heartbeat, process death, log errors, stall)
   - Daemon registration info (PID, start time, instance ID)
   - Last N lines of daemon log
   - Last N lines of task pool log
   - Last N lines of post accept log
   - Current worker states
   - Git status summary

3. Append recovery instructions:
   - "After fixing the issue, exit normally. The overseer will restart the
     daemon."
   - "If the issue cannot be fixed, create a file
     `.llmc/manual_intervention_needed.txt` with explanation."

#### Remediation Execution

1. Send `/clear` to overseer Claude Code session
2. Send constructed prompt
3. Monitor for completion via hooks (same mechanism as worker completion
   detection)
4. Wait for Claude Code to exit its task

#### Remediation Logging

- Each remediation logged to `logs/remediation_<timestamp>.txt`:
  - Full constructed prompt
  - Complete Claude Code session transcript including:
    - All tool calls and arguments
    - All tool outputs
    - All Claude responses
  - Final outcome (success/failure)
  - Duration

- This is in addition to normal JSON logging
- Files preserved indefinitely for debugging

### Restart After Remediation

1. Check for `.llmc/manual_intervention_needed.txt`
   - If exists: log contents, terminate overseer with clear message
2. Start daemon via shell: `llmc up --auto`
3. Watch for successful registration
4. Track start time for cooldown calculation

### Failure Spiral Prevention

- If daemon fails within `restart_cooldown_secs` (default: 60s) of last start:
  - This is treated as a failure spiral
  - Remediation does NOT repeat
  - Overseer terminates with detailed error message
  - Human intervention required

- Rationale: Some failures are not code-fixable (disk full, network down, API
  limits)
- Prevents infinite loop of remediation attempts

### TMUX Session Protection

- Existing "kill all tmux sessions" logic in LLMC must exclude `llmc-overseer`
- `llmc down` should NOT terminate overseer session
- `llmc nuke --all` should NOT terminate overseer session
- Only `llmc overseer stop` (new command) terminates overseer

### Configuration

#### New TOML Section: `[overseer]`

- `remediation_prompt` (required for overseer use)
  - User-provided instructions for Claude Code remediation
  - Should include project context, common issues, preferred fixes

- `heartbeat_timeout_secs` (optional, default: 30)
  - How long before missing heartbeat triggers failure

- `log_error_threshold` (optional, default: 5)
  - Number of errors in log within window to trigger failure

- `log_error_window_secs` (optional, default: 60)
  - Time window for error threshold

- `stall_timeout_secs` (optional, default: 3600)
  - How long without task completion before considered stalled

- `restart_cooldown_secs` (optional, default: 60)
  - Minimum healthy runtime before restart no longer considered failure spiral

### CLI Interface

- `llmc overseer`: Start overseer (foreground, Ctrl-C to stop)
- `llmc overseer start`: Start overseer (daemonized, returns immediately)
- `llmc overseer stop`: Terminate overseer and daemon gracefully
- `llmc overseer status`: Show overseer and daemon status

---

## Implementation Considerations

### State File Changes

- Add `auto_mode: bool` to `State`
- Add `auto_workers: Vec<String>` to track which workers are auto-managed
- Add `overseer_active: bool` to indicate overseer presence
- Add `last_task_completion_unix: Option<u64>` for stall detection

### New Files in `.llmc/`

- `daemon.json`: Daemon registration (pid, start_time, instance_id, log_file)
- `auto.heartbeat`: Heartbeat file (timestamp, instance_id)
- `manual_intervention_needed.txt`: Created by remediation Claude if unfixable
- `overseer.json`: Overseer registration (similar to daemon.json)

### Module Structure

```
src/
  commands/
    up.rs          # Extend with --auto handling
    overseer.rs    # New: overseer command implementation
  auto/
    mod.rs         # Auto mode orchestration
    config.rs      # Auto configuration parsing
    worker.rs      # Auto worker lifecycle
    accept.rs      # Auto accept workflow
    heartbeat.rs   # Heartbeat thread
  overseer/
    mod.rs         # Overseer main loop
    monitor.rs     # Health monitoring
    remediation.rs # Remediation prompt construction and execution
    session.rs     # Overseer Claude Code session management
```

### Testing Strategy

- Unit tests for configuration parsing
- Unit tests for prompt construction
- Integration tests with mock task pool command
- Integration tests with mock post accept command
- Failure injection tests for graceful shutdown
- Overseer tests with mock daemon

### Security Considerations

- Shell commands execute with user privileges
- No additional sandboxing for auto workers (same as manual workers)
- Remediation Claude has full repo access (intentional for fixing issues)
- Log files may contain sensitive task descriptions

### Operational Guidelines

- Start with low concurrency and increase gradually
- Monitor remediation logs for common issues
- Configure stall timeout based on typical task duration
- Review remediation prompt effectiveness periodically
- Set up external alerting on overseer termination

---

## Migration and Compatibility

- Auto mode is entirely opt-in via `--auto` flag
- No changes to existing `llmc up` behavior without flag
- Existing workers can coexist with auto workers
- Configuration additions are purely additive
- State file changes are backward compatible (new optional fields)

## Future Considerations

- Task prioritization in pool
- Dynamic concurrency scaling
- Task timeout configuration
- Partial acceptance (accept some workers, remediate others)
- Metrics and observability integration
- Remote notification on failures
