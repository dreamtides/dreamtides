---
lattice-id: LBSWQN
name: auto-worker-stuck-offline-state-despite-
description: Auto worker stuck in 'offline' state despite Claude Code being ready for input
parent-id: LB6WQN
task-type: bug
priority: 1
labels:
- llmc-auto
- auto-mode
- bug
created-at: 2026-01-22T02:55:25.045522Z
updated-at: 2026-01-22T02:55:25.045522Z
---

# Bug: Auto Worker Stuck in 'offline' State

## Summary

When starting auto mode with `llmc up --auto`, the auto worker gets created and the tmux session starts with Claude Code running, but the worker never transitions from "offline" to "idle" state. This prevents task assignment.

## Environment

- Isolated test environment: `/tmp/llmc-auto-lifecycle-test-23866`
- LLMC_ROOT set correctly
- Daemon running with PID 24986

## Steps to Reproduce

1. Initialize isolated LLMC environment:
   ```bash
   export TEST_DIR="/tmp/llmc-auto-lifecycle-test-XXXX"
   export LLMC_ROOT="$TEST_DIR"
   llmc init --source ~/Documents/GoogleDrive/dreamtides --target "$TEST_DIR"
   ```

2. Configure auto mode in config.toml:
   ```toml
   [auto]
   task_pool_command = "$LLMC_ROOT/test_scripts/task_pool.sh"
   concurrency = 1
   ```

3. Start daemon:
   ```bash
   llmc up --auto &
   ```

4. Wait and check status:
   ```bash
   sleep 60
   llmc status
   ```

## Expected Behavior

- Auto worker should transition from "offline" to "idle" within a few seconds
- Task pool should be polled
- Task should be assigned to idle worker

## Actual Behavior

- Auto worker stays "offline" indefinitely (tested for 90+ seconds)
- Tmux session is running with Claude Code at the welcome screen, ready for input
- No tasks are assigned
- All log files are empty (auto.log, task_pool.log, llmc.jsonl)
- No logs being written at all

## Evidence

```
llmc status (after 90 seconds):
DAEMON
──────
  Status: running  PID: 24986  Uptime: 1m

AUTO WORKERS
────────────
auto-1       offline         llmc/auto-1         1m
```

Claude Code peek shows it's ready for input:
- Shows welcome banner
- Has prompt line: "⏵⏵ bypass permissions on"
- Session exists: `llmc-llmc-auto-lifecycle-test-23866-auto-1`

## Possible Causes

1. Daemon patrol loop not running or not detecting ready sessions
2. Session readiness detection logic not working for auto workers
3. Log file initialization issue
4. Worker state machine not transitioning correctly

## Additional Notes

The daemon stdout shows initialization completed successfully:
```
Starting LLMC daemon...
✓ IPC listener started
Reconciling workers with state...
✓ All workers started
Entering auto mode loop (Ctrl-C to stop)...
✓ 1 auto worker(s) initialized
```

But then nothing happens after that - no patrol activity, no state transitions.