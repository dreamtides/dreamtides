---
lattice-id: LCDWQN
name: test-scenario-task-pool-command-error-ha
description: 'Test Scenario: Task Pool Command Error Handling'
parent-id: LB5WQN
task-type: task
priority: 2
labels:
- testing
- manual-test
- scenario
- auto-mode
- error-handling
blocked-by:
- LCCWQN
created-at: 2026-01-21T22:01:59.228753Z
updated-at: 2026-01-21T22:01:59.228753Z
---

# Test Scenario: Task Pool Command Error Handling

## Objective

Verify that auto mode correctly handles task pool command failures, including
non-zero exit codes and command hangs, triggering appropriate graceful shutdown.

## Prerequisites

- LLMC installed and configured
- A clean LLMC workspace
- No daemon currently running
- Previous test scenario (LCCWQN) completed successfully

## Differentiating Errors from Normal Operations

**Error indicators (expected in this test):**
- Task pool command returning non-zero exit code → daemon should shutdown
- Task pool command hanging → heartbeat should become stale, overseer should detect
- ERROR entries in auto.log for task pool failures
- Daemon exit code non-zero after shutdown

**Normal operations:**
- Empty stdout with exit code 0 = no tasks available (not an error)
- Task pool called repeatedly while workers idle
- Brief delays between task pool invocations

## Setup

```bash
# Ensure clean state
cd ~/llmc
llmc down --force 2>/dev/null || true
llmc nuke --all --yes 2>/dev/null || true

# Create test scripts directory
mkdir -p ~/llmc/test_scripts
```

## Test Sequence

### Part 1: Non-Zero Exit Code Handling

**Step 1.1**: Create task pool that fails immediately.

```bash
cat > ~/llmc/test_scripts/failing_pool.sh << 'EOF'
#!/bin/bash
echo "Task pool error: database connection failed" >&2
exit 1
EOF
chmod +x ~/llmc/test_scripts/failing_pool.sh

# Configure auto mode
cat >> ~/llmc/config.toml << 'EOF'

[auto]
task_pool_command = "~/llmc/test_scripts/failing_pool.sh"
concurrency = 1
EOF
```

**Step 1.2**: Start auto mode and observe shutdown.

```bash
llmc up --auto &
DAEMON_PID=$!

# Wait for daemon to detect failure and shutdown
for i in {1..30}; do
    if ! kill -0 $DAEMON_PID 2>/dev/null; then
        echo "Daemon terminated as expected"
        break
    fi
    echo "Waiting for daemon to shutdown... ($i)"
    sleep 2
done

wait $DAEMON_PID 2>/dev/null
EXIT_CODE=$?
echo "Daemon exit code: $EXIT_CODE"
```

**Verify**:
- Daemon shuts down (doesn't keep running)
- Exit code is non-zero
- Shutdown is graceful (not a crash)

**Step 1.3**: Check error logging.

```bash
cat ~/llmc/logs/auto.log | tail -20
cat ~/llmc/logs/task_pool.log | tail -10
```

**Verify**:
- auto.log shows task pool command failure
- task_pool.log shows stderr output "database connection failed"
- Logs indicate graceful shutdown initiated

### Part 2: Delayed Failure (Success Then Failure)

**Step 2.1**: Create task pool that succeeds once then fails.

```bash
llmc down --force 2>/dev/null || true

cat > ~/llmc/test_scripts/delayed_fail_pool.sh << 'EOF'
#!/bin/bash
MARKER=~/llmc/test_scripts/.task_issued
if [ ! -f "$MARKER" ]; then
    touch "$MARKER"
    echo "Create a file called delayed_fail_test.txt containing 'test'. Do not create other files."
    exit 0
else
    echo "Task pool crashed unexpectedly" >&2
    exit 1
fi
EOF
chmod +x ~/llmc/test_scripts/delayed_fail_pool.sh

# Update config
sed -i.bak 's|failing_pool.sh|delayed_fail_pool.sh|' ~/llmc/config.toml
```

**Step 2.2**: Start daemon and observe behavior.

```bash
llmc up --auto &
DAEMON_PID=$!

# Wait for task assignment, completion, then failure
for i in {1..90}; do
    if ! kill -0 $DAEMON_PID 2>/dev/null; then
        echo "Daemon terminated"
        break
    fi
    
    STATUS=$(llmc status --json 2>/dev/null | jq -r '.workers[] | select(.name == "auto-1") | .status' 2>/dev/null)
    echo "Worker status: $STATUS (iteration $i)"
    
    sleep 5
done

wait $DAEMON_PID 2>/dev/null
EXIT_CODE=$?
echo "Final exit code: $EXIT_CODE"
```

**Verify**:
- First task is assigned and completes
- Second task pool call fails
- Daemon shuts down after failure
- First task's changes may or may not be accepted (depends on timing)

### Part 3: Empty Output vs Error Distinction

**Step 3.1**: Create task pool with empty output (not an error).

```bash
llmc down --force 2>/dev/null || true
rm -f ~/llmc/test_scripts/.task_issued

cat > ~/llmc/test_scripts/empty_pool.sh << 'EOF'
#!/bin/bash
# Return empty output with success - means no tasks available
echo ""
exit 0
EOF
chmod +x ~/llmc/test_scripts/empty_pool.sh

sed -i.bak 's|delayed_fail_pool.sh|empty_pool.sh|' ~/llmc/config.toml
```

**Step 3.2**: Verify daemon stays running with empty pool.

```bash
llmc up --auto &
DAEMON_PID=$!

sleep 30

# Daemon should still be running
if kill -0 $DAEMON_PID 2>/dev/null; then
    echo "Daemon still running (correct - empty pool is not an error)"
else
    echo "ERROR: Daemon terminated unexpectedly"
fi

llmc status
```

**Verify**:
- Daemon remains running
- Workers stay in idle state
- No error logged
- Task pool command called periodically

**Step 3.3**: Check polling behavior.

```bash
# Count task pool invocations over time
sleep 30
cat ~/llmc/logs/task_pool.log | wc -l
```

**Verify**:
- Task pool called multiple times
- Reasonable interval between calls

### Part 4: Partial Output Before Failure

**Step 4.1**: Create task pool that outputs partial data then crashes.

```bash
kill -SIGINT $DAEMON_PID 2>/dev/null
wait $DAEMON_PID 2>/dev/null
llmc down --force 2>/dev/null || true

cat > ~/llmc/test_scripts/partial_pool.sh << 'EOF'
#!/bin/bash
echo -n "Create a file called"
# Simulate crash mid-output
exit 1
EOF
chmod +x ~/llmc/test_scripts/partial_pool.sh

sed -i.bak 's|empty_pool.sh|partial_pool.sh|' ~/llmc/config.toml
```

**Step 4.2**: Verify daemon handles partial output.

```bash
llmc up --auto &
DAEMON_PID=$!

sleep 20

if ! kill -0 $DAEMON_PID 2>/dev/null; then
    echo "Daemon terminated (expected)"
fi

wait $DAEMON_PID 2>/dev/null
echo "Exit code: $?"
```

**Verify**:
- Non-zero exit code detected despite partial output
- Daemon shuts down
- Partial output logged but not used as task

### Part 5: Command Not Found

**Step 5.1**: Configure non-existent task pool command.

```bash
llmc down --force 2>/dev/null || true

sed -i.bak 's|partial_pool.sh|nonexistent_command.sh|' ~/llmc/config.toml
```

**Step 5.2**: Verify error handling.

```bash
llmc up --auto &
DAEMON_PID=$!

sleep 10

if ! kill -0 $DAEMON_PID 2>/dev/null; then
    echo "Daemon terminated (expected)"
fi

wait $DAEMON_PID 2>/dev/null
echo "Exit code: $?"

cat ~/llmc/logs/auto.log | tail -10
```

**Verify**:
- Clear error about command not found
- Daemon shuts down gracefully
- Helpful error message in logs

## Cleanup

```bash
# Stop any running daemon
kill -SIGINT $DAEMON_PID 2>/dev/null
wait $DAEMON_PID 2>/dev/null

# Remove test artifacts
rm -rf ~/llmc/test_scripts
rm -f ~/Documents/GoogleDrive/dreamtides/delayed_fail_test.txt

# Restore config (remove [auto] section or restore backup)
mv ~/llmc/config.toml.bak ~/llmc/config.toml 2>/dev/null || true

# Clean up git
cd ~/Documents/GoogleDrive/dreamtides
git checkout -- . 2>/dev/null || true
git clean -fd 2>/dev/null || true

llmc down --force 2>/dev/null || true
llmc nuke --all --yes 2>/dev/null || true
```

## Expected Issues to Report

1. Non-zero exit code not detected
2. Daemon continues running after task pool failure
3. Empty output incorrectly treated as error
4. Partial output used as task despite failure
5. Command not found not handled gracefully
6. task_pool.log not capturing stderr
7. Error messages not helpful for debugging
8. Daemon exit code is 0 despite failure

## Abort Conditions

**Abort the test and file a task if:**
- Daemon crashes (segfault, panic) instead of graceful shutdown
- Task pool failure causes state file corruption
- Worker receives partial/invalid task
- System resources leaked after failure
