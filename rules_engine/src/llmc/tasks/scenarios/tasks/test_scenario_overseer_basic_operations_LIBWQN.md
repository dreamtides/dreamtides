---
lattice-id: LIBWQN
name: test-scenario-overseer-basic-operations
description: 'Test Scenario: Overseer Basic Operations'
parent-id: LB5WQN
task-type: task
priority: 2
labels:
- testing
- manual-test
- scenario
- overseer
- llmc-auto
blocked-by:
- LIAWQN
created-at: 2026-01-21T22:04:42.982376Z
updated-at: 2026-01-21T22:31:38.823585Z
---

# Test Scenario: Overseer Basic Operations

## Objective

Verify that the overseer correctly starts the daemon, monitors its health via
heartbeat and logs, maintains the overseer Claude Code session, and gracefully
handles Ctrl-C shutdown.

## Prerequisites

- LLMC installed and configured
- No daemon or overseer currently running
- Previous test scenario (LCFWQN) completed successfully
- Overseer configuration in config.toml

## Environment Setup

**This test MUST run in an isolated LLMC instance.** See
`../isolated_test_environment.md` for complete setup instructions.

```bash
# Create isolated test environment
export TEST_DIR="/tmp/llmc-overseer-basic-test-$$"
export LLMC_ROOT="$TEST_DIR"

llmc init --source ~/Documents/GoogleDrive/dreamtides --target "$TEST_DIR"

# Verify isolation
llmc status
```

## Differentiating Errors from Normal Operations

**Error indicators:**

- Overseer fails to start daemon
- Daemon registration file not created
- Heartbeat not detected
- Overseer Claude Code session not created
- Overseer terminates unexpectedly
- `llmc status` not showing "Overseer" section

**Normal operations:**

- Overseer starts daemon via shell command
- Daemon creates registration file
- Heartbeat updated every 5 seconds
- Overseer Claude Code session created (`llmc-overseer`)
- Log monitoring active

## Setup

```bash
# Ensure clean state
cd $LLMC_ROOT
llmc down --force 2>/dev/null || true
llmc nuke --all --yes 2>/dev/null || true

# Kill any existing overseer session
tmux kill-session -t llmc-overseer 2>/dev/null || true

# Create test scripts
mkdir -p $LLMC_ROOT/test_scripts
cat > $LLMC_ROOT/test_scripts/overseer_pool.sh << 'EOF'
#!/bin/bash
COUNTER_FILE=$LLMC_ROOT/test_scripts/.task_counter
if [ ! -f "$COUNTER_FILE" ]; then
    echo "0" > "$COUNTER_FILE"
fi

COUNTER=$(cat "$COUNTER_FILE")
if [ "$COUNTER" -lt 3 ]; then
    COUNTER=$((COUNTER + 1))
    echo "$COUNTER" > "$COUNTER_FILE"
    echo "Create a file called overseer_test_${COUNTER}.txt containing 'Task ${COUNTER}'. Do not create other files."
else
    exit 0
fi
EOF
chmod +x $LLMC_ROOT/test_scripts/overseer_pool.sh

# Configure auto mode and overseer
cat >> $LLMC_ROOT/config.toml << 'EOF'

[auto]
task_pool_command = "$LLMC_ROOT/test_scripts/overseer_pool.sh"
concurrency = 1

[overseer]
remediation_prompt = """
You are debugging LLMC. Check the logs in $LLMC_ROOT/logs/ to understand what went wrong.
Common issues:
- Git conflicts: check git status in worktrees
- Session crashes: restart the affected session
- Config errors: validate config.toml syntax

After fixing the issue, exit normally. The overseer will restart the daemon.
"""
heartbeat_timeout_secs = 30
stall_timeout_secs = 1800
restart_cooldown_secs = 60
EOF
```

## Test Sequence

### Part 1: Overseer Startup

**Step 1.1**: Start overseer.

```bash
cd $LLMC_ROOT
llmc overseer &
OVERSEER_PID=$!

sleep 10
```

**Verify**:

- Overseer process starts without error
- No immediate crash or termination

**Step 1.2**: Verify daemon started by overseer.

```bash
# Check daemon registration
cat $LLMC_ROOT/.llmc/daemon.json | jq '.'
```

**Verify**:

- `daemon.json` exists
- Contains `pid`, `start_time_unix`, `instance_id`, `log_file`
- PID is valid (process exists)

**Step 1.3**: Verify heartbeat mechanism.

```bash
# Check initial heartbeat
cat $LLMC_ROOT/.llmc/auto.heartbeat | jq '.'
FIRST_TS=$(cat $LLMC_ROOT/.llmc/auto.heartbeat | jq '.timestamp_unix')

sleep 10

# Check heartbeat updated
cat $LLMC_ROOT/.llmc/auto.heartbeat | jq '.'
SECOND_TS=$(cat $LLMC_ROOT/.llmc/auto.heartbeat | jq '.timestamp_unix')

if [ "$SECOND_TS" -gt "$FIRST_TS" ]; then
    echo "Heartbeat is being updated"
else
    echo "WARNING: Heartbeat not updating"
fi
```

**Verify**:

- `auto.heartbeat` exists
- Timestamp updates every ~5 seconds
- `instance_id` matches daemon registration

### Part 2: Overseer Session Management

**Step 2.1**: Verify overseer Claude Code session.

```bash
tmux list-sessions | grep llmc-overseer
```

**Verify**:

- Session `llmc-overseer` exists
- Session is in master repository directory

**Step 2.2**: Peek at overseer session.

```bash
tmux capture-pane -t llmc-overseer -p | tail -20
```

**Verify**:

- Claude Code is running in the session
- No errors visible

**Step 2.3**: Verify overseer session protection.

```bash
# Run llmc down and verify overseer session survives
llmc down

tmux list-sessions | grep llmc-overseer
echo "Overseer session should still exist after llmc down"
```

**Verify**:

- `llmc down` does NOT kill overseer session
- Overseer session persists

### Part 3: Status Display Integration

**Step 3.1**: Check llmc status shows overseer info.

```bash
# Restart daemon for status check
llmc up --auto &
sleep 10

llmc status
```

**Verify**:

- Status shows "Overseer" section (if overseer running)
- Shows daemon PID and uptime
- Shows overseer state

**Step 3.2**: Check JSON status output.

```bash
llmc status --json | jq '.overseer'
```

**Verify**:

- Overseer information in JSON output
- Includes state, daemon_pid, uptime

### Part 4: Log Monitoring

**Step 4.1**: Verify overseer monitors daemon logs.

```bash
# Check overseer is tailing daemon logs
# (This is internal behavior, verify via overseer's behavior)
cat $LLMC_ROOT/logs/auto.log | tail -5
```

**Verify**:

- Daemon logs are being written
- No errors that would trigger overseer intervention

**Step 4.2**: Verify task progress tracking.

```bash
# Wait for some tasks to complete
for i in {1..60}; do
    TASK_COUNT=$(ls ~/Documents/GoogleDrive/dreamtides/overseer_test_*.txt 2>/dev/null | wc -l)
    echo "Completed tasks: $TASK_COUNT"

    if [ "$TASK_COUNT" -ge 2 ]; then
        echo "Tasks completing successfully"
        break
    fi
    sleep 5
done
```

**Verify**:

- Tasks are completing
- Overseer observes progress (no stall detection)

### Part 5: Graceful Shutdown

**Step 5.1**: Send Ctrl-C to overseer.

```bash
# Note: If running in background, use kill -SIGINT
kill -SIGINT $OVERSEER_PID

# Wait for shutdown
for i in {1..30}; do
    if ! kill -0 $OVERSEER_PID 2>/dev/null; then
        echo "Overseer terminated"
        break
    fi
    sleep 2
done

wait $OVERSEER_PID 2>/dev/null
EXIT_CODE=$?
echo "Overseer exit code: $EXIT_CODE"
```

**Verify**:

- Overseer shuts down gracefully
- Exit code is 0
- Daemon is also terminated

**Step 5.2**: Verify cleanup.

```bash
# Check daemon is stopped
ps aux | grep "llmc up --auto" | grep -v grep || echo "Daemon stopped"

# Check overseer session still exists (should survive overseer termination)
tmux list-sessions | grep llmc-overseer || echo "Overseer session terminated (expected on overseer shutdown)"
```

**Verify**:

- Daemon process is stopped
- State is clean

### Part 6: Process Identity Verification

**Step 6.1**: Verify overseer detects daemon identity changes.

```bash
# Start fresh
llmc overseer &
OVERSEER_PID=$!
sleep 10

# Record original daemon identity
ORIGINAL_PID=$(cat $LLMC_ROOT/.llmc/daemon.json | jq '.pid')
ORIGINAL_INSTANCE=$(cat $LLMC_ROOT/.llmc/daemon.json | jq -r '.instance_id')
echo "Original: PID=$ORIGINAL_PID, Instance=$ORIGINAL_INSTANCE"

# This would require external manipulation to test fully
# The key verification is that daemon.json contains unique instance_id
```

**Verify**:

- `instance_id` is unique (UUID format)
- Each daemon start generates new `instance_id`

## Cleanup

```bash
# Stop overseer
kill -SIGINT $OVERSEER_PID 2>/dev/null
wait $OVERSEER_PID 2>/dev/null

# Kill overseer session
tmux kill-session -t llmc-overseer 2>/dev/null || true

# Remove test artifacts
rm -rf $LLMC_ROOT/test_scripts
rm -f ~/Documents/GoogleDrive/dreamtides/overseer_test_*.txt

# Clean up git
cd ~/Documents/GoogleDrive/dreamtides
git checkout -- . 2>/dev/null || true
git clean -fd 2>/dev/null || true

llmc down --force 2>/dev/null || true
llmc nuke --all --yes 2>/dev/null || true
```

## Expected Issues to Report

1. Overseer fails to start daemon
2. daemon.json not created or incomplete
3. Heartbeat not being updated
4. Overseer session not created
5. `llmc down` kills overseer session
6. Status doesn't show overseer information
7. Overseer doesn't detect daemon termination
8. Graceful shutdown fails
9. Instance ID not unique across restarts

## Abort Conditions

**Abort the test and file a task if:**

- Overseer panics on startup
- Daemon and overseer interfere with each other
- State file corruption
- TMUX sessions left in broken state
- Overseer enters infinite loop
