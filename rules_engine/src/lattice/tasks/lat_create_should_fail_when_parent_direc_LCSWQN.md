---
lattice-id: LCSWQN
name: lat-create-should-fail-when-parent-direc
description: lat create should fail when parent directory does not exist
parent-id: LCEWQN
task-type: bug
priority: 2
labels:
- testing
- create-command
created-at: 2026-01-20T06:49:16.457848Z
updated-at: 2026-01-20T06:49:16.457848Z
---

## Context
Test scenario: LCCWQN (Document Creation and Basic Operations)
Step 6.3: Attempt to create document in non-existent directory

## Steps to Reproduce
1. Create temp directory and initialize git
2. Initialize Lattice with `.lattice/` directory
3. Run: `lat create nonexistent/ "Some task" -t task`

## Expected Behavior
- Command should fail with appropriate error message
- Exit code should be 3 (user error)
- The `nonexistent/` directory should NOT be created

## Actual Behavior
- Command succeeds (exit code 0)
- Directory `nonexistent/tasks/` is created automatically
- Task file is created at `nonexistent/tasks/some_task.md`

## Command Output
```
$ lat create nonexistent/ "Some task" -t task; echo "Exit code: $?"
LB2WQN nonexistent/tasks/some_task.md
Exit code: 0
```

## Impact
This behavior could lead to unintentional directory creation and document placement when users mistype paths. The test specification expects this to be an error condition.