---
lattice-id: LIDWQN
name: test-scenario-overseer-failure-spiral-pr
description: 'Test Scenario: Overseer Failure Spiral Prevention'
parent-id: LB5WQN
task-type: task
priority: 2
labels:
- testing
- manual-test
- scenario
- overseer
- failure-spiral
- llmc-auto
blocked-by:
- LICWQN
- LEGWQN
created-at: 2026-01-21T22:06:36.953507Z
updated-at: 2026-01-23T19:15:40.861385Z
---

# Test Scenario: Overseer Failure Spiral Prevention

## Objective

Verify that the overseer correctly detects failure spirals (daemon failing
within
restart_cooldown_secs of last start), prevents infinite remediation loops, and
terminates with a clear error message requiring human intervention.

## Prerequisites

- LLMC installed and configured
- No daemon or overseer currently running
- Previous test scenario (LCHWQN) completed successfully
- Overseer configuration with restart_cooldown_secs

## Environment Setup

**This test MUST run in an isolated LLMC instance.** See
`../isolated_test_environment.md` for complete setup instructions.

```bash
# Create isolated test environment
export TEST_DIR="/tmp/llmc-spiral-test-$$"
export LLMC_ROOT="$TEST_DIR"

llmc init --source ~/Documents/GoogleDrive/dreamtides --target "$TEST_DIR"

# Verify isolation
llmc status
```

## Differentiating Errors from Normal Operations

**Error indicators (that are bugs, not expected failures):**

- Overseer enters infinite remediation loop
- Failure spiral not detected
- Manual intervention file not checked
- Overseer continues after manual_intervention_needed created
- Cooldown timer not tracked correctly

**Expected behavior in this test:**

- Daemon fails repeatedly within cooldown period
- Overseer detects failure spiral
- Overseer terminates (does not attempt more remediation)
- Clear error message about human intervention required

## Setup

```bash
# Ensure clean state
cd $LLMC_ROOT
llmc down --force 2>/dev/null || true
llmc nuke --all --yes 2>/dev/null || true
tmux kill-session -t llmc-overseer 2>/dev/null || true
rm -f $LLMC_ROOT/.llmc/manual_intervention_needed_*.txt

# Create test scripts
mkdir -p $LLMC_ROOT/test_scripts

# Create task pool that always fails (simulates unfixable issue)
cat > $LLMC_ROOT/test_scripts/spiral_pool.sh << 'EOF'
#!/bin/bash
# Always fail - simulates persistent infrastructure issue
echo "ERROR: Network unreachable" >&2
exit 1
EOF
chmod +x $LLMC_ROOT/test_scripts/spiral_pool.sh

# Configure with short cooldown for testing
cat >> $LLMC_ROOT/config.toml << 'EOF'

[auto]
task_pool_command = "$LLMC_ROOT/test_scripts/spiral_pool.sh"
concurrency = 1

[overseer]
remediation_prompt = """
You are debugging LLMC. A failure occurred.
Check logs and try to fix the issue.
If unfixable, create ~/.llmc/manual_intervention_needed_$(date +%s).txt
"""
heartbeat_timeout_secs = 15
stall_timeout_secs = 300
restart_cooldown_secs = 30
EOF
```

## Test Sequence

### Part 1: Setup Failure Spiral Condition

**Step 1.1**: Start overseer with failing daemon.

```bash
cd $LLMC_ROOT
llmc overseer &
OVERSEER_PID=$!

echo "Started overseer PID: $OVERSEER_PID"
sleep 5
```

**Step 1.2**: Observe first failure and remediation attempt.

```bash
# Wait for daemon to fail
for i in {1..30}; do
    echo "=== Check $i ==="

    # Check daemon status
    if [ -f $LLMC_ROOT/.llmc/daemon.json ]; then
        DAEMON_PID=$(cat $LLMC_ROOT/.llmc/daemon.json | jq '.pid')
        if ! kill -0 $DAEMON_PID 2>/dev/null; then
            echo "Daemon has died"
        fi
    fi

    # Check for remediation activity
    ls $LLMC_ROOT/logs/remediation_*.txt 2>/dev/null | head -3

    sleep 3
done
```

**Verify**:

- Daemon fails due to task pool error
- Overseer detects failure
- First remediation attempt initiated

### Part 2: Observe Failure Spiral Detection

**Step 2.1**: Wait for second failure within cooldown.

```bash
# After remediation, daemon will restart and fail again
# This should happen within restart_cooldown_secs (30s)

echo "Waiting for failure spiral detection..."

for i in {1..60}; do
    # Check if overseer is still running
    if ! kill -0 $OVERSEER_PID 2>/dev/null; then
        echo "Overseer terminated (expected for failure spiral)"
        break
    fi

    # Check for failure spiral log messages
    grep -i "spiral\|cooldown\|repeated\|manual" $LLMC_ROOT/logs/auto.log 2>/dev/null | tail -5

    sleep 3
done
```

**Verify**:

- Daemon fails again within 30 seconds of restart
- Overseer detects failure spiral
- Overseer does NOT attempt another remediation

**Step 2.2**: Verify overseer termination.

```bash
# Get exit code
wait $OVERSEER_PID 2>/dev/null
EXIT_CODE=$?
echo "Overseer exit code: $EXIT_CODE"
```

**Verify**:

- Overseer terminates (non-zero exit code expected)
- Termination is graceful (not a crash)

### Part 3: Error Message Quality

**Step 3.1**: Check final log messages.

```bash
cat $LLMC_ROOT/logs/auto.log | tail -30
```

**Verify**:

- Clear message about failure spiral detected
- Message indicates human intervention required
- Message includes relevant context (failure count, timing)

**Step 3.2**: Check remediation logs.

```bash
ls -la $LLMC_ROOT/logs/remediation_*.txt

# Should have one (or possibly two) remediation logs
# NOT many (which would indicate infinite loop)
REMEDIATION_COUNT=$(ls $LLMC_ROOT/logs/remediation_*.txt 2>/dev/null | wc -l)
echo "Remediation attempts: $REMEDIATION_COUNT"
```

**Verify**:

- Small number of remediation logs (1-2)
- Not dozens (which would indicate failed prevention)

### Part 4: Manual Intervention File Handling

**Step 4.1**: Create manual intervention file scenario.

```bash
# Reset for new test
rm -f $LLMC_ROOT/logs/remediation_*.txt
rm -f $LLMC_ROOT/.llmc/manual_intervention_needed_*.txt

# Create task pool that works once then the remediation creates manual intervention file
cat > $LLMC_ROOT/test_scripts/spiral_pool.sh << 'EOF'
#!/bin/bash
MARKER=$LLMC_ROOT/test_scripts/.spiral_task_done
if [ ! -f "$MARKER" ]; then
    touch "$MARKER"
    echo "Create file spiral_test.txt with content 'test'. Do not create other files."
    exit 0
else
    # Subsequent calls fail
    exit 1
fi
EOF

# Create remediation prompt that will create manual intervention file
cat > $LLMC_ROOT/config.toml.new << 'EOF'
# ... existing config ...

[overseer]
remediation_prompt = """
Create a manual intervention needed file since this issue requires human attention:

touch ~/.llmc/manual_intervention_needed_$(date +%s).txt
echo "Network issue detected - infrastructure team needs to investigate" > ~/.llmc/manual_intervention_needed_*.txt

Then exit normally.
"""
restart_cooldown_secs = 30
EOF

# Note: Would need to properly update config - this is illustrative
```

**Step 4.2**: Verify overseer checks for manual intervention file.

```bash
# Create manual intervention file directly to test detection
mkdir -p $LLMC_ROOT/.llmc
echo "Test: Network infrastructure is down" > $LLMC_ROOT/.llmc/manual_intervention_needed_test.txt

llmc overseer &
OVERSEER_PID=$!

sleep 10

# Overseer should detect the file and terminate
if ! kill -0 $OVERSEER_PID 2>/dev/null; then
    echo "Overseer correctly terminated due to manual intervention file"
fi

wait $OVERSEER_PID 2>/dev/null
EXIT_CODE=$?
echo "Exit code: $EXIT_CODE"
```

**Verify**:

- Overseer detects manual_intervention_needed_*.txt
- Overseer logs the file contents
- Overseer terminates with clear message

**Step 4.3**: Check manual intervention message logged.

```bash
cat $LLMC_ROOT/logs/auto.log | grep -i "manual\|intervention" | tail -10
```

**Verify**:

- Log shows manual intervention file was found
- Log includes file contents
- Clear message about human action needed

### Part 5: Cooldown Tracking Accuracy

**Step 5.1**: Verify cooldown is tracked from start time.

```bash
# Clean up
rm -f $LLMC_ROOT/.llmc/manual_intervention_needed_*.txt
rm -f $LLMC_ROOT/test_scripts/.spiral_task_done

# Create always-failing pool
cat > $LLMC_ROOT/test_scripts/spiral_pool.sh << 'EOF'
#!/bin/bash
exit 1
EOF

llmc overseer &
OVERSEER_PID=$!

# Track daemon start times
START_TIMES=""
for i in {1..10}; do
    if [ -f $LLMC_ROOT/.llmc/daemon.json ]; then
        START_TIME=$(cat $LLMC_ROOT/.llmc/daemon.json | jq '.start_time_unix')
        echo "Daemon start time: $START_TIME"
        START_TIMES="$START_TIMES $START_TIME"
    fi

    if ! kill -0 $OVERSEER_PID 2>/dev/null; then
        echo "Overseer terminated"
        break
    fi

    sleep 5
done
```

**Verify**:

- Start times are tracked
- Failure within cooldown of LAST start triggers spiral detection
- Not based on first-ever start

### Part 6: Successful Run Resets Cooldown

**Step 6.1**: Verify healthy runtime resets cooldown state.

```bash
# This is a conceptual test - in practice:
# 1. Start daemon that works
# 2. Let it run past cooldown period
# 3. Then fail
# 4. Remediation should be allowed (not spiral)

echo "Conceptual test: After running successfully past cooldown_secs,"
echo "a new failure should trigger normal remediation, not spiral detection"
```

**Verify** (conceptually):

- Daemon running > restart_cooldown_secs clears failure tracking
- Subsequent failure triggers normal remediation
- Only rapid successive failures trigger spiral

## Cleanup

```bash
# Stop any running overseer
kill -SIGINT $OVERSEER_PID 2>/dev/null
wait $OVERSEER_PID 2>/dev/null

# Kill sessions
tmux kill-session -t llmc-overseer 2>/dev/null || true

# Remove test artifacts
rm -rf $LLMC_ROOT/test_scripts
rm -f ~/Documents/GoogleDrive/dreamtides/spiral_test.txt
rm -f $LLMC_ROOT/.llmc/manual_intervention_needed_*.txt

# Restore config
# (Edit config.toml to remove test [auto] and [overseer] sections)

# Clean up git
cd ~/Documents/GoogleDrive/dreamtides
git checkout -- . 2>/dev/null || true
git clean -fd 2>/dev/null || true

llmc down --force 2>/dev/null || true
llmc nuke --all --yes 2>/dev/null || true
```

## Expected Issues to Report

1. Overseer enters infinite remediation loop
2. Failure spiral not detected within cooldown
3. Manual intervention file not detected
4. Manual intervention file contents not logged
5. Unclear error message about spiral
6. Cooldown tracked from wrong timestamp
7. Successful runtime doesn't reset cooldown
8. Overseer crashes instead of graceful termination

## Abort Conditions

**Abort the test and file a task if:**

- Infinite remediation loop occurs
- System resources exhausted
- State corruption from rapid daemon restarts
- Overseer panics during spiral handling
- Manual intervention check causes crash

## Notes

This test validates a critical safety mechanism. Without failure spiral
prevention, the overseer could:

- Exhaust API quotas with repeated Claude calls
- Fill disk with remediation logs
- Create system instability with rapid process cycling
- Never alert humans to unfixable infrastructure issues
