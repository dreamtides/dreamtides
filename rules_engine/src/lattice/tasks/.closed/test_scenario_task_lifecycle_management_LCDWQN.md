---
lattice-id: LCDWQN
name: test-scenario-task-lifecycle-management
description: 'Test Scenario: Task Lifecycle Management'
task-type: task
priority: 2
labels:
- testing
- manual-test
- scenario
created-at: 2026-01-20T06:13:20.678397Z
updated-at: 2026-01-20T06:13:20.678397Z
---

# Test Scenario: Task Lifecycle Management

See [Agent Manual Testing Guide](../../docs/agent_manual_testing.md#LCBWQN) for
general testing instructions.

## Objective

Verify that task lifecycle operations (close, reopen, prune) work correctly,
including filesystem moves, link updates, and state transitions.

## Setup

```bash
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"
git init
git config user.email "test@example.com"
git config user.name "Test Agent"
mkdir -p .lattice

# Create root document
mkdir -p auth
lat create auth/auth.md "Authentication System"
ROOT_ID=$(grep "lattice-id:" auth/auth.md | cut -d' ' -f2)

# Create several tasks
lat create auth/ "Fix login bug" -t bug -p 1
BUG_ID=$(ls auth/tasks/*.md | head -1 | xargs grep "lattice-id:" | cut -d' ' -f2)

lat create auth/ "Add OAuth support" -t feature -p 2
FEATURE_ID=$(ls auth/tasks/*.md | tail -1 | xargs grep "lattice-id:" | cut -d' ' -f2)

lat create auth/ "Update auth docs" -t task -p 3
TASK_ID=$(ls auth/tasks/*.md | tail -1 | xargs grep "lattice-id:" | cut -d' ' -f2)

# Commit initial state
git add .
git commit -m "Initial tasks"
```

## Test Sequence

### Part 1: Basic Close Operation

**Step 1.1**: Close a task.

```bash
lat close $BUG_ID
```

**Verify**:
- Task file moved from `auth/tasks/` to `auth/tasks/.closed/`
- Original file no longer exists at old path
- `.closed/` directory created if it didn't exist
- Frontmatter now includes `closed-at` timestamp

**Step 1.2**: Verify closed state in show.

```bash
lat show $BUG_ID
```

**Verify**:
- State shows as "closed"
- Path in output shows `.closed/` in path

**Step 1.3**: Verify closed state in list.

```bash
lat list
```

**Verify**:
- Closed task NOT shown (default excludes closed)

```bash
lat list --include-closed
```

**Verify**:
- Closed task IS shown
- Shows closed indicator

**Step 1.4**: Verify closed task not in ready.

```bash
lat ready
```

**Verify**:
- Closed task not in ready list

### Part 2: Close Multiple Tasks

**Step 2.1**: Close multiple tasks at once.

```bash
lat close $FEATURE_ID $TASK_ID
```

**Verify**:
- Both tasks moved to `.closed/`
- Both have `closed-at` timestamps

### Part 3: Reopen Operation

**Step 3.1**: Reopen a closed task.

```bash
lat reopen $BUG_ID
```

**Verify**:
- Task moved back from `auth/tasks/.closed/` to `auth/tasks/`
- `closed-at` timestamp removed or cleared
- State shows as "open" in `lat show`

**Step 3.2**: Verify reopened task appears in ready.

```bash
lat ready
```

**Verify**:
- Reopened task appears (if no blockers)

### Part 4: Close with Reason

**Step 4.1**: Close a task with a reason.

```bash
lat close $BUG_ID --reason "Fixed in commit abc123"
```

**Verify**:
- Task closed successfully
- Reason text appended to document body

**Step 4.2**: View closed task body.

```bash
lat show $BUG_ID
```

**Verify**:
- Body includes the closure reason

### Part 5: Prune Operations

**Step 5.1**: Prune a specific closed task.

```bash
# Get the path of a closed task
CLOSED_PATH="auth/tasks/.closed/"
lat prune $CLOSED_PATH
```

**Verify**:
- Task file permanently deleted
- `lat show` for that ID returns "not found"
- `lat list --include-closed` no longer shows it

**Step 5.2**: Attempt prune without path or --all.

```bash
lat prune
```

**Verify**:
- Command fails (requires path or --all)
- Exit code 3 (user error)

**Step 5.3**: Create and close more tasks, then prune all.

```bash
lat create auth/ "Temp task 1" -t task
TEMP1_ID=$(ls auth/tasks/*.md | tail -1 | xargs grep "lattice-id:" | cut -d' ' -f2)
lat create auth/ "Temp task 2" -t task
TEMP2_ID=$(ls auth/tasks/*.md | tail -1 | xargs grep "lattice-id:" | cut -d' ' -f2)
lat close $TEMP1_ID $TEMP2_ID
lat prune --all
```

**Verify**:
- All closed tasks deleted
- `auth/tasks/.closed/` directory may be empty or removed
- No closed tasks in `lat list --include-closed`

### Part 6: Prune with References

**Step 6.1**: Create task with reference to another task.

```bash
lat create auth/ "Main feature" -t feature
MAIN_ID=$(ls auth/tasks/*.md | tail -1 | xargs grep "lattice-id:" | cut -d' ' -f2)

lat create auth/ "Sub task" -t task
SUB_ID=$(ls auth/tasks/*.md | tail -1 | xargs grep "lattice-id:" | cut -d' ' -f2)

# Add inline link to main task in sub task body
# (would need to edit file manually or use lat's features)
```

**Step 6.2**: Close and attempt prune with inline links.

```bash
lat close $SUB_ID
lat prune auth/tasks/.closed/
```

**Verify**:
- If there are inline markdown links TO the pruned task, command should error
- Use `--force` to convert links to plain text

**Step 6.3**: Prune with --force.

```bash
lat prune auth/tasks/.closed/ --force
```

**Verify**:
- Task deleted
- Inline links converted to plain text

### Part 7: Dry Run Operations

**Step 7.1**: Close with --dry-run.

```bash
lat close $MAIN_ID --dry-run
```

**Verify**:
- Shows what would happen
- Task NOT actually moved

**Step 7.2**: Reopen with --dry-run.

```bash
lat close $MAIN_ID
lat reopen $MAIN_ID --dry-run
```

**Verify**:
- Shows what would happen
- Task NOT actually moved back

**Step 7.3**: Prune with --dry-run.

```bash
lat prune --all --dry-run
```

**Verify**:
- Shows what would be deleted
- Nothing actually deleted

### Part 8: Edge Cases

**Step 8.1**: Close already closed task.

```bash
lat close $MAIN_ID
lat close $MAIN_ID
```

**Verify**:
- Second close is idempotent or gives clear message
- No error

**Step 8.2**: Reopen already open task.

```bash
lat reopen $MAIN_ID
lat reopen $MAIN_ID
```

**Verify**:
- Second reopen is idempotent or gives clear message
- No error

**Step 8.3**: Close/reopen non-existent task.

```bash
lat close LNONEXISTENT
lat reopen LNONEXISTENT
```

**Verify**:
- Clear error message
- Exit code 4 (not found)

## Cleanup

```bash
cd /
rm -rf "$TEST_DIR"
```

## Expected Issues to Report

1. Files not moved to correct `.closed/` directory
2. `closed-at` timestamp not set or cleared appropriately
3. Closed tasks appearing in `lat ready`
4. Prune not removing references from frontmatter
5. Prune --force not working on inline links
6. Any panics during lifecycle operations
7. Index not updated after close/reopen
