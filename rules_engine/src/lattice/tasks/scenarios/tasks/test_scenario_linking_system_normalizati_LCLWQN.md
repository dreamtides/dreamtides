---
lattice-id: LCLWQN
name: test-scenario-linking-system-normalizati
description: 'Test Scenario: Linking System and Normalization'
parent-id: LCEWQN
task-type: task
priority: 2
labels:
- testing
- manual-test
- scenario
created-at: 2026-01-20T06:13:20.935399Z
updated-at: 2026-01-21T22:32:22.880087Z
---

# Test Scenario: Linking System and Normalization

See [Agent Manual Testing Guide](../../../docs/agent_manual_testing.md#LCBWQN)
for
general testing instructions.

## Objective

Verify that the linking system works correctly, including shorthand link
expansion, path normalization, link updates on document moves, and backlink
tracking.

## Setup

```bash
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"
git init
git config user.email "test@example.com"
git config user.name "Test Agent"
mkdir -p .lattice

# Create structure
mkdir -p docs/design docs/api tasks

# Create root documents
lat create docs/docs.md "Documentation Root"
lat create docs/design/design.md "Design Documents"
lat create docs/api/api.md "API Documentation"

# Create some documents with content
cat > docs/design/architecture.md << 'EOF'
---
lattice-id: LTEST01
name: architecture
description: System architecture overview
---

# Architecture

This document describes the system architecture.
EOF

cat > docs/api/endpoints.md << 'EOF'
---
lattice-id: LTEST02
name: endpoints
description: API endpoints reference
---

# API Endpoints

See the [architecture doc](LTEST01) for context.
EOF

cat > docs/design/data_model.md << 'EOF'
---
lattice-id: LTEST03
name: data-model
description: Data model design
---

# Data Model

References:
- Architecture: [link](LTEST01)
- API: [endpoints](../api/endpoints.md)
EOF

git add .
git commit -m "Initial structure"
```

## Test Sequence

### Part 1: Shorthand Link Expansion

**Step 1.1**: Run fmt to expand shorthand links.

```bash
lat fmt
```

**Verify in `docs/api/endpoints.md`**:

- Link changed from `[architecture doc](LTEST01)` to
  `[architecture doc](../design/architecture.md#LTEST01)`

**Verify in `docs/design/data_model.md`**:

- Link `[link](LTEST01)` expanded to include path and fragment
- Link `[endpoints](../api/endpoints.md)` now includes `#LTEST02` fragment

### Part 2: Link Validation

**Step 2.1**: Check for link issues.

```bash
lat check
```

**Verify**:

- No E002 errors (missing reference targets)
- No W010 warnings (stale link paths) after fmt
- No W010b warnings (missing fragments) after fmt

**Step 2.2**: Create document with broken link.

```bash
cat > docs/broken.md << 'EOF'
---
lattice-id: LTEST04
name: broken
description: Document with broken link
---

See [nonexistent](LNONEXIST) for details.
EOF
```

```bash
lat check
```

**Verify**:

- E002 error reported for broken link to LNONEXIST
- Shows file path and line number

### Part 3: Backlink Tracking

**Step 3.1**: Check what links TO architecture doc.

```bash
lat links-to LTEST01
```

**Verify**:

- Shows `docs/api/endpoints.md`
- Shows `docs/design/data_model.md`

**Step 3.2**: Check what architecture doc links FROM.

```bash
lat links-from LTEST01
```

**Verify**:

- Shows nothing or empty (architecture has no outgoing links in body)

**Step 3.3**: Check links from data_model.

```bash
lat links-from LTEST03
```

**Verify**:

- Shows LTEST01 (architecture)
- Shows LTEST02 (endpoints)

### Part 4: Link Updates on Move

**Step 4.1**: Move a document that others link to.

```bash
lat mv LTEST01 docs/new_location/architecture.md
```

**Verify**:

- File moved to `docs/new_location/architecture.md`
- In `docs/api/endpoints.md`: link updated to
  `../new_location/architecture.md#LTEST01`
- In `docs/design/data_model.md`: link updated to
  `../new_location/architecture.md#LTEST01`

**Step 4.2**: Verify links still resolve.

```bash
lat check
```

**Verify**:

- No E002 or W010 errors

**Step 4.3**: Verify backlinks still work.

```bash
lat links-to LTEST01
```

**Verify**:

- Still shows the linking documents

### Part 5: Link Updates on Close/Reopen

**Step 5.1**: Create a task that others reference.

```bash
lat create docs/ "Important task" -t task
TASK_ID=$(ls docs/tasks/*.md | tail -1 | xargs grep "lattice-id:" | cut -d' ' -f2)

# Add link to task in another document
cat >> docs/design/data_model.md << EOF

See also [the task]($TASK_ID).
EOF

lat fmt
git add .
git commit -m "Add task and link"
```

**Step 5.2**: Close the task.

```bash
lat close $TASK_ID
```

**Verify in `docs/design/data_model.md`**:

- Link path updated to include `.closed/` in path

**Step 5.3**: Reopen the task.

```bash
lat reopen $TASK_ID
```

**Verify**:

- Link path updated back to non-.closed path

### Part 6: Related Documents in Show

**Step 6.1**: View document with links.

```bash
lat show LTEST03
```

**Verify**:

- "Related" section shows LTEST01 and LTEST02
- Shows document titles/descriptions

**Step 6.2**: View with JSON.

```bash
lat show LTEST03 --json
```

**Verify**:

- `related` array contains the linked documents
- Each has `id`, `name`, `description`

### Part 7: Orphan Detection

**Step 7.1**: Create orphan document (no incoming links).

```bash
cat > docs/orphan.md << 'EOF'
---
lattice-id: LTEST05
name: orphan
description: Nobody links to me
---

Lonely document.
EOF

git add docs/orphan.md
```

**Step 7.2**: Find orphans.

```bash
lat orphans
```

**Verify**:

- Shows LTEST05 (orphan.md)

**Step 7.3**: Exclude roots from orphan list.

```bash
lat orphans --exclude-roots
```

**Verify**:

- Root documents not listed as orphans

### Part 8: Path Finding

**Step 8.1**: Find path between documents.

```bash
lat path LTEST02 LTEST01
```

**Verify**:

- Shows path: LTEST02 → LTEST01 (direct link)

**Step 8.2**: Find longer path.

```bash
# LTEST03 links to LTEST02 which links to LTEST01
lat path LTEST03 LTEST01
```

**Verify**:

- Shows direct or indirect path

### Part 9: Impact Analysis

**Step 9.1**: Check impact of changing architecture doc.

```bash
lat impact LTEST01
```

**Verify**:

- Shows documents that reference LTEST01
- Helps understand change impact

### Part 10: Edge Cases

**Step 10.1**: Self-reference detection.

```bash
cat > docs/selfref.md << 'EOF'
---
lattice-id: LTEST06
name: selfref
description: References itself
---

See [myself](LTEST06) for more.
EOF

lat check
```

**Verify**:

- W008 warning about self-reference

**Step 10.2**: Link with correct path but wrong fragment.

```bash
cat > docs/wrongfrag.md << 'EOF'
---
lattice-id: LTEST07
name: wrongfrag
description: Wrong fragment
---

See [arch](../new_location/architecture.md#LWRONG).
EOF

lat check
```

**Verify**:

- Should detect fragment doesn't match file's ID (or path doesn't match ID)

**Step 10.3**: External URL (not a Lattice link).

```bash
cat > docs/external.md << 'EOF'
---
lattice-id: LTEST08
name: external
description: Has external link
---

See [Google](https://google.com) for more.
EOF

lat check
```

**Verify**:

- External URL not treated as Lattice link
- No errors about it

## Cleanup

```bash
cd /
rm -rf "$TEST_DIR"
```

## Expected Issues to Report

1. Shorthand links not expanded by `lat fmt`
2. Link paths not updated after `lat mv`
3. Link paths not updated after `lat close`/`lat reopen`
4. Backlinks not tracked correctly
5. Orphan detection missing documents
6. Impact analysis missing references
7. Self-reference not warned
8. Any panics in link operations
