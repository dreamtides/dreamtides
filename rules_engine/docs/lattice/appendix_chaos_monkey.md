# Appendix: Chaos Monkey

This appendix documents the automated fuzz testing system. For the broader
testing strategy including unit tests and fakes, see
[Appendix: Testing Strategy](appendix_testing_strategy.md).

## Purpose

The `lat chaosmonkey` command performs automated fuzz testing to discover
edge cases and interaction bugs. It executes random sequences of operations
until a system error occurs, then reports the failure for investigation.

## Execution Model

### Operation Loop

```
repeat until system_error or max_ops reached:
    1. Select random operation type
    2. Generate random parameters
    3. Execute operation
    4. Validate postconditions
    5. Log operation and result
```

### Termination Conditions

- **System error**: Exit code 1 from any operation
- **Postcondition failure**: Expected state doesn't match actual
- **Maximum operations**: `--max-ops N` limit reached
- **Manual interrupt**: Ctrl+C

## Operation Types

### Document Operations

**create_document**: Create a new knowledge base document
- Random path (within existing directories)
- Random name (valid format)
- Random description (variable length)
- Random body content (0-600 lines)

**create_issue**: Create a new issue document
- Random path (within existing directories)
- Random type (bug/feature/task/chore)
- Random status (open/blocked)
- Random priority (0-4)
- Random labels (0-5)

**modify_document**: Change existing document
- Random field selection
- Random new value (valid for field)
- Random body edits (insert/delete/modify lines)

**delete_document**: Remove a document
- Random selection from existing documents
- Tracks broken references for validation

**move_document**: Relocate document
- Random source selection
- Random target directory
- Updates path, preserves ID

### Link Operations

**add_link**: Insert link into document body
- Random source document
- Random target (document or section)
- Random position in body

**remove_link**: Delete link from document
- Random source document
- Random link selection

**add_section_id**: Annotate a header
- Random document with headers
- Random untagged header

### Metadata Operations

**add_label**: Add label to document
- Random document
- Random label (existing or new)

**remove_label**: Remove label
- Random document with labels
- Random label selection

**change_status**: Modify issue status
- Random issue document
- Random valid transition

**change_priority**: Modify issue priority
- Random issue document
- Random priority (0-4)

### Git Operations

**commit**: Commit current changes
- Random commit message
- All staged files

**branch**: Create new branch
- Random branch name
- From current HEAD

**checkout**: Switch branches
- Random existing branch

**merge**: Merge branches
- Random source branch
- May create conflicts

### Index Operations

**rebuild_index**: Force full rebuild
- Simulates fresh clone scenario

**corrupt_index**: Intentionally break index
- Random corruption type (delete rows, modify values)
- Tests recovery behavior

## Parameter Generation

### Random Distributions

- Document count: Weighted toward low (1-10), tail to 100
- Body size: Normal distribution, mean 100 lines, std 50
- Link count per doc: Poisson, lambda 3
- Operation frequency: Configured weights

### Seed Control

The `--seed N` flag enables reproducible runs:
- All random choices derive from seed
- Failure reproduction: note seed, re-run with same seed
- Default: current timestamp

## Postcondition Validation

After each operation, validate:

### Structural Invariants

- All documents have valid Lattice IDs
- All links target existing IDs
- No duplicate IDs exist
- Index matches filesystem state

### Semantic Invariants

- Closed issues don't appear in `lat ready`
- Blocked issues show blocking reasons
- Context budget is respected
- Search results match filters

### Recovery Invariants

- Index corruption triggers rebuild
- Rebuild succeeds and produces consistent state
- No operations permanently break repository

## Reporting

### On System Error

Output includes:

```
=== CHAOS MONKEY FAILURE ===
Seed: 12345678
Operations executed: 847
Failing operation: modify_document

Operation details:
  Document: LXXXX (path/to/doc.md)
  Field: status
  Old value: open
  New value: blocked

Error output:
  System Error: Index constraint violation at ...

Minimal reproduction:
  lat chaosmonkey --seed 12345678 --max-ops 847 --stop-before-last

Recent operations:
  [844] create_issue LYYYY
  [845] add_link LYYYY -> LZZZZ
  [846] commit "test commit"
  [847] modify_document LXXXX (FAILED)

Log file: .lattice/chaosmonkey-12345678.log
```

### On Postcondition Failure

```
=== POSTCONDITION FAILURE ===
Seed: 12345678
Operations executed: 423
Failing check: index_matches_filesystem

Expected documents: 47
Actual in index: 46
Missing: LXXXX (path/to/doc.md)

The document exists in filesystem but not in index.
This suggests an indexing bug in the previous operation.

Previous operation:
  [422] move_document LXXXX from old/path to new/path
```

## Command Options

```
lat chaosmonkey [OPTIONS]

OPTIONS:
  --seed <N>          Random seed for reproducibility
  --max-ops <N>       Maximum operations before stopping (default: 10000)
  --operations <list> Comma-separated operation types to include
  --exclude <list>    Comma-separated operation types to exclude
  --weights <spec>    Custom operation weights (e.g., "create:10,delete:1")
  --stop-before-last  Stop just before the last operation (for debugging)
  --log-level <level> Logging verbosity (error/warn/info/debug/trace)
  --output <path>     Write detailed log to file
```

## Integration

### Automated Fuzzing

Run chaos monkey in CI on a schedule:

```yaml
fuzz:
  schedule: "0 3 * * *"  # Daily at 3 AM
  steps:
    - run: lat chaosmonkey --max-ops 100000
```

### Issue Creation

On failure, create issue with reproduction info:

```bash
lat create "Chaos monkey failure: seed $SEED" \
  --type bug --priority 1 \
  --path issues/ \
  --body-file chaosmonkey-$SEED.log
```

### Regression Tests

Confirmed bugs become deterministic tests:

```rust
#[test]
fn regression_chaosmonkey_seed_12345678() {
    // Bug discovered by chaos monkey, now a permanent test
    let env = TestEnv::new();
    // ... operations from reproduction ...
}
```

## Limitations

### Not Tested

- Network operations (Lattice has none)
- Multi-process concurrency (single-user tool)
- Very large repositories (>10000 documents)

### Known Constraints

- Some git operations require real git (not faked)
- File permission errors are hard to simulate
- Memory limits aren't tested
