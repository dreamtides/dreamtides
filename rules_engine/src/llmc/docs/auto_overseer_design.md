---
lattice-id: LDBWQN
name: auto-overseer-design
description: Technical design for llmc auto mode and overseer command
created-at: 2026-01-21T01:25:55.793081Z
updated-at: 2026-01-21T22:31:38.653047Z
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
  - The command is responsible for tracking task state; LLMC does not mark tasks
    as claimed
  - Expectation: subsequent invocations return different tasks (command manages
    its own queue)
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

- Inherits all ` [defaults] ` options (model, skip_permissions, allowed_tools,
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
4. Start sessions only for auto workers that don't already have running sessions
   (sessions may already exist from `reconcile_and_start_workers` in `llmc up`)
5. Start heartbeat thread (updates `.llmc/auto.heartbeat` every 5 seconds)
6. Enter main loop

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
     - If rebase fails (conflict): transition worker to `rebasing` state, send conflict
       prompt, continue (worker resolves conflict, returns to `needs_review`, accept retried)
     - Squash commits, strip attribution
     - Fast-forward merge to master
   - If `post_accept_command` configured:
     - Execute command
     - If non-zero exit: log error, initiate shutdown
   - Reset worker to idle
4. Run patrol (session health, rebasing in-flight workers, etc.)
5. Sleep patrol_interval_secs

#### Error Handling

Two categories of errors:

**Transient failures** (patrol attempts automatic recovery):

- Worker Claude Code crashes → patrol restarts session (up to 2 retries with
  backoff)
- TMUX session disappears → patrol recreates session
- Source repository has uncommitted changes → exponential backoff retry (see
  below)
- Rebase conflict during accept → worker transitions to `rebasing`, resolves
  conflict, accept retried
- If patrol recovery succeeds, daemon continues normally
- If patrol retries exhausted → escalates to hard failure

**Source repository dirty handling:**

When the daemon attempts to accept a worker's changes but the source repository
(the main development directory) has uncommitted changes, the daemon does NOT
shut down. Instead, it implements exponential backoff:

- First detection: wait 60 seconds before retry
- Each subsequent detection: double the wait time (120s, 240s, 480s, ...)
- Maximum backoff: 1 hour
- Backoff state is persisted in `state.json` to survive daemon restarts
- On successful accept or NoChanges, backoff state is cleared
- Prints message to stdout: "Source repository has uncommitted changes. Will
  retry in N seconds."

This allows the daemon to continue running while a developer commits or stashes
their changes in the source repository.

**Hard failures** (immediate shutdown):

- `task_pool_command` returns non-zero
- `post_accept_command` returns non-zero
- Worker enters error state (after patrol retries exhausted)
- Rebase failure during accept (after worker was ready)
- State file corruption
- Hook IPC failures
- Auto workers unexpectedly disappear from state (see below)

**State corruption detection:**

The daemon monitors for state file corruption by tracking the expected number of
auto workers. Each iteration of the main loop verifies that the loaded state
contains the expected auto workers:

- If all auto workers disappear (count goes from N to 0): immediate shutdown
  with detailed error message indicating possible state file corruption
- If some auto workers are missing (count decreases): warning logged
- Common cause: A worker running a test that uses `LLMC_ROOT` incorrectly and
  overwrites the production state file

State saves include trace-level logging of worker counts to aid debugging.

Shutdown sequence for hard failures:

- Log detailed error with context (worker, state, command output)
- Set `shutdown_requested` flag
- Allow current operations to drain (with timeout)
- Execute `llmc down` logic: graceful Ctrl-C to workers, then kill
- Preserve all worktree state (no cleanup)
- Exit with non-zero code

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
- Terminated via Ctrl-C (first Ctrl-C begins graceful shutdown of both overseer
  and daemon; second Ctrl-C forces immediate termination)

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
   - If mismatch: daemon restarted unexpectedly, treat as failure

2. **Heartbeat Check**
   - Read `.llmc/auto.heartbeat`
   - If file missing or timestamp stale (>heartbeat_timeout_secs): daemon hung

3. **Log Monitoring**
   - Tail daemon log file
   - ANY ERROR or WARN level entry triggers immediate failure handling
   - No thresholds or windowing - single warning/error means failure

4. **Progress Tracking**
   - Monitor for task completions
   - Detect stalled state (no progress for stall_timeout_secs)

#### Failure Detection

- Any ERROR or WARN in daemon logs
- Missing heartbeat for >heartbeat_timeout_secs
- Daemon process terminated (PID no longer exists)
- PID reuse detection (same PID, different start time or instance ID)
- Stalled progress (no task completions for stall_timeout_secs)

### Daemon Termination Protocol

When failure detected:

1. Log failure details with full context
2. Immediately send SIGTERM to daemon PID (no waiting for self-recovery)
3. Wait grace period (30 seconds) for graceful shutdown
4. If still running: send SIGKILL
5. Verify daemon fully terminated
6. Enter remediation mode

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
     `.llmc/manual_intervention_needed_<timestamp>.txt` with explanation."

#### Remediation Execution

1. Send `/clear` to overseer Claude Code session
2. Send constructed prompt
3. Monitor for completion via hooks (same mechanism as worker completion
   detection)
4. Wait for Claude Code to exit its task

#### Remediation Console Output

The overseer prints colored messages to stdout to indicate remediation status:

- **Red** (`⚠ Entering remediation mode...`): When remediation starts
- **Green** (`✓ Remediation completed in N.Ns`): On successful completion
- **Green** (`✓ Remediation complete. Restarting daemon...`): Before daemon restart
- **Yellow** (`⚠ Remediation interrupted`): If shutdown signal received
- **Yellow** (`⚠ Remediation issue: ...`): On timeout or other errors

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

1. Check for any `.llmc/manual_intervention_needed_*.txt` files
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
- Only Ctrl-C on overseer process terminates the overseer

### Configuration

#### New TOML Section: `[overseer]`

- `remediation_prompt` (required for overseer use)
  - User-provided instructions for Claude Code remediation
  - Should include project context, common issues, preferred fixes

- `heartbeat_timeout_secs` (optional, default: 30)
  - How long before missing heartbeat triggers failure

- `stall_timeout_secs` (optional, default: 3600)
  - How long without task completion before considered stalled

- `restart_cooldown_secs` (optional, default: 60)
  - Minimum healthy runtime before restart no longer considered failure spiral

### CLI Interface

- `llmc overseer`: Start overseer (foreground)
  - First Ctrl-C: Begin graceful shutdown (terminates daemon cleanly, then exits)
  - Second Ctrl-C: Force immediate termination (exits process immediately)

---

## Failure Scenarios & Mitigation

Recovery classification:

- **AUTO**: System recovers automatically without external intervention
- **AI**: Overseer invokes Claude Code remediation
- **HUMAN**: Requires human intervention; overseer terminates

### Task Pool Command Failures

| # | Failure | Detection | Recovery | Type |
|---|---------|-----------|----------|------|
| 1 | Command returns non-zero exit | Daemon checks exit code | Daemon shuts down; overseer remediates | AI |
| 2 | Command hangs indefinitely | Heartbeat stale | Overseer kills daemon; remediates | AI |
| 3 | Command returns empty repeatedly | N/A - expected | Daemon waits for tasks | AUTO |
| 4 | Command returns duplicate tasks | Not detectable | External issue; may double-process | HUMAN |

### Worker Failures

| # | Failure | Detection | Recovery | Type |
|---|---------|-----------|----------|------|
| 5 | Worker Claude Code crashes (once) | Patrol detects | Patrol restarts session automatically | AUTO |
| 6 | Worker crashes repeatedly | Patrol retries exhausted | Daemon shuts down; overseer investigates | AI |
| 7 | Worker hangs (infinite loop) | Fallback timeout (5 min) | Daemon shuts down; overseer investigates | AI |
| 8 | Worker enters error state | State logged as ERROR | Daemon shuts down; overseer resets worker | AI |
| 9 | TMUX session killed (once) | Patrol health check | Patrol recreates session | AUTO |
| 10 | TMUX session killed repeatedly | Patrol retries exhausted | Daemon shuts down; overseer restarts | AI |
| 11 | Worker worktree git corruption | Git operations fail | Daemon shuts down; overseer recreates worker | AI |
| 12 | Worker code fails tests | post_accept_command non-zero | Daemon shuts down; overseer may fix or revert | AI |

### Accept Workflow Failures

| # | Failure | Detection | Recovery | Type |
|---|---------|-----------|----------|------|
| 13 | Merge conflict during accept | Git rebase conflict status | Worker transitions to `rebasing`, resolves conflict, accept retried | AUTO |
| 14 | Master diverged significantly | Many conflicts | Daemon shuts down; overseer may need retries | AI |
| 15 | Git lock file stuck | Git lock error | Daemon shuts down; overseer removes lock | AI |
| 15a | Source repo has uncommitted changes | Dirty repo check | Exponential backoff retry (60s, 120s, 240s, ..., max 1hr) | AUTO |

### System Resource Failures

| # | Failure | Detection | Recovery | Type |
|---|---------|-----------|----------|------|
| 16 | Disk full | Write operations fail | AI also fails; failure spiral terminates | HUMAN |
| 17 | Out of memory (OOM) | Daemon disappears | Overseer detects; systemic issue | HUMAN |
| 18 | Claude API rate limits | Worker errors | Daemon shuts down; AI may add delays | AI |
| 19 | Network connectivity loss | API calls fail | AI cannot fix external network | HUMAN |

### State & Configuration Failures

| # | Failure | Detection | Recovery | Type |
|---|---------|-----------|----------|------|
| 20 | state.json corruption | JSON parse error | Overseer runs `llmc doctor --rebuild` | AI |
| 20a | state.json overwritten by test | Auto worker count drops to zero | Daemon shuts down with detailed error | AI |
| 21 | config.toml syntax error | TOML parse error | Overseer fixes TOML syntax | AI |
| 22 | Hook IPC socket failure | Socket operations fail | Overseer recreates socket | AI |
| 23 | .llmc directory permissions | File operations fail | Overseer fixes permissions | AI |

### Overseer-Level Failures

| # | Failure | Detection | Recovery | Type |
|---|---------|-----------|----------|------|
| 24 | Overseer Claude session crashes | Overseer monitors session | Overseer restarts Claude automatically | AUTO |
| 25 | Remediation produces invalid fix | Daemon fails within cooldown | Failure spiral; overseer terminates | HUMAN |
| 26 | Remediation hangs | Hook never fires | May hang; consider timeout in future | HUMAN |
| 27 | Daemon restarts unexpectedly | PID/instance_id mismatch | Overseer kills daemon, remediates | AI |

### Summary

| Type | Count | Scope |
|------|-------|-------|
| AUTO | 6 | Empty task pool, single worker/session crash (patrol recovers), overseer session crash, source repo dirty, rebase conflicts during accept |
| AI | 16 | Repeated crashes, severe conflicts, config errors, most operational failures |
| HUMAN | 6 | Resource exhaustion, network, failure spirals |

**Design principle:** Patrol handles transient failures automatically. AI
remediation for persistent operational issues. Human intervention only for
external/environmental issues or failure spirals.

---

## Implementation Considerations

### State File Changes

- Add `auto_mode: bool` to `State`
- Add `auto_workers: Vec<String>` to track which workers are auto-managed
- Add `overseer_active: bool` to indicate overseer presence
- Add `last_task_completion_unix: Option<u64>` for stall detection
- Add `source_repo_dirty_retry_after_unix: Option<u64>` for source repo dirty
  backoff timing
- Add `source_repo_dirty_backoff_secs: Option<u64>` for current backoff value
  (60s, 120s, ...)

### New Files in `.llmc/`

- `daemon.json`: Daemon registration (pid, start_time, instance_id, log_file)
- `auto.heartbeat`: Heartbeat file (timestamp, instance_id)
- `overseer.json`: Overseer registration (similar to daemon.json)
- `manual_intervention_needed_<timestamp>.txt`: Created by remediation if
  unfixable

### Module Structure

All new Rust source files use minimum 2-word names:

```
src/
  commands/
    up_command.rs           # Extend with --auto handling
    overseer_command.rs     # New: overseer command implementation
  auto_mode/
    auto_orchestrator.rs    # Auto mode main orchestration
    auto_config.rs          # Auto configuration parsing
    auto_workers.rs         # Auto worker lifecycle
    auto_accept.rs          # Auto accept workflow
    heartbeat_thread.rs     # Heartbeat background thread
  overseer_mode/
    overseer_loop.rs        # Overseer main loop
    health_monitor.rs       # Health monitoring
    remediation_prompt.rs   # Prompt construction
    remediation_executor.rs # Remediation execution
    overseer_session.rs     # Overseer Claude Code session management
```

### Testing Strategy

- Unit tests for configuration parsing
- Unit tests for prompt construction
- Integration tests with mock task pool command
- Integration tests with mock post accept command
- Failure injection tests for graceful shutdown
- Overseer tests with mock daemon

---

## Migration and Compatibility

- Auto mode is entirely opt-in via `--auto` flag
- No changes to existing `llmc up` behavior without flag
- Existing workers can coexist with auto workers
- Configuration additions are purely additive
- State file changes are backward compatible (new optional fields)

---

## Serena MCP Integration with Worktrees

### Background

Claude Code MCP plugins (like Serena) operate on configured projects, not the
working directory. For LLMC workers in git worktrees, this could cause Serena
to edit the master repository instead of the worktree.

### Solution

LLMC creates a unique Serena project for each worktree:

1. **On worker creation** (`llmc add`, auto worker creation):
   - Creates `.serena/project.yml` with unique `project_name` using the session
     name format `{session_prefix}-{worker}` (e.g., `llmc-adam` or
     `llmc-test-123-adam` for custom LLMC_ROOT)
   - Registers the worktree path in Serena's global config
     (`~/.serena/serena_config.yml`)

2. **On worker reset** (after task completion in auto mode):
   - Recreates `.serena/project.yml` for the fresh worktree
   - Ensures the worktree remains registered with Serena

This ensures that when Claude uses Serena tools, they operate on the correct
worktree project rather than the master repository.

### Implementation Details

- `create_serena_project()` in `commands/add.rs` creates the
  `.serena/project.yml`
- `register_serena_project()` adds the worktree path to Serena's global
  `projects:` list
- `reset_worker_to_idle()` in `auto_mode/auto_accept.rs` calls
  `create_serena_project()`
  after recreating the worktree

### Troubleshooting

If you see uncommitted changes in the master repository that match worktree
commits:

1. Check if the worktree has a `.serena/project.yml` file
2. Verify the project is registered in `~/.serena/serena_config.yml`
3. Ensure the `project_name` in the worktree differs from the master repo

---

## Implementation Order

Tasks should be implemented in this sequence:

1. LDVWQN - Add module structure for auto_mode and overseer_mode
2. LDCWQN - Add auto configuration parsing to config.rs
3. LDFWQN - Implement heartbeat mechanism for auto mode daemon
4. LDUWQN - Create dedicated log files for auto mode operations
5. LDEWQN - Implement auto worker creation and lifecycle management
6. LDGWQN - Implement task pool command execution
7. LDHWQN - Implement auto accept workflow
8. LDIWQN - Implement auto mode daemon main loop
9. LDJWQN - Implement transient vs hard failure handling in auto mode
10. LDDWQN - Add overseer configuration parsing
11. LDKWQN - Implement overseer Claude Code session management
12. LDLWQN - Implement daemon health monitoring for overseer
13. LDMWQN - Implement daemon termination protocol for overseer
14. LDNWQN - Implement remediation prompt construction
15. LDOWQN - Implement remediation execution and logging
16. LDPWQN - Implement overseer main loop with failure spiral detection
17. LDTWQN - Integrate --auto flag into llmc up command
18. LDSWQN - Add llmc overseer CLI command
19. LDQWQN - Update llmc status to display auto workers and overseer state
20. LDRWQN - Protect overseer TMUX session from kill operations
21. LDWWQN - Write integration tests for auto mode
22. LDXWQN - Write integration tests for overseer
