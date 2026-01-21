---
lattice-id: LH7WQN
name: test-scenario-merge-conflict-handling-du
description: 'Test Scenario: Merge Conflict Handling During Auto Accept'
parent-id: LB5WQN
task-type: task
priority: 2
labels:
- testing
- manual-test
- scenario
- auto-mode
- merge-conflict
blocked-by:
- LH6WQN
created-at: 2026-01-21T22:02:47.025721Z
updated-at: 2026-01-21T22:31:38.818295Z
---

# Test Scenario: Merge Conflict Handling During Auto Accept

## Objective

Verify that auto mode correctly handles merge conflicts during the accept
workflow,
transitioning workers to `rebasing` state, sending conflict prompts, and
successfully
completing accept after worker resolves conflicts.

## Prerequisites

- LLMC installed and configured
- A clean LLMC workspace
- No daemon currently running
- Previous test scenario (LCDWQN) completed successfully

## Differentiating Errors from Normal Operations

**Error indicators:**

- Worker stuck in `rebasing` state indefinitely (>10 minutes)
- Daemon crashes on conflict detection
- Conflict not detected (accept proceeds with merge commits)
- Worker not receiving conflict resolution prompt
- Conflict resolution prompt malformed

**Normal operations:**

- Worker transitioning to `rebasing` state during conflict
- Worker receiving conflict resolution prompt
- Worker successfully resolving conflicts
- Accept completing after resolution
- Multiple rebase attempts before success

## Setup

```bash
# Ensure clean state
cd ~/llmc
llmc down --force 2>/dev/null || true
llmc nuke --all --yes 2>/dev/null || true

# Create test scripts directory
mkdir -p ~/llmc/test_scripts

# Create a file in master that will conflict
cd ~/Documents/GoogleDrive/dreamtides
cat > conflict_test_file.txt << 'EOF'
Line 1: Original content
Line 2: This will be modified
Line 3: End of file
EOF
git add conflict_test_file.txt
git commit -m "Add conflict test file"

# Record commit for later cleanup
CONFLICT_SETUP_COMMIT=$(git rev-parse HEAD)
echo "$CONFLICT_SETUP_COMMIT" > ~/llmc/test_scripts/.setup_commit

# Create task pool that modifies the same file
cat > ~/llmc/test_scripts/conflict_pool.sh << 'EOF'
#!/bin/bash
MARKER=~/llmc/test_scripts/.task_issued
if [ ! -f "$MARKER" ]; then
    touch "$MARKER"
    cat << 'TASK'
Edit the file conflict_test_file.txt to change Line 2 to say "Line 2: Modified by auto worker".
Make sure to commit your change.
TASK
else
    exit 0
fi
EOF
chmod +x ~/llmc/test_scripts/conflict_pool.sh

# Configure auto mode
cat >> ~/llmc/config.toml << 'EOF'

[auto]
task_pool_command = "~/llmc/test_scripts/conflict_pool.sh"
concurrency = 1
EOF
```

## Test Sequence

### Part 1: Setup Conflict Scenario

**Step 1.1**: Start auto mode to assign task.

```bash
cd ~/llmc
llmc up --auto &
DAEMON_PID=$!

# Wait for worker to start working
sleep 20
```

**Step 1.2**: While worker is working, create conflicting commit on master.

```bash
# Check that worker is working
llmc status

# Create conflicting change in master
cd ~/Documents/GoogleDrive/dreamtides
cat > conflict_test_file.txt << 'EOF'
Line 1: Original content
Line 2: MODIFIED BY MASTER - WILL CONFLICT
Line 3: End of file
EOF
git add conflict_test_file.txt
git commit -m "Create conflict: modify same line"

echo "Conflicting commit created on master"
```

### Part 2: Observe Conflict Detection

**Step 2.1**: Wait for worker to complete and conflict to be detected.

```bash
cd ~/llmc

# Monitor for rebasing state
for i in {1..60}; do
    STATUS=$(llmc status --json 2>/dev/null | jq -r '.workers[] | select(.name == "auto-1") | .status' 2>/dev/null)
    echo "Worker status: $STATUS"

    if [ "$STATUS" = "rebasing" ]; then
        echo "Conflict detected - worker in rebasing state"
        break
    fi

    sleep 5
done
```

**Verify**:

- Worker transitions to `rebasing` state
- Daemon does NOT shut down

**Step 2.2**: Check conflict prompt sent to worker.

```bash
llmc peek auto-1 --lines 100 | tail -50
```

**Verify**:

- Worker received conflict resolution prompt
- Prompt includes information about conflicting files
- Prompt includes resolution instructions

### Part 3: Monitor Conflict Resolution

**Step 3.1**: Watch worker resolve conflict.

```bash
# Monitor worker progress
for i in {1..90}; do
    STATUS=$(llmc status --json 2>/dev/null | jq -r '.workers[] | select(.name == "auto-1") | .status' 2>/dev/null)
    echo "Worker status: $STATUS (iteration $i)"

    if [ "$STATUS" = "idle" ]; then
        echo "Worker returned to idle - conflict resolved and accepted"
        break
    elif [ "$STATUS" = "needs_review" ]; then
        echo "Worker completed conflict resolution - awaiting auto-accept"
    fi

    sleep 5
done
```

**Verify**:

- Worker eventually returns to `idle` state
- Worker successfully resolved the conflict
- Changes were accepted

**Step 3.2**: Verify final state of conflict file.

```bash
cd ~/Documents/GoogleDrive/dreamtides
cat conflict_test_file.txt
git log --oneline -5
```

**Verify**:

- File contains worker's change (not master's conflicting change, unless merged)
- Git history shows successful merge/rebase
- No merge commits (fast-forward only)

### Part 4: Conflict Resolution Logging

**Step 4.1**: Check auto.log for conflict handling.

```bash
cat ~/llmc/logs/auto.log | grep -i "conflict\|rebase\|resolv" | tail -20
```

**Verify**:

- Log shows conflict detection
- Log shows transition to rebasing state
- Log shows conflict resolution prompt sent
- Log shows successful accept after resolution

### Part 5: Unresolvable Conflict (Edge Case)

**Step 5.1**: Setup scenario where conflict cannot be auto-resolved.

```bash
# Stop current daemon
kill -SIGINT $DAEMON_PID 2>/dev/null
wait $DAEMON_PID 2>/dev/null

# Reset for new test
llmc reset auto-1 --yes 2>/dev/null || true
rm -f ~/llmc/test_scripts/.task_issued

# Create complex conflict that's hard to resolve
cd ~/Documents/GoogleDrive/dreamtides

# Create file with complex structure
cat > complex_conflict.txt << 'EOF'
// Header
function process() {
    // Original implementation
    return originalValue;
}
// Footer
EOF
git add complex_conflict.txt
git commit -m "Add complex file for conflict test"

# Create task that heavily modifies the file
cat > ~/llmc/test_scripts/conflict_pool.sh << 'EOF'
#!/bin/bash
MARKER=~/llmc/test_scripts/.task_issued
if [ ! -f "$MARKER" ]; then
    touch "$MARKER"
    cat << 'TASK'
Completely rewrite complex_conflict.txt to implement a new algorithm:
```
// New Header - Rewritten
function newProcess() {
    // Completely new implementation
    const result = computeNewValue();
    return transformResult(result);
}
// New Footer
```
Commit the change.
TASK
else
    exit 0
fi
EOF
```

**Step 5.2**: Create conflicting change and start daemon.

```bash
llmc up --auto &
DAEMON_PID=$!

# Wait for worker to start
sleep 30

# Create very different change on master
cd ~/Documents/GoogleDrive/dreamtides
cat > complex_conflict.txt << 'EOF'
// MASTER Header - Different structure
class Processor {
    execute() {
        // Master implementation
        return masterValue;
    }
}
// MASTER Footer
EOF
git add complex_conflict.txt
git commit -m "Master: completely different structure"
```

**Step 5.3**: Monitor complex conflict handling.

```bash
cd ~/llmc

for i in {1..120}; do
    STATUS=$(llmc status --json 2>/dev/null | jq -r '.workers[] | select(.name == "auto-1") | .status' 2>/dev/null)
    echo "Worker status: $STATUS"

    if [ "$STATUS" = "idle" ] || [ "$STATUS" = "error" ]; then
        echo "Final state reached: $STATUS"
        break
    fi

    sleep 5
done

# Check final result
cd ~/Documents/GoogleDrive/dreamtides
cat complex_conflict.txt
git log --oneline -5
```

**Verify**:

- Worker either resolves complex conflict or escalates appropriately
- No daemon crash
- State is consistent (idle or error, not stuck)

## Cleanup

```bash
# Stop daemon
kill -SIGINT $DAEMON_PID 2>/dev/null
wait $DAEMON_PID 2>/dev/null

# Remove test artifacts
rm -rf ~/llmc/test_scripts
rm -f ~/Documents/GoogleDrive/dreamtides/conflict_test_file.txt
rm -f ~/Documents/GoogleDrive/dreamtides/complex_conflict.txt

# Reset git to before test
cd ~/Documents/GoogleDrive/dreamtides
SETUP_COMMIT=$(cat ~/llmc/test_scripts/.setup_commit 2>/dev/null)
[ -n "$SETUP_COMMIT" ] && git reset --hard ${SETUP_COMMIT}^ 2>/dev/null || true

git checkout -- . 2>/dev/null || true
git clean -fd 2>/dev/null || true

llmc down --force 2>/dev/null || true
llmc nuke --all --yes 2>/dev/null || true
```

## Expected Issues to Report

1. Conflict not detected during accept
2. Worker not transitioned to rebasing state
3. Conflict resolution prompt not sent
4. Worker stuck in rebasing state forever
5. Daemon crashes on conflict
6. Accept creates merge commit instead of fast-forward
7. Worker changes lost during conflict resolution
8. Conflict resolution retried infinitely

## Abort Conditions

**Abort the test and file a task if:**

- Git repository becomes corrupted
- Worker worktree left in unrecoverable state
- Daemon panics during conflict handling
- State file becomes inconsistent
- Worker receives malformed conflict prompt
