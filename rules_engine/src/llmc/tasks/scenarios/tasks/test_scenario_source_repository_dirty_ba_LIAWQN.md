---
lattice-id: LIAWQN
name: test-scenario-source-repository-dirty-ba
description: 'Test Scenario: Source Repository Dirty Backoff'
parent-id: LB5WQN
task-type: task
priority: 2
labels:
- testing
- manual-test
- scenario
- auto-mode
- dirty-repo
- llmc-auto
blocked-by:
- LH7WQN
created-at: 2026-01-21T22:03:45.116204Z
updated-at: 2026-01-21T22:31:38.844656Z
---

# Test Scenario: Source Repository Dirty Backoff

## Objective

Verify that auto mode correctly handles the scenario where the source repository
(master) has uncommitted changes, implementing exponential backoff retry instead
of immediate shutdown.

## Prerequisites

- LLMC installed and configured
- No daemon currently running
- Previous test scenario (LCEWQN) completed successfully

## Environment Setup

**This test MUST run in an isolated LLMC instance.** See
`../isolated_test_environment.md` for complete setup instructions.

```bash
# Create isolated test environment
export TEST_DIR="/tmp/llmc-dirty-repo-test-$$"
export LLMC_ROOT="$TEST_DIR"

llmc init --source ~/Documents/GoogleDrive/dreamtides --target "$TEST_DIR"

# Verify isolation
llmc status
```

## Differentiating Errors from Normal Operations

**Error indicators:**

- Daemon crashes when source repo is dirty
- No backoff retry (immediate failure)
- Backoff state not persisted across daemon restarts
- Backoff never resets after successful accept
- Incorrect backoff timing (not exponential)

**Normal operations (expected in this test):**

- Message: "Source repository has uncommitted changes. Will retry in N seconds."
- Exponential backoff: 60s → 120s → 240s → ...
- Daemon continues running during backoff
- Successful accept after repo becomes clean

## Setup

```bash
# Ensure clean state
cd $LLMC_ROOT
llmc down --force 2>/dev/null || true
llmc nuke --all --yes 2>/dev/null || true

# Create test scripts
mkdir -p $LLMC_ROOT/test_scripts
cat > $LLMC_ROOT/test_scripts/dirty_pool.sh << 'EOF'
#!/bin/bash
MARKER=$LLMC_ROOT/test_scripts/.task_issued
if [ ! -f "$MARKER" ]; then
    touch "$MARKER"
    echo "Create a file called dirty_backoff_test.txt containing 'Test file'. Do not create other files."
else
    exit 0
fi
EOF
chmod +x $LLMC_ROOT/test_scripts/dirty_pool.sh

# Configure auto mode
cat >> $LLMC_ROOT/config.toml << 'EOF'

[auto]
task_pool_command = "$LLMC_ROOT/test_scripts/dirty_pool.sh"
concurrency = 1
EOF
```

## Test Sequence

### Part 1: Basic Dirty Repo Detection

**Step 1.1**: Start auto mode and let task begin.

```bash
cd $LLMC_ROOT
llmc up --auto &
DAEMON_PID=$!

# Wait for worker to start working
sleep 20
```

**Step 1.2**: Create uncommitted changes in source repo while worker is working.

```bash
cd ~/Documents/GoogleDrive/dreamtides
echo "Uncommitted work in progress" > uncommitted_file.txt
echo "Source repo now has uncommitted changes"
```

**Step 1.3**: Wait for worker to complete and observe backoff.

```bash
cd $LLMC_ROOT

# Monitor for needs_review then backoff message
for i in {1..60}; do
    STATUS=$(llmc status --json 2>/dev/null | jq -r '.workers[] | select(.name == "auto-1") | .status' 2>/dev/null)
    echo "Worker status: $STATUS"

    # Check daemon output for backoff message
    cat $LLMC_ROOT/logs/auto.log 2>/dev/null | grep -i "uncommitted\|retry\|backoff" | tail -3

    if [ "$STATUS" = "needs_review" ]; then
        echo "Worker ready for accept - should trigger dirty repo check"
    fi

    sleep 5
done
```

**Verify**:

- Daemon detects uncommitted changes
- Daemon logs "Source repository has uncommitted changes" message
- Daemon does NOT shut down
- Worker stays in needs_review (accept deferred)

### Part 2: Exponential Backoff Timing

**Step 2.1**: Monitor backoff intervals.

```bash
# Watch for backoff progression
echo "Monitoring backoff intervals (this takes several minutes)..."

for i in {1..20}; do
    cat $LLMC_ROOT/logs/auto.log 2>/dev/null | grep -i "retry in" | tail -5

    # Check if daemon still running
    if ! kill -0 $DAEMON_PID 2>/dev/null; then
        echo "ERROR: Daemon terminated unexpectedly"
        break
    fi

    sleep 30
done
```

**Verify**:

- First retry: ~60 seconds
- Second retry: ~120 seconds (2x)
- Intervals double each time
- Daemon remains running throughout

**Step 2.2**: Check state file for backoff persistence.

```bash
cat $LLMC_ROOT/state.json | jq '.source_repo_dirty_retry_after_unix, .source_repo_dirty_backoff_secs'
```

**Verify**:

- `source_repo_dirty_retry_after_unix` is set (future timestamp)
- `source_repo_dirty_backoff_secs` shows current backoff value

### Part 3: Recovery After Cleanup

**Step 3.1**: Remove uncommitted changes from source repo.

```bash
cd ~/Documents/GoogleDrive/dreamtides
rm uncommitted_file.txt
git status
echo "Source repo is now clean"
```

**Step 3.2**: Wait for accept to succeed.

```bash
cd $LLMC_ROOT

for i in {1..60}; do
    STATUS=$(llmc status --json 2>/dev/null | jq -r '.workers[] | select(.name == "auto-1") | .status' 2>/dev/null)
    echo "Worker status: $STATUS"

    if [ "$STATUS" = "idle" ]; then
        echo "Worker returned to idle - accept succeeded!"
        break
    fi

    sleep 5
done
```

**Verify**:

- Accept succeeds after source repo is clean
- Worker returns to idle state
- File exists in master repo

**Step 3.3**: Verify backoff state cleared.

```bash
cat $LLMC_ROOT/state.json | jq '.source_repo_dirty_retry_after_unix, .source_repo_dirty_backoff_secs'
```

**Verify**:

- Backoff fields are null/cleared
- Ready for normal operation

### Part 4: Backoff Persistence Across Restart

**Step 4.1**: Create dirty state and restart daemon.

```bash
# Reset for new test
rm -f $LLMC_ROOT/test_scripts/.task_issued
llmc reset auto-1 --yes 2>/dev/null || true

# Create dirty file
cd ~/Documents/GoogleDrive/dreamtides
echo "Another uncommitted file" > uncommitted2.txt

# Restart daemon
cd $LLMC_ROOT
kill -SIGINT $DAEMON_PID 2>/dev/null
wait $DAEMON_PID 2>/dev/null

# Wait for task and backoff to begin
llmc up --auto &
DAEMON_PID=$!
sleep 60

# Note backoff state
cat $LLMC_ROOT/state.json | jq '.source_repo_dirty_backoff_secs'
BACKOFF_BEFORE=$(cat $LLMC_ROOT/state.json | jq '.source_repo_dirty_backoff_secs')
echo "Backoff before restart: $BACKOFF_BEFORE"

# Restart daemon
kill -SIGINT $DAEMON_PID 2>/dev/null
wait $DAEMON_PID 2>/dev/null

llmc up --auto &
DAEMON_PID=$!
sleep 10
```

**Step 4.2**: Verify backoff state preserved.

```bash
BACKOFF_AFTER=$(cat $LLMC_ROOT/state.json | jq '.source_repo_dirty_backoff_secs')
echo "Backoff after restart: $BACKOFF_AFTER"

if [ "$BACKOFF_BEFORE" = "$BACKOFF_AFTER" ]; then
    echo "Backoff state preserved across restart"
else
    echo "WARNING: Backoff state may have changed"
fi
```

**Verify**:

- Backoff state survives daemon restart
- Doesn't reset to initial 60s value

### Part 5: Maximum Backoff Cap

**Step 5.1**: Verify backoff doesn't exceed 1 hour.

```bash
# Check current backoff
cat $LLMC_ROOT/state.json | jq '.source_repo_dirty_backoff_secs'
```

**Note**: This test is time-consuming. For practical testing:

- Verify the backoff math in code review
- Or let daemon run for extended period and check max value

**Verify**:

- Backoff caps at 3600 seconds (1 hour)
- Does not continue doubling beyond cap

### Part 6: NoChanges Task Clears Backoff

**Step 6.1**: Setup task that produces no changes.

```bash
# Clean source repo
cd ~/Documents/GoogleDrive/dreamtides
rm uncommitted2.txt

# Stop daemon and reconfigure
cd $LLMC_ROOT
kill -SIGINT $DAEMON_PID 2>/dev/null
wait $DAEMON_PID 2>/dev/null

rm -f $LLMC_ROOT/test_scripts/.task_issued

cat > $LLMC_ROOT/test_scripts/dirty_pool.sh << 'EOF'
#!/bin/bash
MARKER=$LLMC_ROOT/test_scripts/.task_issued
if [ ! -f "$MARKER" ]; then
    touch "$MARKER"
    echo "Verify the README exists. Do not make any changes. Just confirm the file exists."
else
    exit 0
fi
EOF
```

**Step 6.2**: Verify NoChanges clears backoff.

```bash
# Set artificial backoff state (would require code modification or manual state edit)
# For now, just verify normal completion path

llmc up --auto &
DAEMON_PID=$!

for i in {1..60}; do
    STATUS=$(llmc status --json 2>/dev/null | jq -r '.workers[] | select(.name == "auto-1") | .status' 2>/dev/null)
    if [ "$STATUS" = "idle" ] || [ "$STATUS" = "no_changes" ]; then
        echo "Task completed with status: $STATUS"
        break
    fi
    sleep 5
done

cat $LLMC_ROOT/state.json | jq '.source_repo_dirty_retry_after_unix, .source_repo_dirty_backoff_secs'
```

**Verify**:

- NoChanges completion clears any backoff state
- Daemon ready for next task

## Cleanup

```bash
# Stop daemon
kill -SIGINT $DAEMON_PID 2>/dev/null
wait $DAEMON_PID 2>/dev/null

# Remove test artifacts
rm -rf $LLMC_ROOT/test_scripts
rm -f ~/Documents/GoogleDrive/dreamtides/dirty_backoff_test.txt
rm -f ~/Documents/GoogleDrive/dreamtides/uncommitted*.txt

# Clean up git
cd ~/Documents/GoogleDrive/dreamtides
git checkout -- . 2>/dev/null || true
git clean -fd 2>/dev/null || true

llmc down --force 2>/dev/null || true
llmc nuke --all --yes 2>/dev/null || true
```

## Expected Issues to Report

1. Daemon shuts down instead of implementing backoff
2. Backoff not exponential (fixed interval)
3. Backoff state not persisted in state.json
4. Backoff not preserved across daemon restart
5. Backoff not cleared after successful accept
6. Maximum backoff exceeds 1 hour
7. No user-visible message about backoff
8. Accept attempted during backoff period

## Abort Conditions

**Abort the test and file a task if:**

- Daemon crashes when source repo is dirty
- State file corruption from backoff state
- Accept proceeds despite dirty repo
- Backoff causes daemon to become unresponsive
