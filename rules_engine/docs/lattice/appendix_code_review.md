# Appendix: Code Review

Checklist for reviewing Lattice changes to maintain the "bulletproof" design goal.

## Error Handling

- [ ] All failure modes identified and classified (expected vs system error)
- [ ] Expected errors return `LatticeError` with actionable messages
- [ ] System errors use `panic!` with explanation
- [ ] Structured JSON output for expected errors (with `--json`)

## Logging

All code must use the `tracing` crate. Silent operations are a critical design
flaw—every significant action should be traceable in logs.

**Required logging:**
- [ ] Entry/exit of public functions (`#[instrument]` or manual spans)
- [ ] All git operations with command and result
- [ ] All SQLite queries with affected rows
- [ ] All file system operations (read, write, delete)
- [ ] Error conditions before returning `LatticeError`
- [ ] State observations (unexpected conditions, recovery actions)

**Log levels:** `error!` (operation failed), `warn!` (degraded/recoverable),
`info!` (milestones), `debug!` (--verbose detail), `trace!` (dev only)

```rust
#[instrument(skip(index), fields(id = %id))]
pub fn get_document(index: &Index, id: LatticeId) -> Result<Document, LatticeError> {
    let doc = index.get(&id).ok_or_else(|| {
        warn!(%id, "document not found");
        LatticeError::DocumentNotFound { id }
    })?;
    Ok(doc)
}
```

## Concurrency

- [ ] Safe under concurrent `lat` invocations (multiple terminals, CI)
- [ ] SQLite uses WAL mode with appropriate busy timeout
- [ ] File operations handle TOCTOU races (check-then-use)
- [ ] Claim files use atomic create/delete, not read-modify-write

## Data Integrity & Atomicity

- [ ] Multi-step operations are atomic or safely interruptible
- [ ] Partial failures leave consistent state (no half-written files)
- [ ] Index can always be rebuilt from git (never sole source of truth)
- [ ] Writes use temp file + rename pattern for atomicity

## Performance

- [ ] No O(n²) or worse algorithms on document count
- [ ] Git operations batched where possible (single `git ls-files`, not per-file)
- [ ] SQLite queries use indices; no full table scans for common operations
- [ ] Large file reads are lazy/streaming where possible

## Test Coverage

- [ ] Happy path test for new functionality
- [ ] Test each expected error case returns correct `LatticeError`
- [ ] Edge cases from list below have test coverage
- [ ] Chaos monkey updated if new operations added

## Code Organization

**Coupling:** Flag tight coupling between modules. Prefer dependency injection
(e.g., `GitOps` trait) over direct calls to external systems.

**Size limits:** Functions over 50 lines or files over 500 lines should be
split. Suggest extraction points for large blocks.

**Reuse:** Flag duplicate logic that should be extracted. Common patterns:
path manipulation, frontmatter parsing, ID validation, error formatting.

## Edge Cases to Consider

**File system:** File deleted/moved between check and use, permission denied,
disk full, symlinks, unicode paths

**Git state:** Uncommitted changes, detached HEAD, shallow/sparse clone, merge
conflicts, rebase in progress, worktrees with divergent state

**Document content:** Missing/malformed frontmatter, circular dependencies,
self-references, duplicate IDs, documents at 500-line boundary

**Index state:** Missing/corrupted index, out of sync with filesystem,
concurrent access from multiple processes

**User input:** Invalid IDs, nonexistent paths, empty/very long strings,
special characters, shell metacharacters

## Code Style

- [ ] Functions return `Result<T, LatticeError>`, not `Option` for errors
- [ ] No `.unwrap()`; use `.ok_or(LatticeError::...)?` or `.unwrap_or_else(|| panic!())`
- [ ] Panics only for invariant violations (Lattice's fault, not user's)
- [ ] Follow patterns in existing code; maintain consistency
