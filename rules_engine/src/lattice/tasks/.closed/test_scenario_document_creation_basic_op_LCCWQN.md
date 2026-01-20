---
lattice-id: LCCWQN
name: test-scenario-document-creation-basic-op
description: 'Test Scenario: Document Creation and Basic Operations'
task-type: task
priority: 2
labels:
- testing
- manual-test
- scenario
created-at: 2026-01-20T06:13:20.569307Z
updated-at: 2026-01-20T06:13:20.569307Z
---

# Test Scenario: Document Creation and Basic Operations

See [Agent Manual Testing Guide](../../docs/agent_manual_testing.md#LCBWQN) for
general testing instructions.

## Objective

Verify that document creation commands work correctly for all document types,
auto-placement rules are followed, and basic viewing operations function.

## Setup

```bash
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"
git init
git config user.email "test@example.com"
git config user.name "Test Agent"
mkdir -p .lattice
```

## Test Sequence

### Part 1: Root Document Creation

**Step 1.1**: Create a root document for the `api` directory.

```bash
mkdir -p api
lat create api/api.md "API System Root"
```

**Verify**:
- File exists at `api/api.md`
- Frontmatter has `lattice-id`, `name: api`, `description: API System Root`
- No `task-type` or `priority` fields (this is a knowledge base document)
- No `parent-id` (root documents have no parent)

**Step 1.2**: View the root document.

```bash
lat show <id-from-step-1.1>
```

**Verify**:
- Output shows `<id>: api - API System Root`
- Type shows as `[doc]` not a task type

### Part 2: Task Creation with Auto-Placement

**Step 2.1**: Create a bug task under the api directory.

```bash
lat create api/ "Fix authentication token expiry" -t bug -p 1
```

**Verify**:
- File created in `api/tasks/` directory (NOT `api/`)
- Filename is auto-generated from description (e.g., `fix_authentication_token_expiry.md`)
- `task-type: bug` in frontmatter
- `priority: 1` in frontmatter
- `parent-id` references the api root document ID from Step 1.1
- `name` field matches filename pattern (hyphens instead of underscores)

**Step 2.2**: Create a feature task with labels.

```bash
lat create api/ "Add OAuth 2.0 support" -t feature -p 2 -l auth,security
```

**Verify**:
- File in `api/tasks/`
- `task-type: feature`
- `priority: 2`
- `labels: [auth, security]`
- Timestamps present (`created-at`, `updated-at`)

**Step 2.3**: Create a task type task.

```bash
lat create api/ "Write unit tests for auth module" -t task -p 3
```

**Verify**:
- `task-type: task`
- `priority: 3`

**Step 2.4**: Create a chore type task.

```bash
lat create api/ "Update dependencies" -t chore -p 4
```

**Verify**:
- `task-type: chore`
- `priority: 4` (backlog)

### Part 3: Knowledge Base Document Creation

**Step 3.1**: Create a knowledge base document (no -t flag).

```bash
lat create api/ "OAuth 2.0 implementation design"
```

**Verify**:
- File created in `api/docs/` directory (NOT `api/tasks/`)
- No `task-type` field
- No `priority` field
- Has `parent-id` referencing api root

### Part 4: Viewing Operations

**Step 4.1**: View task with `--short` format.

```bash
lat show <bug-task-id> --short
```

**Verify**:
- Single line output
- Format: `<id> [open] P1 bug: <name> - <description>`

**Step 4.2**: View task with `--json` format.

```bash
lat show <bug-task-id> --json
```

**Verify**:
- Valid JSON output
- Contains all expected fields: `id`, `name`, `description`, `state`, `priority`,
  `task_type`, `created_at`, `path`, `parent`
- `state` is `"open"`

**Step 4.3**: View multiple documents at once.

```bash
lat show <bug-id> <feature-id> <kb-doc-id>
```

**Verify**:
- All three documents displayed
- Separated by blank lines

**Step 4.4**: View with `--peek` format.

```bash
lat show <bug-task-id> --peek
```

**Verify**:
- Condensed two-line output
- Shows parent and counts

### Part 5: Listing Operations

**Step 5.1**: List all documents.

```bash
lat list
```

**Verify**:
- Shows all created documents (root, tasks, kb doc)
- Does not show closed tasks (none exist yet)

**Step 5.2**: List tasks only.

```bash
lat list --type bug
```

**Verify**:
- Only shows the bug task

**Step 5.3**: List by path prefix.

```bash
lat list --path api/tasks/
```

**Verify**:
- Only shows tasks, not the root or kb document

### Part 6: Edge Cases

**Step 6.1**: Create document with very long description.

```bash
lat create api/ "This is a very long description that should be truncated when generating the filename but preserved in full in the description field of the frontmatter" -t task
```

**Verify**:
- Filename is truncated to reasonable length (~40 chars)
- Full description preserved in frontmatter

**Step 6.2**: Create document with special characters in description.

```bash
lat create api/ "Fix bug: user's data isn't saved (issue #123)" -t bug
```

**Verify**:
- Filename sanitizes special characters
- Description preserved exactly

**Step 6.3**: Attempt to create document in non-existent directory.

```bash
lat create nonexistent/ "Some task" -t task
```

**Verify**:
- Command fails with appropriate error
- Exit code is 3 (user error)

**Step 6.4**: Commit and verify git tracking.

```bash
git add .
git commit -m "Initial documents"
git status
```

**Verify**:
- All `.md` files are committed
- `.lattice/` directory contents are tracked appropriately

### Part 7: Stats Command

**Step 7.1**: View project statistics.

```bash
lat stats
```

**Verify**:
- Shows document counts by type
- Shows counts by priority
- Shows counts by state (all should be "open")

## Cleanup

```bash
cd /
rm -rf "$TEST_DIR"
```

## Expected Issues to Report

Report any of the following as bugs using the `lattice_create_task()` MCP tool with
directory `rules_engine/src/lattice/tasks/qa/` (the full path including `/qa/` is required):

1. Files created in wrong directory (tasks in docs/, docs in tasks/)
2. Missing or incorrect frontmatter fields
3. Filename generation issues
4. Parent-id not set correctly
5. Labels not saved
6. Any panics or exit code 1
7. Invalid JSON output
8. Missing documents in list output
