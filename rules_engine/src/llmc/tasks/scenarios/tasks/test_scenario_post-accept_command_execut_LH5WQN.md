---
lattice-id: LH5WQN
name: test-scenario-post-accept-command-execut
description: 'Test Scenario: Post-Accept Command Execution'
parent-id: LB5WQN
task-type: task
priority: 2
labels:
- testing
- manual-test
- scenario
- auto-mode
- post-accept
blocked-by:
- LH4WQN
created-at: 2026-01-21T22:01:20.201938Z
updated-at: 2026-01-21T22:31:38.839311Z
---

# Test Scenario: Post-Accept Command Execution

## Objective

Verify that the `post_accept_command` configuration is correctly executed after
a worker's changes are merged to master, that the daemon blocks until
completion,
and that failures trigger graceful shutdown.

## Prerequisites

- LLMC installed and configured
- A clean LLMC workspace
- No daemon currently running
- Previous test scenario (LCBWQN) completed successfully

## Differentiating Errors from Normal Operations

**Error indicators:**

- Post-accept command not executed
- Daemon not blocking during post-accept execution
- Non-zero exit from post-accept not triggering shutdown
- Post-accept logs missing from `~/llmc/logs/post_accept.log`
- Next task assigned before post-accept completes

**Normal operations:**

- Delay between accept and next task assignment (post-accept running)
- Post-accept command output appearing in logs
- Daemon blocking while post-accept runs

## Setup

```bash
# Ensure clean state
cd ~/llmc
llmc down --force 2>/dev/null || true
llmc nuke --all --yes 2>/dev/null || true

# Create task pool and post-accept scripts
mkdir -p ~/llmc/test_scripts
cat > ~/llmc/test_scripts/post_accept_pool.sh << 'EOF'
#!/bin/bash
COUNTER_FILE=~/llmc/test_scripts/.task_counter
if [ ! -f "$COUNTER_FILE" ]; then
    echo "0" > "$COUNTER_FILE"
fi

COUNTER=$(cat "$COUNTER_FILE")
if [ "$COUNTER" -lt 2 ]; then
    COUNTER=$((COUNTER + 1))
    echo "$COUNTER" > "$COUNTER_FILE"
    echo "Create a file called post_accept_test_${COUNTER}.txt containing 'Task ${COUNTER}'. Do not create other files."
else
    exit 0
fi
EOF
chmod +x ~/llmc/test_scripts/post_accept_pool.sh

# Create post-accept command that takes noticeable time
cat > ~/llmc/test_scripts/post_accept.sh << 'EOF'
#!/bin/bash
echo "Post-accept starting at $(date)"
echo "Running validation..."

# Simulate validation work
sleep 5

# Check that the file exists in master
cd ~/Documents/GoogleDrive/dreamtides
if ls post_accept_test_*.txt 1>/dev/null 2>&1; then
    echo "Validation passed: test files exist"
    echo "Post-accept completed at $(date)"
    exit 0
else
    echo "Validation failed: no test files found"
    exit 1
fi
EOF
chmod +x ~/llmc/test_scripts/post_accept.sh

# Configure auto mode with post_accept_command
cat >> ~/llmc/config.toml << 'EOF'

[auto]
task_pool_command = "~/llmc/test_scripts/post_accept_pool.sh"
concurrency = 1
post_accept_command = "~/llmc/test_scripts/post_accept.sh"
EOF
```

## Test Sequence

### Part 1: Post-Accept Command Invocation

**Step 1.1**: Start auto mode.

```bash
llmc up --auto &
DAEMON_PID=$!
sleep 10
```

**Step 1.2**: Wait for first task completion and post-accept.

```bash
# Wait for first task to complete
for i in {1..60}; do
    if [ -f ~/Documents/GoogleDrive/dreamtides/post_accept_test_1.txt ]; then
        echo "First task file created"
        break
    fi
    sleep 5
done

# Give post-accept time to run
sleep 10
```

**Verify**:

- File `post_accept_test_1.txt` exists in master
- Post-accept command was executed

**Step 1.3**: Check post-accept logs.

```bash
cat ~/llmc/logs/post_accept.log
```

**Verify**:

- Log shows "Post-accept starting" message
- Log shows "Running validation" message
- Log shows "Validation passed" message
- Log shows "Post-accept completed" message

### Part 2: Blocking Behavior

**Step 2.1**: Verify daemon blocked during post-accept.

```bash
# Check auto.log for blocking indication
cat ~/llmc/logs/auto.log | grep -i "post.accept\|block\|wait" | tail -10
```

**Verify**:

- Log shows daemon waiting for post-accept completion
- No new task assigned during post-accept execution

**Step 2.2**: Verify sequential execution.

```bash
# Wait for second task
for i in {1..60}; do
    if [ -f ~/Documents/GoogleDrive/dreamtides/post_accept_test_2.txt ]; then
        echo "Second task file created"
        break
    fi
    sleep 5
done

# Check timing in logs
cat ~/llmc/logs/post_accept.log | grep "completed\|starting"
```

**Verify**:

- Second post-accept only starts after first completes
- Time gap matches expected blocking behavior

### Part 3: Post-Accept Failure Handling

**Step 3.1**: Stop daemon and modify post-accept to fail.

```bash
kill -SIGINT $DAEMON_PID 2>/dev/null
wait $DAEMON_PID 2>/dev/null

# Modify post-accept to fail
cat > ~/llmc/test_scripts/post_accept.sh << 'EOF'
#!/bin/bash
echo "Post-accept starting - will fail"
exit 1
EOF

# Reset task counter for new task
rm -f ~/llmc/test_scripts/.task_counter
```

**Step 3.2**: Restart daemon and observe failure shutdown.

```bash
llmc up --auto &
DAEMON_PID=$!

# Wait and observe
for i in {1..30}; do
    if ! kill -0 $DAEMON_PID 2>/dev/null; then
        echo "Daemon terminated (expected due to post-accept failure)"
        break
    fi
    sleep 5
done
```

**Verify**:

- Daemon shuts down after post-accept failure
- Exit code is non-zero

**Step 3.3**: Check error logging.

```bash
cat ~/llmc/logs/auto.log | tail -20
cat ~/llmc/logs/post_accept.log | tail -10
```

**Verify**:

- Log shows post-accept failure
- Log shows graceful shutdown initiated
- Error details captured

### Part 4: Long-Running Post-Accept

**Step 4.1**: Configure long-running post-accept.

```bash
# Reset
llmc down --force 2>/dev/null || true
rm -f ~/llmc/test_scripts/.task_counter

cat > ~/llmc/test_scripts/post_accept.sh << 'EOF'
#!/bin/bash
echo "Starting long validation..."
sleep 30
echo "Long validation complete"
exit 0
EOF
```

**Step 4.2**: Verify daemon waits full duration.

```bash
llmc up --auto &
DAEMON_PID=$!

START_TIME=$(date +%s)

# Wait for task to complete and post-accept to finish
for i in {1..60}; do
    if [ -f ~/Documents/GoogleDrive/dreamtides/post_accept_test_1.txt ]; then
        # Check if worker is back to idle (post-accept done)
        STATUS=$(llmc status --json 2>/dev/null | jq -r '.workers[] | select(.name == "auto-1") | .status' 2>/dev/null)
        if [ "$STATUS" = "idle" ]; then
            END_TIME=$(date +%s)
            DURATION=$((END_TIME - START_TIME))
            echo "Post-accept cycle completed in $DURATION seconds"
            break
        fi
    fi
    sleep 5
done
```

**Verify**:

- Duration includes post-accept wait time (should be at least 30 seconds)
- No next task assigned during wait

## Cleanup

```bash
# Stop daemon
kill -SIGINT $DAEMON_PID 2>/dev/null
wait $DAEMON_PID 2>/dev/null

# Remove test artifacts
rm -rf ~/llmc/test_scripts
rm -f ~/Documents/GoogleDrive/dreamtides/post_accept_test_*.txt

# Clean up git
cd ~/Documents/GoogleDrive/dreamtides
git checkout -- . 2>/dev/null || true
git clean -fd 2>/dev/null || true

llmc down --force 2>/dev/null || true
llmc nuke --all --yes 2>/dev/null || true
```

## Expected Issues to Report

1. Post-accept command not executed
2. Post-accept output not logged to post_accept.log
3. Daemon doesn't block during post-accept
4. Next task assigned before post-accept completes
5. Post-accept failure doesn't trigger shutdown
6. Post-accept exit code not checked
7. Long-running post-accept times out unexpectedly
8. Post-accept runs in wrong directory

## Abort Conditions

**Abort the test and file a task if:**

- Post-accept script never invoked
- Daemon crashes instead of graceful shutdown on failure
- Post-accept has access to wrong repository
- Logs show post-accept running multiple times per accept
