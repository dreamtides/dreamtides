---
lattice-id: LB7WQN
name: test-scenario-basic-auto-worker-lifecycl
description: 'Test Scenario: Basic Auto Worker Lifecycle'
parent-id: LB5WQN
task-type: task
priority: 2
labels:
- testing
- manual-test
- scenario
- auto-mode
created-at: 2026-01-21T21:59:20.029973Z
updated-at: 2026-01-21T21:59:20.029973Z
---

# Test Scenario: Basic Auto Worker Lifecycle

## Objective

Verify that auto mode correctly creates auto workers, assigns tasks from the task pool, 
and handles task completion with automatic acceptance.

## Prerequisites

- LLMC installed and configured
- A clean LLMC workspace (run `llmc nuke --all` if needed)
- No daemon currently running (`llmc down`)

## Differentiating Errors from Normal Operations

**Error indicators to watch for:**
- Any `ERROR` or `WARN` messages in `~/llmc/logs/auto.log`
- Non-zero exit codes from `llmc up --auto`
- Workers in `error` state in `llmc status`
- Missing heartbeat file `~/llmc/.llmc/auto.heartbeat`
- Daemon shutting down unexpectedly

**Normal operations:**
- Workers cycling through `idle` → `working` → `needs_review` → `idle`
- Empty task pool causing workers to wait (not an error)
- Self-review prompts being sent

## Setup

```bash
# Ensure clean state
cd ~/llmc
llmc down --force 2>/dev/null || true
llmc nuke --all --yes 2>/dev/null || true

# Create a simple task pool script that returns one task then empty
mkdir -p ~/llmc/test_scripts
cat > ~/llmc/test_scripts/task_pool.sh << 'EOF'
#!/bin/bash
MARKER=~/llmc/test_scripts/.task_issued
if [ ! -f "$MARKER" ]; then
    touch "$MARKER"
    echo "Create a file called test_auto_task.txt in the root directory containing the text 'Hello from auto mode'. Do not create any other files."
else
    # No more tasks
    exit 0
fi
EOF
chmod +x ~/llmc/test_scripts/task_pool.sh

# Add auto configuration to config.toml
cat >> ~/llmc/config.toml << 'EOF'

[auto]
task_pool_command = "~/llmc/test_scripts/task_pool.sh"
concurrency = 1
EOF
```

## Test Sequence

### Part 1: Auto Mode Startup

**Step 1.1**: Start auto mode daemon.

```bash
llmc up --auto &
DAEMON_PID=$!
sleep 5
```

**Verify**:
- Daemon starts without errors
- `~/llmc/.llmc/daemon.json` exists and contains `pid`, `start_time_unix`, `instance_id`
- `~/llmc/.llmc/auto.heartbeat` exists and is being updated

**Step 1.2**: Check auto worker creation.

```bash
llmc status
```

**Verify**:
- Auto worker `auto-1` exists
- Status shows "Auto Workers" section
- Worker is in `idle` or `working` state

### Part 2: Task Assignment

**Step 2.1**: Wait for task assignment and completion.

```bash
# Wait for worker to complete (check status periodically)
for i in {1..60}; do
    STATUS=$(llmc status --json 2>/dev/null | jq -r '.workers[] | select(.name == "auto-1") | .status' 2>/dev/null)
    if [ "$STATUS" = "idle" ] && [ -f ~/llmc/test_scripts/.task_issued ]; then
        echo "Task completed, worker back to idle"
        break
    fi
    sleep 5
done
```

**Verify**:
- Task was assigned to auto-1
- Worker completed the task
- Changes were automatically accepted (no manual `llmc review` needed)
- Worker returned to `idle` state

**Step 2.2**: Verify task was executed.

```bash
# Check if the file was created in master
cd ~/Documents/GoogleDrive/dreamtides
cat test_auto_task.txt
```

**Verify**:
- File `test_auto_task.txt` exists in master repo
- Contains "Hello from auto mode"

### Part 3: Logging Verification

**Step 3.1**: Check auto-specific logs.

```bash
cat ~/llmc/logs/auto.log | tail -50
```

**Verify**:
- Log contains task assignment event
- Log contains task completion/accept event
- No ERROR or WARN entries

**Step 3.2**: Check task pool logs.

```bash
cat ~/llmc/logs/task_pool.log | tail -20
```

**Verify**:
- Shows task pool command invocations
- Shows the task description that was returned

### Part 4: Graceful Shutdown

**Step 4.1**: Stop the daemon.

```bash
kill -SIGINT $DAEMON_PID
wait $DAEMON_PID 2>/dev/null
```

**Verify**:
- Daemon shuts down gracefully
- Exit code is 0
- Workers are stopped

## Cleanup

```bash
# Remove test artifacts
rm -rf ~/llmc/test_scripts
rm -f ~/Documents/GoogleDrive/dreamtides/test_auto_task.txt

# Remove auto config (edit config.toml to remove [auto] section)
# Or restore from backup

# Clean up git
cd ~/Documents/GoogleDrive/dreamtides
git checkout -- . 2>/dev/null || true
git clean -fd 2>/dev/null || true

llmc down --force 2>/dev/null || true
llmc nuke --all --yes 2>/dev/null || true
```

## Expected Issues to Report

1. Auto worker not created with correct name (`auto-1`)
2. Task pool command not executed
3. Task not assigned to worker
4. Changes not automatically accepted (stuck in needs_review)
5. Logs not created in expected locations
6. Heartbeat file not updated
7. Daemon does not shut down gracefully
8. Any ERROR or WARN in logs
9. Worker enters error state unexpectedly

## Abort Conditions

**Abort the test and file a task if:**
- Daemon crashes on startup
- Auto worker creation fails
- Task assignment hangs for more than 10 minutes
- Any unhandled panic occurs
- State file becomes corrupted
