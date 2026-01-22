---
lattice-id: LICWQN
name: test-scenario-overseer-failure-detection
description: 'Test Scenario: Overseer Failure Detection and Remediation'
parent-id: LB5WQN
task-type: task
priority: 2
labels:
- testing
- manual-test
- scenario
- overseer
- remediation
- llmc-auto
blocked-by:
- LIBWQN
created-at: 2026-01-21T22:05:41.880949Z
updated-at: 2026-01-21T22:31:38.829615Z
---

# Test Scenario: Overseer Failure Detection and Remediation

## Objective

Verify that the overseer correctly detects daemon failures (heartbeat timeout,
log errors, process death, stall), terminates the daemon, constructs appropriate
remediation prompts, and executes remediation via Claude Code.

## Prerequisites

- LLMC installed and configured
- A clean LLMC workspace
- No daemon or overseer currently running
- Previous test scenario (LCGWQN) completed successfully
- Overseer configuration with remediation_prompt

## Differentiating Errors from Normal Operations

**Error indicators:**

- Overseer doesn't detect daemon failure
- Remediation prompt not sent to overseer session
- Remediation logs not created
- Daemon not terminated before remediation
- Overseer crashes during remediation
- Remediation session hangs indefinitely

**Normal operations:**

- Brief delay between failure detection and termination
- Overseer session receiving remediation prompt
- Remediation attempting to fix issues
- Daemon restart after successful remediation

## Setup

```bash
# Ensure clean state
cd ~/llmc
llmc down --force 2>/dev/null || true
llmc nuke --all --yes 2>/dev/null || true
tmux kill-session -t llmc-overseer 2>/dev/null || true

# Create test scripts
mkdir -p ~/llmc/test_scripts

# Task pool that works initially
cat > ~/llmc/test_scripts/remediation_pool.sh << 'EOF'
#!/bin/bash
MARKER=~/llmc/test_scripts/.task_issued
if [ ! -f "$MARKER" ]; then
    touch "$MARKER"
    echo "Create a file called remediation_test.txt containing 'test'. Do not create other files."
else
    exit 0
fi
EOF
chmod +x ~/llmc/test_scripts/remediation_pool.sh

# Configure auto mode and overseer
cat >> ~/llmc/config.toml << 'EOF'

[auto]
task_pool_command = "~/llmc/test_scripts/remediation_pool.sh"
concurrency = 1

[overseer]
remediation_prompt = """
You are debugging LLMC auto mode. A failure was detected.

Check the following:
1. ~/llmc/logs/auto.log for daemon errors
2. ~/llmc/logs/task_pool.log for task pool issues
3. Git status in worktrees for conflicts

Common fixes:
- If task pool script has errors, fix the script
- If git conflicts, resolve them
- If TMUX sessions are missing, they'll be recreated

After diagnosing and fixing the issue, exit normally.
If the issue cannot be fixed automatically, create a file:
~/.llmc/manual_intervention_needed_$(date +%s).txt
with an explanation of the problem.
"""
heartbeat_timeout_secs = 15
stall_timeout_secs = 300
restart_cooldown_secs = 60
EOF
```

## Test Sequence

### Part 1: Heartbeat Timeout Detection

**Step 1.1**: Start overseer normally.

```bash
cd ~/llmc
llmc overseer &
OVERSEER_PID=$!

# Wait for daemon to start
sleep 15
```

**Step 1.2**: Simulate heartbeat freeze by killing daemon's heartbeat thread.

```bash
# Get daemon PID
DAEMON_PID=$(cat ~/llmc/.llmc/daemon.json | jq '.pid')
echo "Daemon PID: $DAEMON_PID"

# Record current heartbeat
cat ~/llmc/.llmc/auto.heartbeat | jq '.timestamp_unix'

# One way to simulate heartbeat failure: kill the daemon abruptly
kill -9 $DAEMON_PID

echo "Daemon killed - heartbeat will become stale"
```

**Step 1.3**: Wait for overseer to detect failure.

```bash
# Monitor overseer behavior
for i in {1..30}; do
    echo "=== Check $i ==="

    # Check if overseer detected the failure
    cat ~/llmc/logs/auto.log 2>/dev/null | tail -5

    # Check for remediation session activity
    tmux capture-pane -t llmc-overseer -p 2>/dev/null | tail -10

    sleep 5
done
```

**Verify**:

- Overseer detects stale heartbeat within ~15 seconds
- Overseer logs failure detection
- Remediation prompt is sent to overseer Claude session

### Part 2: Log Error Detection

**Step 2.1**: Restart and setup for log error test.

```bash
# Wait for any remediation to complete
sleep 30

# Reset task marker
rm -f ~/llmc/test_scripts/.task_issued

# Modify task pool to trigger error
cat > ~/llmc/test_scripts/remediation_pool.sh << 'EOF'
#!/bin/bash
# First call succeeds, second call errors
COUNTER_FILE=~/llmc/test_scripts/.error_counter
if [ ! -f "$COUNTER_FILE" ]; then
    echo "1" > "$COUNTER_FILE"
    echo "Create a file called log_error_test.txt. Do not create other files."
    exit 0
else
    COUNT=$(cat "$COUNTER_FILE")
    if [ "$COUNT" -eq 1 ]; then
        echo "2" > "$COUNTER_FILE"
        echo "FATAL: Database connection lost" >&2
        exit 1
    else
        exit 0
    fi
fi
EOF
```

**Step 2.2**: Monitor for log-based failure detection.

```bash
# The error from task pool should appear in logs and trigger remediation

for i in {1..60}; do
    # Check for ERROR in auto.log
    if grep -i "error\|fatal" ~/llmc/logs/auto.log 2>/dev/null | tail -3; then
        echo "Error logged"
    fi

    # Check overseer state
    llmc status 2>/dev/null | grep -i "overseer\|remediat" || true

    sleep 5
done
```

**Verify**:

- ERROR entry in daemon log triggers overseer
- Overseer initiates remediation
- Remediation prompt includes log context

### Part 3: Remediation Prompt Construction

**Step 3.1**: Check remediation prompt includes context.

```bash
# Find recent remediation log
ls -la ~/llmc/logs/remediation_*.txt 2>/dev/null | head -5

# Read most recent remediation log
REMEDIATION_LOG=$(ls -t ~/llmc/logs/remediation_*.txt 2>/dev/null | head -1)
if [ -n "$REMEDIATION_LOG" ]; then
    echo "=== Remediation Log ==="
    head -100 "$REMEDIATION_LOG"
fi
```

**Verify**:

- Remediation log created with timestamp
- Contains user-configured remediation_prompt
- Contains failure type
- Contains relevant log excerpts
- Contains worker states
- Contains recovery instructions

**Step 3.2**: Verify prompt structure.

```bash
if [ -n "$REMEDIATION_LOG" ]; then
    grep -i "failure type\|last.*lines\|worker state\|after fixing" "$REMEDIATION_LOG"
fi
```

**Verify**:

- Structured sections present
- Error context is specific, not generic

### Part 4: Remediation Execution

**Step 4.1**: Monitor remediation Claude Code session.

```bash
# Peek at overseer session during remediation
tmux capture-pane -t llmc-overseer -p | tail -30
```

**Verify**:

- Claude Code received the remediation prompt
- Claude Code is actively working on the issue

**Step 4.2**: Check for /clear before remediation.

```bash
# Check remediation log for /clear indication
if [ -n "$REMEDIATION_LOG" ]; then
    head -20 "$REMEDIATION_LOG" | grep -i "clear"
fi
```

**Verify**:

- Session cleared before new remediation prompt

### Part 5: Daemon Restart After Remediation

**Step 5.1**: Wait for remediation completion and daemon restart.

```bash
for i in {1..60}; do
    # Check for new daemon registration
    if [ -f ~/llmc/.llmc/daemon.json ]; then
        NEW_PID=$(cat ~/llmc/.llmc/daemon.json | jq '.pid')
        if [ "$NEW_PID" != "$DAEMON_PID" ]; then
            echo "New daemon started with PID: $NEW_PID"
            break
        fi
    fi

    sleep 5
done
```

**Verify**:

- New daemon started after remediation
- New instance_id generated
- Daemon running normally

**Step 5.2**: Verify system health after remediation.

```bash
llmc status
cat ~/llmc/.llmc/auto.heartbeat | jq '.'
```

**Verify**:

- System recovered
- Heartbeat active
- Workers operational

### Part 6: Remediation Logging Completeness

**Step 6.1**: Examine full remediation log.

```bash
REMEDIATION_LOG=$(ls -t ~/llmc/logs/remediation_*.txt 2>/dev/null | head -1)
if [ -n "$REMEDIATION_LOG" ]; then
    echo "=== Full Remediation Log ==="
    cat "$REMEDIATION_LOG"
fi
```

**Verify**:

- Full prompt captured
- All tool calls logged
- All tool outputs logged
- Claude responses logged
- Final outcome (success/failure) recorded
- Duration recorded

### Part 7: Stall Detection

**Step 7.1**: Test stall detection (long-running, time permitting).

```bash
# This requires waiting for stall_timeout_secs (300s in config)
# For practical testing, may need to reduce timeout or skip

echo "Stall detection test would require ${stall_timeout_secs}s wait"
echo "Verify: If no task completions for stall_timeout, overseer should intervene"
```

**Verify** (conceptually):

- No task completion for stall_timeout_secs triggers remediation
- Stall is different from empty task pool (idle is not stall)

## Cleanup

```bash
# Stop overseer
kill -SIGINT $OVERSEER_PID 2>/dev/null
wait $OVERSEER_PID 2>/dev/null

# Kill sessions
tmux kill-session -t llmc-overseer 2>/dev/null || true

# Remove test artifacts
rm -rf ~/llmc/test_scripts
rm -f ~/Documents/GoogleDrive/dreamtides/remediation_test.txt
rm -f ~/Documents/GoogleDrive/dreamtides/log_error_test.txt

# Clean up git
cd ~/Documents/GoogleDrive/dreamtides
git checkout -- . 2>/dev/null || true
git clean -fd 2>/dev/null || true

llmc down --force 2>/dev/null || true
llmc nuke --all --yes 2>/dev/null || true
```

## Expected Issues to Report

1. Heartbeat timeout not detected
2. Log errors not triggering remediation
3. Remediation prompt missing context
4. Remediation log not created
5. /clear not sent before remediation
6. Claude Code session not receiving prompt
7. Daemon not restarted after remediation
8. Stall detection not working
9. Remediation hangs without timeout

## Abort Conditions

**Abort the test and file a task if:**

- Overseer crashes during remediation
- Remediation corrupts state
- Multiple concurrent remediations triggered
- System enters unrecoverable state
- Remediation loop (immediate re-failure)
