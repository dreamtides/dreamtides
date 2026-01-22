---
lattice-id: LH4WQN
name: test-scenario-auto-accept-workflow-with-
description: 'Test Scenario: Auto Accept Workflow with Rebase and Squash'
parent-id: LB5WQN
task-type: task
priority: 2
labels:
- testing
- manual-test
- scenario
- auto-mode
- accept
- llmc-auto
blocked-by:
- LH3WQN
created-at: 2026-01-21T22:00:37.426141Z
updated-at: 2026-01-21T22:31:38.801638Z
---

# Test Scenario: Auto Accept Workflow with Rebase and Squash

## Objective

Verify that the auto accept workflow correctly rebases worker changes onto
master,
squashes multiple commits into a single commit, strips agent attribution, and
performs a fast-forward merge.

## Prerequisites

- LLMC installed and configured
- No daemon currently running
- Previous test scenario (LCAWQN) completed successfully

## Environment Setup

**This test MUST run in an isolated LLMC instance.** See
`../isolated_test_environment.md` for complete setup instructions.

```bash
# Create isolated test environment
export TEST_DIR="/tmp/llmc-auto-accept-test-$$"
export LLMC_ROOT="$TEST_DIR"

llmc init --source ~/Documents/GoogleDrive/dreamtides --target "$TEST_DIR"

# Verify isolation
llmc status
```

## Differentiating Errors from Normal Operations

**Error indicators:**

- Rebase failures that trigger daemon shutdown
- Non-fast-forward merges (merge commits in history)
- Attribution markers remaining in commit messages ("Generated with",
  "Co-Authored-By")
- Multiple commits per task in master history
- Worker not reset to idle after accept
- Git lock errors persisting after retries

**Normal operations:**

- Temporary rebase-in-progress state during accept
- Brief delay while squashing commits
- Worker transitioning through needs_review before auto-accept

## Setup

```bash
# Ensure clean state
cd $LLMC_ROOT
llmc down --force 2>/dev/null || true
llmc nuke --all --yes 2>/dev/null || true

# Create a task pool script
mkdir -p $LLMC_ROOT/test_scripts
cat > $LLMC_ROOT/test_scripts/accept_pool.sh << 'EOF'
#!/bin/bash
MARKER=$LLMC_ROOT/test_scripts/.accept_task_issued
if [ ! -f "$MARKER" ]; then
    touch "$MARKER"
    cat << 'TASK'
Create a feature that involves multiple commits:
1. First, create a file called accept_test_part1.txt with content "Part 1"
2. Commit that change with message "Add part 1"
3. Then create a file called accept_test_part2.txt with content "Part 2"
4. Commit that change with message "Add part 2"
5. Finally create accept_test_combined.txt with content "Combined feature"
6. Commit that with message "Add combined"

Make exactly 3 separate commits as described.
TASK
else
    exit 0
fi
EOF
chmod +x $LLMC_ROOT/test_scripts/accept_pool.sh

# Configure auto mode
cat >> $LLMC_ROOT/config.toml << 'EOF'

[auto]
task_pool_command = "$LLMC_ROOT/test_scripts/accept_pool.sh"
concurrency = 1
EOF

# Record current master HEAD for later verification
cd ~/Documents/GoogleDrive/dreamtides
MASTER_HEAD_BEFORE=$(git rev-parse HEAD)
echo "$MASTER_HEAD_BEFORE" > $LLMC_ROOT/test_scripts/.master_head_before
```

## Test Sequence

### Part 1: Task Execution with Multiple Commits

**Step 1.1**: Start auto mode and let worker create multiple commits.

```bash
cd $LLMC_ROOT
llmc up --auto &
DAEMON_PID=$!

# Wait for task to be assigned and worker to start working
sleep 30
```

**Step 1.2**: Monitor worker progress.

```bash
# Check worker is making commits
for i in {1..60}; do
    WORKER_STATUS=$(llmc status --json 2>/dev/null | jq -r '.workers[] | select(.name == "auto-1") | .status' 2>/dev/null)
    echo "Worker status: $WORKER_STATUS"

    # Check commits in worktree
    cd $LLMC_ROOT/.worktrees/auto-1 2>/dev/null && git log --oneline -5 2>/dev/null
    cd $LLMC_ROOT

    if [ "$WORKER_STATUS" = "idle" ]; then
        echo "Task completed"
        break
    fi
    sleep 10
done
```

**Verify**:

- Worker creates multiple commits during task
- Worker transitions through working → needs_review → idle

### Part 2: Squash Verification

**Step 2.1**: Check master history after accept.

```bash
cd ~/Documents/GoogleDrive/dreamtides
git log --oneline -10
```

**Verify**:

- Only ONE new commit since $MASTER_HEAD_BEFORE
- Multiple worker commits squashed into single commit
- Commit message is reasonable (not "Add part 1\nAdd part 2\nAdd combined")

**Step 2.2**: Verify all files present.

```bash
ls accept_test_*.txt
cat accept_test_part1.txt
cat accept_test_part2.txt
cat accept_test_combined.txt
```

**Verify**:

- All 3 files exist
- Content is correct
- All in single commit

### Part 3: Attribution Stripping

**Step 3.1**: Check commit message for stripped attribution.

```bash
git log -1 --format="%B"
```

**Verify**:

- No "Generated with Claude" or similar
- No "Co-Authored-By: Claude" or similar
- Clean commit message

**Step 3.2**: Check commit author.

```bash
git log -1 --format="%an <%ae>"
```

**Verify**:

- Author is NOT "Claude" or similar
- Author matches repository configuration

### Part 4: Fast-Forward Merge Verification

**Step 4.1**: Verify no merge commits.

```bash
MASTER_HEAD_BEFORE=$(cat $LLMC_ROOT/test_scripts/.master_head_before)
git log --oneline $MASTER_HEAD_BEFORE..HEAD
```

**Verify**:

- Linear history (no merge commits)
- Only new commits from auto worker

**Step 4.2**: Verify branch was deleted.

```bash
git branch -a | grep "llmc/auto-1" || echo "Branch correctly deleted"
```

**Verify**:

- No `llmc/auto-1` branch exists after accept

### Part 5: Worker Reset Verification

**Step 5.1**: Check worker state.

```bash
llmc status
```

**Verify**:

- Worker `auto-1` is in `idle` state
- Worker has fresh worktree
- No pending task

**Step 5.2**: Verify worktree is clean and at master.

```bash
cd $LLMC_ROOT/.worktrees/auto-1
git status
git log -1 --oneline
cd ~/Documents/GoogleDrive/dreamtides
git log -1 --oneline
```

**Verify**:

- Worktree is clean (no uncommitted changes)
- Worktree HEAD matches master HEAD

### Part 6: Accept Logging

**Step 6.1**: Check auto.log for accept operations.

```bash
cat $LLMC_ROOT/logs/auto.log | grep -i "accept\|rebase\|squash\|merge" | tail -20
```

**Verify**:

- Log shows rebase operation
- Log shows squash operation
- Log shows successful merge
- No errors

## Cleanup

```bash
# Stop daemon
kill -SIGINT $DAEMON_PID 2>/dev/null
wait $DAEMON_PID 2>/dev/null

# Remove test artifacts
rm -rf $LLMC_ROOT/test_scripts
rm -f ~/Documents/GoogleDrive/dreamtides/accept_test_*.txt

# Clean up git
cd ~/Documents/GoogleDrive/dreamtides
git checkout -- . 2>/dev/null || true
git clean -fd 2>/dev/null || true

# Reset to before test
MASTER_HEAD_BEFORE=$(cat $LLMC_ROOT/test_scripts/.master_head_before 2>/dev/null)
[ -n "$MASTER_HEAD_BEFORE" ] && git reset --hard $MASTER_HEAD_BEFORE

llmc down --force 2>/dev/null || true
llmc nuke --all --yes 2>/dev/null || true
```

## Expected Issues to Report

1. Multiple commits not squashed into one
2. Attribution markers remaining in commit message
3. Merge commit created instead of fast-forward
4. Worker branch not deleted after accept
5. Worker not reset to idle state
6. Worktree not recreated after accept
7. Files missing after squash
8. Rebase fails unexpectedly
9. Git lock file prevents accept completion

## Abort Conditions

**Abort the test and file a task if:**

- Accept corrupts git history
- Worker gets stuck in needs_review indefinitely
- Daemon crashes during accept
- Squash loses file changes
- Master branch diverges unexpectedly
