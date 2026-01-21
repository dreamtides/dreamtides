---
lattice-id: LCBWQN
name: agent-manual-testing
description: Guide for AI agents performing manual testing of the Lattice system
parent-id: LCEWQN
labels:
- testing
- documentation
created-at: 2026-01-20T06:11:03.923739Z
updated-at: 2026-01-21T22:31:38.503909Z
---

# Agent Manual Testing Guide

This document provides instructions for AI agents executing manual test
scenarios
for the Lattice system.

## Test Environment Setup

Each test sequence MUST be executed in a hermetic environment:

```bash
# Create a unique temp directory for this test run
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"

# Initialize a git repository (required for Lattice)
git init
git config user.email "test@example.com"
git config user.name "Test Agent"

# Initialize Lattice (creates .lattice/ directory)
mkdir -p .lattice

# Verify lat is available
lat --version
```

**Critical**: Each test runs in isolation. Do not modify the main dreamtides
repository. All operations occur in the temporary directory.

## Test Execution Protocol

### Before Each Test Sequence

1. Create a fresh temp directory as described above
2. Follow the specific setup instructions in the test scenario
3. Note the test scenario ID for bug reporting

### During Test Execution

1. Execute commands exactly as specified
2. Verify outputs match expected behavior
3. Note any deviations, errors, or unexpected outputs
4. Continue testing unless a blocking error occurs

### Error Classification

**Blocking Errors** (abort test):

- Panic/crash in `lat` command
- Index corruption that prevents further operations
- System error (exit code 1)

**Non-Blocking Errors** (continue and report):

- Unexpected output format
- Missing expected warning
- Incorrect state after operation
- Performance issues

### Bug Reporting

Report all issues using the `lattice_create_task()` MCP tool:

**Directory**: `rules_engine/src/lattice/tasks/qa/`

> **IMPORTANT**: The directory parameter must be exactly
> `rules_engine/src/lattice/tasks/qa/`
> (including the `/qa/` suffix). Do NOT use `rules_engine/src/lattice/` or
> `rules_engine/src/lattice/tasks/` alone.

**Required fields**:

- `description`: Brief summary of the issue
- `task_type`: Usually `bug`
- `priority`: 1 for crashes/data loss, 2 for functional bugs, 3 for cosmetic
- `body`: Must include:
  - Test scenario ID (e.g., "Scenario: LXXXXXX")
  - Steps to reproduce
  - Expected behavior
  - Actual behavior
  - Full command output if relevant

**Example bug report body**:
```markdown
## Context
Test scenario: LXXXXXX (Document Creation)
Step 5: Create task with labels

## Steps to Reproduce
1. Created temp directory and initialized git
2. Ran: `lat create api/ "Fix bug" -t bug -l urgent,security`
3. Ran: `lat show <id> --json`

## Expected Behavior
JSON output should include `"labels": ["urgent", "security"]`

## Actual Behavior
JSON output shows `"labels": []` - labels not saved

## Command Output
[paste full output here]
```

## Understanding Expected vs Unexpected Behavior

### Expected Errors (Not Bugs)

These are intentional error handling:

- `lat show LNONEXISTENT` → "Error: Document not found" (exit 4)
- `lat create` without required args → Usage error (exit 3)
- `lat check` finding broken links → Reports error E002 (exit 2)
- Invalid frontmatter in manually created file → Validation error

### System Errors (Bugs to Report)

These indicate implementation problems:

- Any Rust panic or stack trace
- Exit code 1 (system error)
- Index doesn't match filesystem after any `lat` operation
- Operation claims success but state is wrong
- Silent failures (no error but no effect)

## Command Reference

### Document Creation

```bash
# Task with auto-placement
lat create <parent>/ "<description>" -t <type> [-p <priority>] [-l <labels>]
# Creates: <parent>/tasks/<generated_filename>.md

# Knowledge base document
lat create <parent>/ "<description>"
# Creates: <parent>/docs/<generated_filename>.md

# Root document (explicit path)
lat create <path>/<dir>.md "<description>"
```

### Task Lifecycle

```bash
lat close <id>           # Move to .closed/
lat reopen <id>          # Move from .closed/
lat prune <path>         # Delete closed tasks
lat prune --all          # Delete all closed tasks
```

### Queries

```bash
lat show <id>            # Display document
lat list [options]       # Search/filter
lat ready                # Ready work
lat blocked              # Blocked tasks
lat search "<query>"     # Full-text search
```

### Validation

```bash
lat check                # Validate all documents
lat check --fix          # Auto-fix issues
lat doctor               # System health
lat doctor --fix         # Fix system issues
```

### Dependencies

```bash
lat dep add <task> <depends-on>    # Task depends on depends-on
lat dep remove <task> <depends-on>
lat dep tree <task>                # Show dependency tree
```

### Links

```bash
lat fmt                  # Normalize all links
lat links-from <id>      # Outgoing links
lat links-to <id>        # Incoming links (backlinks)
```

## Exit Codes

| Code | Meaning | Is Bug? |
|------|---------|---------|
| 0 | Success | No |
| 1 | System error | YES |
| 2 | Validation error | No |
| 3 | User error (bad args) | No |
| 4 | Not found | No |

## Verifying State

After operations, verify state using:

```bash
# Check document exists and has correct content
lat show <id> --json

# Verify index matches filesystem
lat doctor

# Check for validation errors
lat check

# List all documents
lat list --include-closed
```

## Cleanup

After test completion (success or failure):

```bash
cd /
rm -rf "$TEST_DIR"
```

Do not leave test artifacts on the filesystem.
