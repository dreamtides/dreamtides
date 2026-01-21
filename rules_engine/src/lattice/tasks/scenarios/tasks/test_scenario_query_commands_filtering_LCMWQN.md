---
lattice-id: LCMWQN
name: test-scenario-query-commands-filtering
description: 'Test Scenario: Query Commands and Filtering'
parent-id: LCEWQN
task-type: task
priority: 2
labels:
- testing
- manual-test
- scenario
created-at: 2026-01-20T06:15:47.313110Z
updated-at: 2026-01-21T22:32:22.896179Z
---

# Test Scenario: Query Commands and Filtering

See [Agent Manual Testing Guide](../../../docs/agent_manual_testing.md#LCBWQN)
for
general testing instructions.

## Objective

Verify that query commands (list, ready, blocked, search, stats, stale, changes)
work correctly with various filtering options.

## Setup

```bash
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"
git init
git config user.email "test@example.com"
git config user.name "Test Agent"
mkdir -p .lattice

# Create structure with multiple areas
for dir in auth api database; do
    mkdir -p $dir
    lat create $dir/$dir.md "$dir system root"
done

# Create variety of tasks for filtering tests
lat create auth/ "Fix login security vulnerability" -t bug -p 0 -l security,urgent
lat create auth/ "Add two-factor authentication" -t feature -p 1 -l security,auth
lat create auth/ "Update password hashing" -t task -p 2 -l security
lat create auth/ "Audit auth dependencies" -t chore -p 4 -l security

lat create api/ "Fix rate limiting bug" -t bug -p 1 -l performance
lat create api/ "Add GraphQL endpoint" -t feature -p 2 -l graphql
lat create api/ "Refactor REST handlers" -t task -p 3

lat create database/ "Fix deadlock issue" -t bug -p 0 -l performance,critical
lat create database/ "Add connection pooling" -t feature -p 2 -l performance

# Create some knowledge base docs
lat create auth/ "Security best practices guide"
lat create api/ "API versioning strategy"

# Close one task
FIRST_TASK=$(lat list --path auth/tasks --json 2>/dev/null | head -1)
# Get first auth task ID and close it
AUTH_TASK_ID=$(ls auth/tasks/*.md | head -1 | xargs grep "lattice-id:" | cut -d' ' -f2)
lat close $AUTH_TASK_ID

git add .
git commit -m "Initial test data"
```

## Test Sequence

### Part 1: Basic List Operations

**Step 1.1**: List all documents (default).

```bash
lat list
```

**Verify**:

- Shows all open tasks and KB documents
- Does NOT show closed tasks
- Shows ID, type/priority indicator, name, description

**Step 1.2**: Include closed tasks.

```bash
lat list --include-closed
```

**Verify**:

- Shows closed task
- Closed tasks have `/closed` indicator

**Step 1.3**: Show only closed tasks.

```bash
lat list --closed-only
```

**Verify**:

- Only shows the closed task

### Part 2: Type Filtering

**Step 2.1**: Filter by bug type.

```bash
lat list --type bug
```

**Verify**:

- Only shows bug tasks
- Features, tasks, chores not shown

**Step 2.2**: Filter by feature type.

```bash
lat list --type feature
```

**Verify**:

- Only shows feature tasks

**Step 2.3**: Combine type with path.

```bash
lat list --type bug --path auth/
```

**Verify**:

- Only shows auth bugs

### Part 3: Priority Filtering

**Step 3.1**: Filter by exact priority.

```bash
lat list --priority 0
```

**Verify**:

- Only shows P0 tasks

**Step 3.2**: Filter by priority range.

```bash
lat list --priority-min 0 --priority-max 1
```

**Verify**:

- Shows P0 and P1 tasks only

**Step 3.3**: Exclude backlog (P4).

```bash
lat list --priority-max 3
```

**Verify**:

- P4 tasks not shown

### Part 4: Label Filtering

**Step 4.1**: Filter by single label.

```bash
lat list --label security
```

**Verify**:

- Only shows documents with "security" label

**Step 4.2**: Filter by multiple labels (AND logic).

```bash
lat list --label security,urgent
```

**Verify**:

- Only shows documents with BOTH labels

**Step 4.3**: Filter by any label (OR logic).

```bash
lat list --label-any security,graphql
```

**Verify**:

- Shows documents with EITHER label

### Part 5: Path Filtering

**Step 5.1**: Filter by path prefix.

```bash
lat list --path auth/
```

**Verify**:

- Only shows documents under auth/

**Step 5.2**: Filter by nested path.

```bash
lat list --path auth/tasks/
```

**Verify**:

- Only shows tasks under auth/, not root or docs

### Part 6: Date Filtering

**Step 6.1**: Filter by creation date.

```bash
TODAY=$(date +%Y-%m-%d)
lat list --created-after $TODAY
```

**Verify**:

- Shows documents created today

**Step 6.2**: Filter by update date.

```bash
lat list --updated-after $TODAY
```

**Verify**:

- Shows recently updated documents

### Part 7: Output Formats

**Step 7.1**: JSON output.

```bash
lat list --json
```

**Verify**:

- Valid JSON array
- Each item has id, name, description, state, etc.

**Step 7.2**: Compact format.

```bash
lat list --format compact
```

**Verify**:

- Shorter output, just ID and name

**Step 7.3**: One-line format.

```bash
lat list --format oneline
```

**Verify**:

- One document per line

### Part 8: Sorting

**Step 8.1**: Sort by priority.

```bash
lat list --sort priority
```

**Verify**:

- P0 tasks first, then P1, etc.

**Step 8.2**: Sort by creation date.

```bash
lat list --sort created
```

**Verify**:

- Oldest first

**Step 8.3**: Reverse sort.

```bash
lat list --sort created --reverse
```

**Verify**:

- Newest first

### Part 9: Ready Command

**Step 9.1**: Basic ready.

```bash
lat ready
```

**Verify**:

- Shows open tasks with no blockers
- Does NOT show P4 (backlog) by default
- Does NOT show closed tasks

**Step 9.2**: Include backlog.

```bash
lat ready --include-backlog
```

**Verify**:

- P4 tasks now included

**Step 9.3**: Filter ready by priority.

```bash
lat ready --priority 0
```

**Verify**:

- Only P0 ready tasks

**Step 9.4**: Pretty format.

```bash
lat ready --pretty
```

**Verify**:

- Visual tree format
- Legend shown

**Step 9.5**: JSON format.

```bash
lat ready --json
```

**Verify**:

- Valid JSON array
- Contains body text for AI context

### Part 10: Blocked Command

**Step 10.1**: Add some blocking relationships.

```bash
# Get two task IDs
TASK1=$(ls api/tasks/*.md | head -1 | xargs grep "lattice-id:" | cut -d' ' -f2)
TASK2=$(ls api/tasks/*.md | tail -1 | xargs grep "lattice-id:" | cut -d' ' -f2)
lat dep add $TASK2 $TASK1
```

**Step 10.2**: List blocked tasks.

```bash
lat blocked
```

**Verify**:

- Shows TASK2 (blocked by TASK1)

**Step 10.3**: Show blockers.

```bash
lat blocked --show-blockers
```

**Verify**:

- Shows what's blocking each task

### Part 11: Search Command

**Step 11.1**: Basic full-text search.

```bash
lat search "security"
```

**Verify**:

- Shows documents containing "security"

**Step 11.2**: Search with path filter.

```bash
lat search "fix" --path auth/
```

**Verify**:

- Only searches in auth/

**Step 11.3**: Search with type filter.

```bash
lat search "bug" --type bug
```

**Verify**:

- Only searches bug tasks

### Part 12: Stats Command

**Step 12.1**: View overall stats.

```bash
lat stats
```

**Verify**:

- Shows document counts by type
- Shows counts by priority
- Shows counts by state

**Step 12.2**: Stats for specific path.

```bash
lat stats --path auth/
```

**Verify**:

- Stats scoped to auth/ only

### Part 13: Stale Command

**Step 13.1**: Find stale tasks (long threshold).

```bash
lat stale --days 0
```

**Verify**:

- Shows tasks updated more than 0 days ago (all of them)

**Step 13.2**: Find stale with reasonable threshold.

```bash
lat stale --days 30
```

**Verify**:

- No tasks shown (all created today)

### Part 14: Changes Command

**Step 14.1**: Show recent changes.

```bash
git log --oneline -1
COMMIT=$(git rev-parse HEAD~1 2>/dev/null || echo "HEAD")
lat changes --since $COMMIT
```

**Verify**:

- Shows documents changed since commit

### Part 15: Roots Only Filter

**Step 15.1**: List only root documents.

```bash
lat list --roots-only
```

**Verify**:

- Only shows auth.md, api.md, database.md
- Does NOT show tasks or docs

### Part 16: Limit and Pagination

**Step 16.1**: Limit results.

```bash
lat list --limit 3
```

**Verify**:

- Only shows 3 documents

**Step 16.2**: Combine limit with sort.

```bash
lat list --sort priority --limit 3
```

**Verify**:

- Shows top 3 by priority

## Cleanup

```bash
cd /
rm -rf "$TEST_DIR"
```

## Expected Issues to Report

1. Filter not working correctly (wrong documents returned)
2. Closed tasks appearing when they shouldn't
3. Labels not filtering properly (AND vs OR logic)
4. JSON output invalid or missing fields
5. Sort order incorrect
6. Search not finding expected content
7. Stats counts incorrect
8. Any panics in query operations
