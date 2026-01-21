---
lattice-id: LCOWQN
name: test-scenario-linter-validation-fixes
description: 'Test Scenario: Linter Validation and Fixes'
parent-id: LCEWQN
task-type: task
priority: 2
labels:
- testing
- manual-test
- scenario
created-at: 2026-01-20T06:15:47.675663Z
updated-at: 2026-01-21T22:32:22.886294Z
---

# Test Scenario: Linter Validation and Fixes

See [Agent Manual Testing Guide](../../../docs/agent_manual_testing.md#LCBWQN)
for
general testing instructions.

## Objective

Verify that `lat check` correctly detects all error and warning conditions, and
that `lat fmt` correctly formats and fixes documents.

## Setup

```bash
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"
git init
git config user.email "test@example.com"
git config user.name "Test Agent"
mkdir -p .lattice project/tasks project/docs

# Create a valid root document
lat create project/project.md "Test Project Root"
ROOT_ID=$(grep "lattice-id:" project/project.md | cut -d' ' -f2)

git add .
git commit -m "Initial setup"
```

## Test Sequence

### Part 1: Error Detection (E-codes)

#### E001: Duplicate Lattice ID

**Step 1.1**: Create duplicate ID scenario.

```bash
cat > project/docs/doc1.md << 'EOF'
---
lattice-id: LDUPLIC
name: doc-one
description: First document
---
Content.
EOF

cat > project/docs/doc2.md << 'EOF'
---
lattice-id: LDUPLIC
name: doc-two
description: Second document with same ID
---
Content.
EOF
```

**Step 1.2**: Check detects duplicate.

```bash
lat check
```

**Verify**:

- E001 error reported
- Both file paths shown
- Exit code 2

#### E002: Missing Reference Target

**Step 1.3**: Create broken link.

```bash
cat > project/docs/broken_link.md << 'EOF'
---
lattice-id: LBROKEN
name: broken-link
description: Has broken link
---

See [nonexistent](LNOTREAL) for details.
EOF
```

```bash
lat check
```

**Verify**:

- E002 error for missing target LNOTREAL
- Shows file and line number

#### E003: Invalid Frontmatter Key

**Step 1.4**: Create doc with typo in key.

```bash
cat > project/docs/bad_key.md << 'EOF'
---
lattice-id: LBADKEY
name: bad-key
description: Has bad key
priortiy: 2
---

Content.
EOF
```

```bash
lat check
```

**Verify**:

- E003 error for 'priortiy'
- Suggests 'priority'

#### E004: Missing Required Field (Task)

**Step 1.5**: Create task without priority.

```bash
cat > project/tasks/no_priority.md << 'EOF'
---
lattice-id: LNOPRIO
name: no-priority
description: Task without priority
task-type: bug
---

Missing priority.
EOF
```

```bash
lat check
```

**Verify**:

- E004 error: task missing priority

#### E005: Invalid Field Value

**Step 1.6**: Create doc with invalid priority.

```bash
cat > project/tasks/bad_priority.md << 'EOF'
---
lattice-id: LBADPRI
name: bad-priority
description: Invalid priority value
task-type: task
priority: 99
---

Bad priority.
EOF
```

```bash
lat check
```

**Verify**:

- E005 error: priority 99 invalid (allowed 0-4)

#### E006: Circular Blocking

**Step 1.7**: Create circular dependency.

```bash
cat > project/tasks/task_a.md << 'EOF'
---
lattice-id: LTASKA1
name: task-a
description: Task A
task-type: task
priority: 2
blocked-by: [LTASKB1]
---
EOF

cat > project/tasks/task_b.md << 'EOF'
---
lattice-id: LTASKB1
name: task-b
description: Task B
task-type: task
priority: 2
blocked-by: [LTASKA1]
---
EOF
```

```bash
lat check
```

**Verify**:

- E006 error: circular blocking dependency

#### E007: Invalid ID Format

**Step 1.8**: Create doc with malformed ID.

```bash
cat > project/docs/bad_id.md << 'EOF'
---
lattice-id: INVALID
name: bad-id
description: Malformed ID
---
EOF
```

```bash
lat check
```

**Verify**:

- E007 error: malformed lattice-id

#### E008: Name-Filename Mismatch

**Step 1.9**: Create doc with mismatched name.

```bash
cat > project/docs/my_document.md << 'EOF'
---
lattice-id: LMISMAT
name: wrong-name
description: Name doesn't match filename
---
EOF
```

```bash
lat check
```

**Verify**:

- E008 error: name should be 'my-document'

### Part 2: Warning Detection (W-codes)

**Step 2.1**: Clean up errors first.

```bash
rm project/docs/doc2.md project/docs/broken_link.md project/docs/bad_key.md
rm project/tasks/no_priority.md project/tasks/bad_priority.md
rm project/tasks/task_a.md project/tasks/task_b.md
rm project/docs/bad_id.md project/docs/my_document.md
```

#### W001: Document Too Large

**Step 2.2**: Create large document.

```bash
{
echo '---'
echo 'lattice-id: LLARGE1'
echo 'name: large-doc'
echo 'description: Very large document'
echo '---'
for i in $(seq 1 600); do
    echo "Line $i of content"
done
} > project/docs/large_doc.md
```

```bash
lat check
```

**Verify**:

- W001 warning: 600+ lines (recommended max 500)

#### W002/W003: Name/Description Too Long

**Step 2.3**: Create doc with long name.

```bash
LONG_NAME=$(printf 'a%.0s' {1..70})
cat > project/docs/long_name.md << EOF
---
lattice-id: LLONGN1
name: $LONG_NAME
description: Has very long name
---
EOF
```

```bash
lat check
```

**Verify**:

- W002 warning: name too long

#### W017: Document Not in Standard Location

**Step 2.4**: Create doc outside tasks/ or docs/.

```bash
cat > project/loose_doc.md << 'EOF'
---
lattice-id: LLOOSE1
name: loose-doc
description: Not in standard location
---
EOF
```

```bash
lat check
```

**Verify**:

- W017 warning: not in tasks/ or docs/ directory

#### W018: Task in docs/ Directory

**Step 2.5**: Create task in docs/.

```bash
cat > project/docs/misplaced_task.md << 'EOF'
---
lattice-id: LMISPLA
name: misplaced-task
description: Task in docs directory
task-type: bug
priority: 2
---
EOF
```

```bash
lat check
```

**Verify**:

- W018 warning: task in docs/ directory

### Part 3: Auto-Fix with lat check --fix

**Step 3.1**: Create fixable issues.

```bash
cat > project/docs/fixable.md << 'EOF'
---
lattice-id: LFIXA01
name: wrong-name
description: Has fixable name mismatch
---

Content.
EOF
```

```bash
lat check --fix
```

**Verify**:

- E008 (name mismatch) fixed automatically
- name field updated to 'fixable'

### Part 4: lat fmt Formatting

**Step 4.1**: Create poorly formatted document.

```bash
cat > project/docs/messy.md << 'EOF'
---
lattice-id: LMESSY1
name: messy
description: Poorly formatted
---


# Header without blank line after
Content right after header.


Multiple blank lines above.

*   Wrong list marker
+   Another wrong marker

Line with trailing spaces.

No final newline
EOF
```

**Step 4.2**: Run fmt.

```bash
lat fmt
```

**Step 4.3**: Verify formatting.

```bash
cat project/docs/messy.md
```

**Verify**:

- Single blank line between sections
- Consistent list markers (-)
- Trailing whitespace removed
- Final newline added
- Blank lines around headers

### Part 5: Link Normalization with fmt

**Step 5.1**: Create doc with shorthand links.

```bash
cat > project/docs/links_test.md << 'EOF'
---
lattice-id: LLINKS1
name: links-test
description: Test link normalization
---

# Links

See the [root document]($ROOT_ID) for context.

Also check [messy doc](messy.md) nearby.
EOF
```

Replace placeholder:
```bash
sed -i '' "s/\$ROOT_ID/$ROOT_ID/" project/docs/links_test.md 2>/dev/null || \
sed -i "s/\$ROOT_ID/$ROOT_ID/" project/docs/links_test.md
```

**Step 5.2**: Run fmt.

```bash
lat fmt
```

**Step 5.3**: Verify link expansion.

```bash
cat project/docs/links_test.md
```

**Verify**:

- First link expanded to include path: `../project.md#$ROOT_ID`
- Second link has fragment added: `messy.md#LMESSY1`

### Part 6: Check Modes

**Step 6.1**: Errors only mode.

```bash
lat check --errors-only
```

**Verify**:

- Only shows E-codes, not W-codes

**Step 6.2**: Path-scoped check.

```bash
lat check --path project/docs/
```

**Verify**:

- Only checks docs/, not tasks/

**Step 6.3**: Staged-only check.

```bash
git add project/docs/messy.md
lat check --staged-only
```

**Verify**:

- Only checks staged files

### Part 7: JSON Output

**Step 7.1**: Check with JSON.

```bash
lat check --json
```

**Verify**:

- Valid JSON output
- Contains `documents_checked`, `errors`, `warnings`, `summary`

## Cleanup

```bash
cd /
rm -rf "$TEST_DIR"
```

## Expected Issues to Report

1. Error codes not detected (E001-E008, E011-E013)
2. Warning codes not detected (W001-W020)
3. --fix not correcting fixable issues
4. lat fmt not normalizing formatting
5. lat fmt not expanding links
6. JSON output invalid or incomplete
7. Wrong exit codes
8. Any panics in validation
