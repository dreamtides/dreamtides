---
lattice-id: LCKWQN
name: test-scenario-dependency-blocking-relati
description: 'Test Scenario: Dependency and Blocking Relationships'
parent-id: LCEWQN
task-type: task
priority: 2
labels:
- testing
- manual-test
- scenario
created-at: 2026-01-20T06:13:20.827488Z
updated-at: 2026-01-21T22:32:22.863815Z
---

# Test Scenario: Dependency and Blocking Relationships

See [Agent Manual Testing Guide](../../../docs/agent_manual_testing.md#LCBWQN)
for
general testing instructions.

## Objective

Verify that dependency management (blocking/blocked-by) works correctly,
including bidirectional consistency, ready queue computation, and cycle
detection.

## Setup

```bash
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"
git init
git config user.email "test@example.com"
git config user.name "Test Agent"
mkdir -p .lattice

# Create root
mkdir -p project
lat create project/project.md "Test Project"

# Create a chain of tasks
lat create project/ "Task A - Foundation" -t task -p 1
A_ID=$(ls project/tasks/*.md | tail -1 | xargs grep "lattice-id:" | cut -d' ' -f2)

lat create project/ "Task B - Depends on A" -t task -p 2
B_ID=$(ls project/tasks/*.md | tail -1 | xargs grep "lattice-id:" | cut -d' ' -f2)

lat create project/ "Task C - Depends on B" -t task -p 2
C_ID=$(ls project/tasks/*.md | tail -1 | xargs grep "lattice-id:" | cut -d' ' -f2)

lat create project/ "Task D - Independent" -t task -p 3
D_ID=$(ls project/tasks/*.md | tail -1 | xargs grep "lattice-id:" | cut -d' ' -f2)

git add .
git commit -m "Initial tasks"
```

## Test Sequence

### Part 1: Basic Dependency Addition

**Step 1.1**: Add dependency: B depends on A.

```bash
lat dep add $B_ID $A_ID
```

**Verify**:

- Task B's frontmatter has `blocked-by: [$A_ID]`
- Task A's frontmatter has `blocking: [$B_ID]`
- Bidirectional consistency maintained

**Step 1.2**: Verify blocked state.

```bash
lat show $B_ID
```

**Verify**:

- State shows as "blocked"
- "Depends on" section lists Task A

**Step 1.3**: Check ready queue.

```bash
lat ready
```

**Verify**:

- Task A IS in ready list (no blockers)
- Task B is NOT in ready list (blocked)
- Task C is still ready (no dependencies yet)
- Task D is ready

### Part 2: Chain Dependencies

**Step 2.1**: Add C depends on B.

```bash
lat dep add $C_ID $B_ID
```

**Verify**:

- Task C's frontmatter has `blocked-by: [$B_ID]`
- Task B's frontmatter has `blocking: [$C_ID] ` (in addition to being blocked-by
  A)

**Step 2.2**: Check ready queue with chain.

```bash
lat ready
```

**Verify**:

- Only Task A and Task D are ready
- B and C are blocked

**Step 2.3**: View blocked tasks.

```bash
lat blocked
```

**Verify**:

- Shows B and C
- Does NOT show A or D

**Step 2.4**: View blocked with --show-blockers.

```bash
lat blocked --show-blockers
```

**Verify**:

- Shows what's blocking each task
- B blocked by A
- C blocked by B

### Part 3: Dependency Tree

**Step 3.1**: View dependency tree.

```bash
lat dep tree $C_ID
```

**Verify**:

- Shows C depends on B depends on A
- Tree structure visible

**Step 3.2**: View dependency tree with JSON.

```bash
lat dep tree $C_ID --json
```

**Verify**:

- Valid JSON output
- Contains dependency chain

### Part 4: Unblocking via Close

**Step 4.1**: Close Task A to unblock B.

```bash
lat close $A_ID
```

**Verify**:

- Task A moved to `.closed/`

**Step 4.2**: Check if B is now ready.

```bash
lat ready
```

**Verify**:

- Task B IS now in ready list (A is closed)
- Task C is still blocked (B is not closed)

```bash
lat show $B_ID
```

**Verify**:

- State shows as "open" (not blocked)
- "Depends on" shows A as closed

**Step 4.3**: Close B to unblock C.

```bash
lat close $B_ID
lat ready
```

**Verify**:

- Task C is now ready

### Part 5: Dependency Removal

**Step 5.1**: Reopen tasks for more testing.

```bash
lat reopen $A_ID
lat reopen $B_ID
```

**Step 5.2**: Remove dependency.

```bash
lat dep remove $B_ID $A_ID
```

**Verify**:

- B's `blocked-by` no longer contains A
- A's `blocking` no longer contains B
- B is now ready (assuming no other blockers)

**Step 5.3**: Verify removal with JSON.

```bash
lat dep remove $C_ID $B_ID --json
```

**Verify**:

- JSON output includes `became_ready: true` (since C's only blocker removed)

### Part 6: Circular Dependency Detection

**Step 6.1**: Attempt to create circular dependency.

```bash
# Recreate chain: B depends on A
lat dep add $B_ID $A_ID
lat dep add $C_ID $B_ID

# Now try: A depends on C (would create cycle)
lat dep add $A_ID $C_ID
```

**Verify**:

- Command should fail with E006 (circular blocking)
- Clear error message about the cycle
- No changes made to any documents

**Step 6.2**: Verify no partial changes.

```bash
lat show $A_ID --json | grep blocked-by
```

**Verify**:

- A has no `blocked-by` (cycle not created)

### Part 7: discovered-from Links

**Step 7.1**: Create task with discovered-from.

```bash
lat create project/ "Discovered issue" -t bug --deps discovered-from:$A_ID
DISC_ID=$(ls project/tasks/*.md | tail -1 | xargs grep "lattice-id:" | cut -d' ' -f2)
```

**Verify**:

- Task has `discovered-from: [$A_ID]`
- NOT blocked (discovered-from is soft link)

**Step 7.2**: Query discovered tasks.

```bash
lat list --discovered-from $A_ID
```

**Verify**:

- Shows the discovered issue task

### Part 8: Multiple Blockers

**Step 8.1**: Add multiple blockers to one task.

```bash
lat create project/ "Blocked by many" -t task
MULTI_ID=$(ls project/tasks/*.md | tail -1 | xargs grep "lattice-id:" | cut -d' ' -f2)

lat dep add $MULTI_ID $A_ID
lat dep add $MULTI_ID $D_ID
```

**Verify**:

- Task has `blocked-by: [$A_ID, $D_ID]`
- Task is blocked

**Step 8.2**: Close one blocker.

```bash
lat close $A_ID
lat show $MULTI_ID
```

**Verify**:

- Task still blocked (D is still open)

**Step 8.3**: Close second blocker.

```bash
lat close $D_ID
lat ready
```

**Verify**:

- Task is now ready (all blockers closed)

### Part 9: Edge Cases

**Step 9.1**: Add dependency to self.

```bash
lat reopen $A_ID
lat dep add $A_ID $A_ID
```

**Verify**:

- Should fail (self-dependency is cycle)
- Clear error message

**Step 9.2**: Add dependency to non-existent task.

```bash
lat dep add $A_ID LNONEXISTENT
```

**Verify**:

- Should fail with not found error
- Exit code 4

**Step 9.3**: Remove non-existent dependency.

```bash
lat dep remove $A_ID $D_ID  # A doesn't depend on D
```

**Verify**:

- Should handle gracefully (no-op or clear message)

## Cleanup

```bash
cd /
rm -rf "$TEST_DIR"
```

## Expected Issues to Report

1. Bidirectional consistency not maintained (blocking/blocked-by mismatch)
2. Circular dependency not detected
3. Blocked tasks appearing in ready queue
4. Tasks not becoming ready when all blockers closed
5. discovered-from incorrectly blocking tasks
6. lat dep remove not updating both sides
7. Any panics in dependency operations
