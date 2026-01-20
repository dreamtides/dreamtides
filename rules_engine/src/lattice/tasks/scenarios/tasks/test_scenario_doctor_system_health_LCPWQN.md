---
lattice-id: LCPWQN
name: test-scenario-doctor-system-health
description: 'Test Scenario: Doctor and System Health'
task-type: task
priority: 2
labels:
- testing
- manual-test
- scenario
created-at: 2026-01-20T06:15:47.815772Z
updated-at: 2026-01-20T06:15:47.815772Z
---

# Test Scenario: Doctor and System Health

See [Agent Manual Testing Guide](../../docs/agent_manual_testing.md#LCBWQN) for
general testing instructions.

## Objective

Verify that `lat doctor` correctly diagnoses system health issues and can fix
them when appropriate.

## Setup

```bash
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"
git init
git config user.email "test@example.com"
git config user.name "Test Agent"

# Create initial structure
mkdir -p project/tasks project/docs .lattice

lat create project/project.md "Test Project"
ROOT_ID=$(grep "lattice-id:" project/project.md | cut -d' ' -f2)

lat create project/ "Test task" -t task
TASK_ID=$(ls project/tasks/*.md | head -1 | xargs grep "lattice-id:" | cut -d' ' -f2)

git add .
git commit -m "Initial setup"
```

## Test Sequence

### Part 1: Healthy System Check

**Step 1.1**: Run doctor on healthy system.

```bash
lat doctor
```

**Verify**:
- All checks pass (✓)
- Shows check categories: Core System, Index Integrity, Git Integration, Configuration
- Exit code 0

**Step 1.2**: JSON output.

```bash
lat doctor --json
```

**Verify**:
- Valid JSON with `checks` array and `summary`
- All statuses are "passed"

### Part 2: Index Integrity Issues

**Step 2.1**: Corrupt index by removing file.

```bash
# Delete a file without going through lat
rm project/tasks/*.md
```

**Step 2.2**: Run doctor.

```bash
lat doctor
```

**Verify**:
- Index Integrity check fails
- Shows indexed ID doesn't have corresponding file
- Suggests fix

**Step 2.3**: Fix with --fix.

```bash
lat doctor --fix
```

**Verify**:
- Index rebuilt
- No more errors

**Step 2.4**: Verify fix.

```bash
lat doctor
```

**Verify**:
- All checks pass now

### Part 3: Missing Index

**Step 3.1**: Delete index file.

```bash
rm .lattice/index.sqlite*
```

**Step 3.2**: Run doctor.

```bash
lat doctor
```

**Verify**:
- Core System check fails: index missing
- Suggests rebuild

**Step 3.3**: Fix index.

```bash
lat doctor --fix
```

**Verify**:
- Index rebuilt from filesystem
- Documents re-indexed

**Step 3.4**: Verify documents accessible.

```bash
lat show $ROOT_ID
```

**Verify**:
- Document still accessible after rebuild

### Part 4: WAL Corruption Simulation

**Step 4.1**: Create fake WAL files.

```bash
echo "corrupt" > .lattice/index.sqlite-wal
echo "corrupt" > .lattice/index.sqlite-shm
```

**Step 4.2**: Run doctor.

```bash
lat doctor
```

**Verify**:
- WAL health check warns or errors
- Suggests cleanup

**Step 4.3**: Fix WAL issues.

```bash
lat doctor --fix
```

**Verify**:
- WAL files cleaned up
- Index functional

### Part 5: Git Integration Checks

**Step 5.1**: Check on valid git repo.

```bash
lat doctor
```

**Verify**:
- Git Integration check passes
- Shows repository valid

**Step 5.2**: Create merge conflict scenario.

```bash
git checkout -b feature
echo "change" >> project/project.md
git add .
git commit -m "Feature change"

git checkout master
echo "different" >> project/project.md
git add .
git commit -m "Master change"

git merge feature || true  # Will conflict
```

**Step 5.3**: Run doctor during conflict.

```bash
lat doctor
```

**Verify**:
- Git Integration shows warning about merge in progress
- Info level, not blocking error

**Step 5.4**: Resolve conflict and continue.

```bash
git checkout --ours project/project.md
git add .
git commit -m "Resolved"
```

### Part 6: Configuration Checks

**Step 6.1**: Create invalid config.

```bash
echo "invalid: yaml: content:" > .lattice/config.toml
```

**Step 6.2**: Run doctor.

```bash
lat doctor
```

**Verify**:
- Configuration check fails
- Shows parse error

**Step 6.3**: Fix config.

```bash
rm .lattice/config.toml
lat doctor
```

**Verify**:
- Config check passes (no config = valid)

### Part 7: Closed State Consistency

**Step 7.1**: Create task and close it.

```bash
lat create project/ "Another task" -t task
TASK2_ID=$(ls project/tasks/*.md | head -1 | xargs grep "lattice-id:" | cut -d' ' -f2)
lat close $TASK2_ID
```

**Step 7.2**: Manually move file without updating index.

```bash
# Move file back but don't use lat reopen
mv project/tasks/.closed/*.md project/tasks/
```

**Step 7.3**: Run doctor.

```bash
lat doctor
```

**Verify**:
- Closed State check warns: is_closed mismatch
- File path doesn't match indexed closed state

**Step 7.4**: Fix closed state.

```bash
lat doctor --fix
```

**Verify**:
- Index updated to match filesystem state

### Part 8: Deep Mode

**Step 8.1**: Run deep checks.

```bash
lat doctor --deep
```

**Verify**:
- Additional checks run
- Takes longer
- More thorough validation

### Part 9: Dry Run

**Step 9.1**: Create issue and dry-run fix.

```bash
rm .lattice/index.sqlite
lat doctor --fix --dry-run
```

**Verify**:
- Shows what would be fixed
- Index NOT actually rebuilt

**Step 9.2**: Verify index still missing.

```bash
ls .lattice/index.sqlite
```

**Verify**:
- File does not exist (dry-run didn't create it)

**Step 9.3**: Actually fix.

```bash
lat doctor --fix
```

### Part 10: Quiet Mode

**Step 10.1**: Run quiet.

```bash
lat doctor --quiet
```

**Verify**:
- Only warnings and errors shown
- Passed checks not listed

### Part 11: Check vs Doctor Distinction

**Step 11.1**: Create document content issue.

```bash
cat > project/docs/broken_link.md << 'EOF'
---
lattice-id: LBROK01
name: broken-link
description: Has broken link
---

See [nonexistent](LNOTREAL).
EOF
```

**Step 11.2**: Run both commands.

```bash
lat doctor
lat check
```

**Verify**:
- lat doctor: system healthy (broken link is content issue)
- lat check: E002 error (broken link detected)

### Part 12: Exit Codes

**Step 12.1**: Healthy system.

```bash
rm project/docs/broken_link.md
lat doctor
echo "Exit code: $?"
```

**Verify**: Exit code 0

**Step 12.2**: With warnings.

```bash
# Create condition that causes warning but not error
lat doctor
echo "Exit code: $?"
```

**Verify**: Exit code 0 or 3 depending on warnings

**Step 12.3**: With errors.

```bash
rm .lattice/index.sqlite
lat doctor
echo "Exit code: $?"
```

**Verify**: Exit code 2 (errors present)

## Cleanup

```bash
cd /
rm -rf "$TEST_DIR"
```

## Expected Issues to Report

1. Doctor not detecting missing index
2. Doctor not detecting filesystem-index mismatch
3. --fix not repairing issues
4. --dry-run actually making changes
5. WAL corruption not handled
6. Git state issues not reported
7. Config parse errors not caught
8. Wrong exit codes
9. Any panics in doctor operations
