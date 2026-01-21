---
lattice-id: LCNWQN
name: test-scenario-hierarchy-structure-comman
description: 'Test Scenario: Hierarchy and Structure Commands'
parent-id: LCEWQN
task-type: task
priority: 2
labels:
- testing
- manual-test
- scenario
created-at: 2026-01-20T06:15:47.486596Z
updated-at: 2026-01-21T22:32:22.875066Z
---

# Test Scenario: Hierarchy and Structure Commands

See [Agent Manual Testing Guide](../../../docs/agent_manual_testing.md#LCBWQN)
for
general testing instructions.

## Objective

Verify that hierarchy commands (tree, roots, children) and parent-id management
work correctly.

## Setup

```bash
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"
git init
git config user.email "test@example.com"
git config user.name "Test Agent"
mkdir -p .lattice

# Create nested hierarchy
mkdir -p project/backend/auth project/backend/api project/frontend/components

# Create root documents at each level
lat create project/project.md "Main Project"
PROJECT_ID=$(grep "lattice-id:" project/project.md | cut -d' ' -f2)

lat create project/backend/backend.md "Backend Services"
BACKEND_ID=$(grep "lattice-id:" project/backend/backend.md | cut -d' ' -f2)

lat create project/backend/auth/auth.md "Authentication Service"
AUTH_ID=$(grep "lattice-id:" project/backend/auth/auth.md | cut -d' ' -f2)

lat create project/backend/api/api.md "API Gateway"
API_ID=$(grep "lattice-id:" project/backend/api/api.md | cut -d' ' -f2)

lat create project/frontend/frontend.md "Frontend Application"
FRONTEND_ID=$(grep "lattice-id:" project/frontend/frontend.md | cut -d' ' -f2)

# Create tasks under various roots
lat create project/backend/auth/ "Fix login bug" -t bug -p 1
lat create project/backend/auth/ "Add OAuth" -t feature -p 2
lat create project/backend/api/ "Rate limiting" -t task -p 2
lat create project/frontend/ "Fix UI bug" -t bug -p 1

# Create KB docs
lat create project/backend/auth/ "Auth design document"
lat create project/backend/api/ "API specification"

git add .
git commit -m "Initial hierarchy"
```

## Test Sequence

### Part 1: Tree Command

**Step 1.1**: Display full tree.

```bash
lat tree
```

**Verify**:

- Shows hierarchical structure
- Displays all directories with documents
- Shows document counts or names

**Step 1.2**: Tree from specific path.

```bash
lat tree project/backend/
```

**Verify**:

- Only shows backend subtree
- Includes auth/ and api/

**Step 1.3**: Tree with depth limit.

```bash
lat tree --depth 1
```

**Verify**:

- Only shows one level deep

**Step 1.4**: Tree with counts.

```bash
lat tree --counts
```

**Verify**:

- Shows document counts per directory

**Step 1.5**: Tasks only tree.

```bash
lat tree --tasks-only
```

**Verify**:

- Only shows tasks/ directories
- Hides docs/ directories

**Step 1.6**: Docs only tree.

```bash
lat tree --docs-only
```

**Verify**:

- Only shows docs/ directories
- Hides tasks/ directories

### Part 2: Roots Command

**Step 2.1**: List all root documents.

```bash
lat roots
```

**Verify**:

- Shows project.md, backend.md, auth.md, api.md, frontend.md
- Shows child counts for each

**Step 2.2**: JSON output.

```bash
lat roots --json
```

**Verify**:

- Valid JSON array
- Each root has id, path, child_count

### Part 3: Children Command

**Step 3.1**: List children of a root.

```bash
lat children $AUTH_ID
```

**Verify**:

- Shows tasks and docs under auth/
- Does NOT show documents from other directories

**Step 3.2**: Children with recursive flag.

```bash
lat children $BACKEND_ID --recursive
```

**Verify**:

- Shows all documents under backend/
- Includes nested auth/ and api/ documents

**Step 3.3**: Children tasks only.

```bash
lat children $AUTH_ID --tasks
```

**Verify**:

- Only shows tasks, not KB docs

**Step 3.4**: Children docs only.

```bash
lat children $AUTH_ID --docs
```

**Verify**:

- Only shows KB docs, not tasks

### Part 4: Parent-ID Verification

**Step 4.1**: Verify parent-id in auth tasks.

```bash
lat show <auth-task-id> --json
```

**Verify**:

- `parent.id` matches $AUTH_ID
- Parent is the auth.md root document

**Step 4.2**: Verify parent-id in backend tasks.

```bash
lat show <api-task-id> --json
```

**Verify**:

- Parent is the api.md root document

**Step 4.3**: Format to update parent-ids.

```bash
# Manually remove parent-id from a file
FILE=$(ls project/backend/auth/tasks/*.md | head -1)
sed -i '' '/parent-id/d' "$FILE" 2>/dev/null || sed -i '/parent-id/d' "$FILE"

lat fmt
```

**Verify**:

- parent-id is restored by lat fmt

### Part 5: Move and Parent Update

**Step 5.1**: Move task to different directory.

```bash
TASK_ID=$(ls project/backend/auth/tasks/*.md | head -1 | xargs grep "lattice-id:" | cut -d' ' -f2)
lat mv $TASK_ID project/backend/api/tasks/moved_task.md
```

**Verify**:

- File moved to new location
- parent-id updated to API_ID
- name updated to "moved-task"

**Step 5.2**: Verify new parent.

```bash
lat show $TASK_ID --json
```

**Verify**:

- Parent now shows api.md root

### Part 6: Overview Command

**Step 6.1**: Repository overview.

```bash
lat overview
```

**Verify**:

- Shows critical documents
- Prioritizes root documents

**Step 6.2**: Contextual overview.

```bash
lat overview $AUTH_ID
```

**Verify**:

- Shows documents relevant to auth context

**Step 6.3**: Overview with limit.

```bash
lat overview --limit 5
```

**Verify**:

- Only shows 5 documents

**Step 6.4**: Overview JSON.

```bash
lat overview --json
```

**Verify**:

- Valid JSON array

### Part 7: Alternative Root Document Format

**Step 7.1**: Create directory with 00_ prefix root.

```bash
mkdir -p project/config
cat > project/config/00_overview.md << 'EOF'
---
lattice-id: LCONFIG
name: config-overview
description: Configuration overview
---

# Configuration

Main configuration document.
EOF

lat create project/config/ "Config setting 1" -t task
```

**Step 7.2**: Verify 00_ recognized as root.

```bash
lat roots
```

**Verify**:

- Shows 00_overview.md as a root

**Step 7.3**: Verify parent-id uses 00_ root.

```bash
lat show <config-task-id> --json
```

**Verify**:

- Parent is LCONFIG (the 00_overview.md)

### Part 8: Duplicate Root Detection

**Step 8.1**: Try to create conflicting roots.

```bash
mkdir -p project/conflict
lat create project/conflict/conflict.md "Standard root"
cat > project/conflict/00_alt.md << 'EOF'
---
lattice-id: LCONFLICT
name: alt-root
description: Alternative root
---

Conflicting root.
EOF
```

**Step 8.2**: Check detects the conflict.

```bash
lat check
```

**Verify**:

- E013 error: duplicate root documents in same directory

### Part 9: Edge Cases

**Step 9.1**: Document in directory without root.

```bash
mkdir -p project/orphan_dir/tasks
cat > project/orphan_dir/tasks/orphan_task.md << 'EOF'
---
lattice-id: LORPHAN
name: orphan-task
description: Task without root
task-type: task
priority: 2
---

Task in directory without root document.
EOF
```

**Step 9.2**: Verify no parent-id.

```bash
lat show LORPHAN --json
```

**Verify**:

- No parent (directory has no root document)

**Step 9.3**: Children of non-root document.

```bash
lat children LORPHAN
```

**Verify**:

- Should handle gracefully (not a root document)

## Cleanup

```bash
cd /
rm -rf "$TEST_DIR"
```

## Expected Issues to Report

1. Tree not showing correct hierarchy
2. Roots missing some root documents
3. Children showing wrong documents
4. Parent-id not updated after move
5. Parent-id not populated by lat fmt
6. 00_ prefix not recognized as root
7. Duplicate root not detected (E013)
8. Any panics in hierarchy operations
