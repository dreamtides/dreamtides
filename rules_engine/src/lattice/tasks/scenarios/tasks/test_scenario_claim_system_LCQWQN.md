---
lattice-id: LCQWQN
name: test-scenario-claim-system
description: 'Test Scenario: Claim System'
task-type: task
priority: 2
labels:
- testing
- manual-test
- scenario
created-at: 2026-01-20T06:17:15.139979Z
updated-at: 2026-01-20T06:17:15.139979Z
---

# Test Scenario: Claim System

See [Agent Manual Testing Guide](../../docs/agent_manual_testing.md#LCBWQN) for
general testing instructions.

## Objective

Verify that the claim system for tracking local work-in-progress operates
correctly, including claiming, releasing, and interaction with task lifecycle.

## Important Note

Claims are stored in `~/.lattice/claims/`, NOT in the git repository. This test
will modify your user's claim storage. The test should clean up after itself,
but be aware of this side effect.

## Setup

```bash
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"
git init
git config user.email "test@example.com"
git config user.name "Test Agent"
mkdir -p .lattice project/tasks

lat create project/project.md "Test Project"
ROOT_ID=$(grep "lattice-id:" project/project.md | cut -d' ' -f2)

# Create several tasks
lat create project/ "Task Alpha" -t task -p 1
ALPHA_ID=$(ls project/tasks/*.md | tail -1 | xargs grep "lattice-id:" | cut -d' ' -f2)

lat create project/ "Task Beta" -t task -p 2
BETA_ID=$(ls project/tasks/*.md | tail -1 | xargs grep "lattice-id:" | cut -d' ' -f2)

lat create project/ "Task Gamma" -t task -p 3
GAMMA_ID=$(ls project/tasks/*.md | tail -1 | xargs grep "lattice-id:" | cut -d' ' -f2)

git add .
git commit -m "Initial tasks"

# Clean up any existing claims for this repo
lat claim --release-all 2>/dev/null || true
```

## Test Sequence

### Part 1: Basic Claim Operations

**Step 1.1**: Claim a task.

```bash
lat claim $ALPHA_ID
```

**Verify**:
- Command succeeds
- Claim file created in `~/.lattice/claims/<repo-hash>/`

**Step 1.2**: Verify claim shows in task view.

```bash
lat show $ALPHA_ID
```

**Verify**:
- Output shows "Claimed: true" or similar indicator

**Step 1.3**: Verify claimed task in JSON.

```bash
lat show $ALPHA_ID --json
```

**Verify**:
- JSON has `"claimed": true`

### Part 2: Claims and Ready Queue

**Step 2.1**: Check ready queue without claimed.

```bash
lat ready
```

**Verify**:
- Task Alpha NOT shown (it's claimed)
- Tasks Beta and Gamma shown

**Step 2.2**: Ready with include-claimed.

```bash
lat ready --include-claimed
```

**Verify**:
- Task Alpha IS shown
- Marked as claimed (e.g., [CLAIMED] indicator)

### Part 3: List Claims

**Step 3.1**: List all claims.

```bash
lat claim --list
```

**Verify**:
- Shows Alpha is claimed
- Shows claim timestamp
- Shows work path (current directory)

**Step 3.2**: Claim another task.

```bash
lat claim $BETA_ID
lat claim --list
```

**Verify**:
- Both Alpha and Beta shown as claimed

### Part 4: Release Claims

**Step 4.1**: Release specific claim.

```bash
lat claim --release $ALPHA_ID
```

**Verify**:
- Command succeeds
- Alpha no longer claimed

**Step 4.2**: Verify release.

```bash
lat claim --list
```

**Verify**:
- Only Beta shown (Alpha released)

```bash
lat ready
```

**Verify**:
- Alpha now appears in ready list (no longer claimed)

### Part 5: Release All Claims

**Step 5.1**: Claim multiple tasks.

```bash
lat claim $ALPHA_ID
lat claim $GAMMA_ID
lat claim --list
```

**Verify**:
- Alpha, Beta, Gamma all claimed

**Step 5.2**: Release all.

```bash
lat claim --release-all
```

**Verify**:
- All claims released

**Step 5.3**: Verify.

```bash
lat claim --list
```

**Verify**:
- No claims listed

### Part 6: Auto-Release on Close

**Step 6.1**: Claim and close.

```bash
lat claim $ALPHA_ID
lat claim --list  # Alpha is claimed
lat close $ALPHA_ID
```

**Verify**:
- Task closed successfully

**Step 6.2**: Verify claim auto-released.

```bash
lat claim --list
```

**Verify**:
- Alpha NOT in claim list (auto-released on close)

### Part 7: Claiming Closed Tasks

**Step 7.1**: Try to claim closed task.

```bash
lat claim $ALPHA_ID  # Alpha is closed
```

**Verify**:
- Should fail or warn (can't claim closed task)

### Part 8: Claiming Blocked Tasks

**Step 8.1**: Create blocking relationship.

```bash
lat reopen $ALPHA_ID
lat dep add $BETA_ID $ALPHA_ID  # Beta blocked by Alpha
```

**Step 8.2**: Claim blocked task.

```bash
lat claim $BETA_ID
```

**Verify**:
- May succeed or warn (policy decision)
- If succeeds, task is claimed even though blocked

### Part 9: Garbage Collection

**Step 9.1**: Create stale claim scenario.

```bash
lat claim $GAMMA_ID
lat close $GAMMA_ID  # Should auto-release, but let's test gc
```

**Step 9.2**: Run garbage collection.

```bash
lat claim --gc
```

**Verify**:
- Reports any stale claims cleaned up
- Claims for closed tasks removed

### Part 10: Edge Cases

**Step 10.1**: Claim non-existent task.

```bash
lat claim LNONEXISTENT
```

**Verify**:
- Fails with not found error
- Exit code 4

**Step 10.2**: Double claim same task.

```bash
lat reopen $ALPHA_ID
lat claim $ALPHA_ID
lat claim $ALPHA_ID
```

**Verify**:
- Second claim is idempotent (no error)
- Still only one claim entry

**Step 10.3**: Release non-claimed task.

```bash
lat claim --release $BETA_ID  # Not currently claimed
```

**Verify**:
- Handles gracefully (no error or clear message)

### Part 11: Claim Storage Location

**Step 11.1**: Verify claim storage.

```bash
# Claims should be in ~/.lattice/claims/<hash>/
ls ~/.lattice/claims/ 2>/dev/null || echo "No claims directory"
```

**Verify**:
- Claims directory exists after claiming
- Subdirectory named by repo hash

### Part 12: Multiple Repos (Advanced)

**Step 12.1**: Create second repo.

```bash
SECOND_DIR=$(mktemp -d)
cd "$SECOND_DIR"
git init
git config user.email "test@example.com"
git config user.name "Test Agent"
mkdir -p .lattice project/tasks

lat create project/project.md "Second Project"
lat create project/ "Second task" -t task
SECOND_TASK_ID=$(ls project/tasks/*.md | tail -1 | xargs grep "lattice-id:" | cut -d' ' -f2)

lat claim $SECOND_TASK_ID
```

**Step 12.2**: Verify claims are repo-scoped.

```bash
cd "$TEST_DIR"
lat claim --list
```

**Verify**:
- Claims from second repo NOT shown
- Only shows claims for current repo

**Step 12.3**: Cleanup second repo.

```bash
cd "$SECOND_DIR"
lat claim --release-all
cd "$TEST_DIR"
rm -rf "$SECOND_DIR"
```

### Part 13: Doctor Integration

**Step 13.1**: Doctor checks claims.

```bash
lat claim $ALPHA_ID
lat close $ALPHA_ID  # Creates potential stale claim
lat doctor
```

**Verify**:
- Doctor warns about stale claim (if any)

**Step 13.2**: Doctor fix cleans claims.

```bash
lat doctor --fix
lat claim --list
```

**Verify**:
- Stale claims cleaned up

## Cleanup

```bash
lat claim --release-all
cd /
rm -rf "$TEST_DIR"
```

## Expected Issues to Report

1. Claims not persisting to ~/.lattice/claims/
2. Claimed tasks appearing in ready queue
3. Auto-release on close not working
4. Claims not repo-scoped
5. --gc not cleaning stale claims
6. Doctor not detecting claim issues
7. Any panics in claim operations
8. Claim file format issues
