---
lattice-id: LH3WQN
name: test-scenario-auto-mode-concurrency-with
description: 'Test Scenario: Auto Mode Concurrency with Multiple Workers'
parent-id: LB5WQN
task-type: task
priority: 2
labels:
- testing
- manual-test
- scenario
- auto-mode
- concurrency
blocked-by:
- LH2WQN
created-at: 2026-01-21T21:59:55.443775Z
updated-at: 2026-01-21T22:31:38.807512Z
---

# Test Scenario: Auto Mode Concurrency with Multiple Workers

## Objective

Verify that auto mode correctly manages multiple concurrent auto workers,
distributing
tasks appropriately and handling simultaneous completions without race
conditions.

## Prerequisites

- LLMC installed and configured
- A clean LLMC workspace
- No daemon currently running
- Previous test scenario (LB7WQN) completed successfully

## Differentiating Errors from Normal Operations

**Error indicators:**

- Workers receiving duplicate tasks
- Race conditions during accept (git conflicts between auto workers)
- Workers stuck waiting when tasks are available
- Any `ERROR` or `WARN` in logs
- Heartbeat becoming stale while daemon appears to be running
- Deadlock symptoms (all workers idle despite tasks available)

**Normal operations:**

- Workers processing tasks at different speeds
- Some workers idle while others work (if fewer tasks than workers)
- Sequential commits to master (one at a time)
- Post-accept commands blocking subsequent accepts

## Setup

```bash
# Ensure clean state
cd ~/llmc
llmc down --force 2>/dev/null || true
llmc nuke --all --yes 2>/dev/null || true

# Create a task pool that returns multiple tasks
mkdir -p ~/llmc/test_scripts
cat > ~/llmc/test_scripts/concurrent_pool.sh << 'EOF'
#!/bin/bash
COUNTER_FILE=~/llmc/test_scripts/.task_counter
if [ ! -f "$COUNTER_FILE" ]; then
    echo "0" > "$COUNTER_FILE"
fi

COUNTER=$(cat "$COUNTER_FILE")
if [ "$COUNTER" -lt 4 ]; then
    COUNTER=$((COUNTER + 1))
    echo "$COUNTER" > "$COUNTER_FILE"
    echo "Create a file called concurrent_test_${COUNTER}.txt containing 'Task ${COUNTER} completed by auto worker'. Do not create other files."
else
    # No more tasks
    exit 0
fi
EOF
chmod +x ~/llmc/test_scripts/concurrent_pool.sh

# Configure auto mode with concurrency=2
cat >> ~/llmc/config.toml << 'EOF'

[auto]
task_pool_command = "~/llmc/test_scripts/concurrent_pool.sh"
concurrency = 2
EOF
```

## Test Sequence

### Part 1: Concurrent Worker Creation

**Step 1.1**: Start auto mode with concurrency=2.

```bash
llmc up --auto &
DAEMON_PID=$!
sleep 10
```

**Verify**:

- Both `auto-1` and `auto-2` workers are created
- `llmc status` shows both under "Auto Workers" section
- Both workers show in separate TMUX sessions (`llmc-auto-1`, `llmc-auto-2`)

**Step 1.2**: Verify worker isolation.

```bash
llmc status --json | jq '.workers[] | select(.name | startswith("auto-")) | {name, status, worktree_path}'
```

**Verify**:

- Each worker has distinct worktree path
- Worktrees are at `.worktrees/auto-1` and `.worktrees/auto-2`

### Part 2: Concurrent Task Processing

**Step 2.1**: Monitor both workers processing tasks.

```bash
# Poll status to observe concurrent processing
for i in {1..10}; do
    echo "=== Check $i ==="
    llmc status | grep -A5 "Auto Workers"
    sleep 10
done
```

**Verify**:

- Both workers receive tasks
- Workers may be in different states simultaneously (one working, one idle)
- No worker receives a duplicate task

**Step 2.2**: Wait for all tasks to complete.

```bash
# Wait until all 4 tasks are done
for i in {1..120}; do
    TASK_COUNT=$(ls ~/Documents/GoogleDrive/dreamtides/concurrent_test_*.txt 2>/dev/null | wc -l)
    if [ "$TASK_COUNT" -ge 4 ]; then
        echo "All 4 tasks completed"
        break
    fi
    sleep 5
done
```

**Verify**:

- All 4 files created: `concurrent_test_1.txt` through `concurrent_test_4.txt`
- Each file has correct content
- No duplicate files

### Part 3: Sequential Accept Verification

**Step 3.1**: Check git history shows clean sequential commits.

```bash
cd ~/Documents/GoogleDrive/dreamtides
git log --oneline -10
```

**Verify**:

- 4 separate commits for the 4 tasks
- No merge commits (all fast-forward)
- Commits appear in order (may not be task order, depends on completion order)

**Step 3.2**: Verify no interleaved commits.

```bash
git log --format="%h %s" -10 | grep -i "concurrent_test"
```

**Verify**:

- Each task has exactly one commit
- No partial commits or duplicates

### Part 4: Stress Test - All Workers Busy

**Step 4.1**: Observe behavior when all workers are busy.

```bash
# Check logs to see task queuing behavior
cat ~/llmc/logs/auto.log | grep -i "idle\|task\|assign" | tail -30
```

**Verify**:

- Tasks are only assigned when workers are idle
- No task is lost when workers are busy
- Daemon correctly waits for available workers

### Part 5: Shutdown with Active Workers

**Step 5.1**: Note current state and initiate shutdown.

```bash
llmc status
kill -SIGINT $DAEMON_PID
```

**Verify**:

- Shutdown initiates gracefully
- Active workers receive shutdown signal (Ctrl-C to Claude)

**Step 5.2**: Wait for shutdown completion.

```bash
wait $DAEMON_PID 2>/dev/null
echo "Exit code: $?"
```

**Verify**:

- All workers stopped
- Worktrees preserved (not deleted)
- Exit code is 0 for graceful shutdown

## Cleanup

```bash
# Remove test artifacts
rm -rf ~/llmc/test_scripts
rm -f ~/Documents/GoogleDrive/dreamtides/concurrent_test_*.txt

# Clean up git
cd ~/Documents/GoogleDrive/dreamtides
git checkout -- . 2>/dev/null || true
git clean -fd 2>/dev/null || true

# Reset repo to before test commits
git log --oneline -20 | grep -i "concurrent_test" | tail -1 | cut -d' ' -f1 | xargs -I{} git reset --hard {}^ 2>/dev/null || true

llmc down --force 2>/dev/null || true
llmc nuke --all --yes 2>/dev/null || true

# Remove auto config section from config.toml
```

## Expected Issues to Report

1. Only one auto worker created despite concurrency=2
2. Same task assigned to multiple workers
3. Race condition during accept (git errors)
4. Workers deadlocked waiting for each other
5. Heartbeat stops while workers are still running
6. Shutdown doesn't wait for active workers
7. Worktrees left in dirty state after shutdown
8. Git history shows merge commits instead of fast-forward
9. Task pool called more times than tasks needed

## Abort Conditions

**Abort the test and file a task if:**

- Daemon panics during concurrent operations
- Git repository becomes corrupted
- Workers interfere with each other's worktrees
- State file shows inconsistent worker states
- Memory usage grows unbounded
