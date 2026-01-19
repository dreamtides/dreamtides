# Appendix: Code Review

Checklist for reviewing Lattice changes to maintain the "bulletproof" design goal.

## Summary

- **Classify errors:** Expected errors (user's fault) return `LatticeError`;
  system errors (Lattice's fault) use `panic!`
- **No silent failures:** Every significant action must be logged via `tracing`
- **High quality logs:** Always analyze logs for usefulness
- **Atomic operations:** Multi-step changes use temp file + rename; partial
  failures leave consistent state
- **Concurrency safe:** Handle concurrent `lat` invocations, TOCTOU races,
  SQLite WAL
- **Performance aware:** No O(n²) algorithms; batch git operations; use indices
- **Test expected errors:** Each `LatticeError` variant should have test
  coverage
- **High quality tests:** Tests use public APIs, are not repetitive, and have
  good assertion messages, exist in /tests/ directory (not in `mod tests {}`.)
- **Avoid `.unwrap()`:** Use `.ok_or(LatticeError::...)?` or explicit panic with
  reason
- **Keep it small:** Try for functions under 50 lines, files under 500 lines
- **Avoid code duplication:** Search for opportunities to factor out shared code
- **Ensure docs are updated:** Verify relevant markdown documentation is
  still accurate

## Error Handling

- [ ] All failure modes classified (expected errors → `LatticeError`, system
  errors → `panic!` with explanation)
- [ ] Expected errors have actionable messages and structured JSON output
- [ ] No `.unwrap()`; use `.ok_or(LatticeError::...)?` or
  `.unwrap_or_else(|| panic!("reason"))`

## Logging

- [ ] All git, SQLite, and file operations logged with results
- [ ] Error conditions logged before returning `LatticeError`
- [ ] Logs are *useful*—analyze whether they help diagnose real issues
- [ ] Appropriate levels: `error!` (failed), `warn!` (recoverable), `info!`
  (milestones), `debug!` (verbose detail)

## Atomicity & Concurrency

- [ ] Multi-step operations use temp file + rename; partial failures leave
  consistent state
- [ ] Safe under concurrent `lat` invocations (SQLite WAL, atomic file ops)
- [ ] File operations handle TOCTOU races; claim files use atomic create/delete

## Performance

- [ ] No O(n²) algorithms on document count
- [ ] Git operations batched; SQLite queries use indices
- [ ] Run `just bench-lattice` for performance-sensitive changes

## Testing

- [ ] Tests in the /tests/ directory, NOT inline in Rust files
- [ ] Tests use public APIs only, if you can't reach it via the public API don't
  test it.
- [ ] Do not add `#[cfg(test)]` helpers only for use in tests
- [ ] Happy path and each expected error case covered
- [ ] Tests are not repetitive—don't test the same thing twice
- [ ] Assertion messages explain what failed and why
- [ ] Chaos monkey updated if new operations added

## Code Organization

- [ ] Functions under 50 lines, files under 500 lines
- [ ] Duplicate logic extracted (path manipulation, frontmatter parsing, ID
  validation, error formatting)
- [ ] Loose coupling; prefer dependency injection over direct external calls
- [ ] Relevant markdown documentation updated

## Edge Cases

**File system:** TOCTOU races, permission denied, disk full, symlinks, unicode
**Git state:** Uncommitted changes, detached HEAD, shallow clone, conflicts
**Documents:** Malformed frontmatter, circular deps, duplicate IDs
**Index:** Missing/corrupted, out of sync, concurrent access

**User input:** Invalid IDs, empty/long strings, special characters

## Code Style

- [ ] Do not allow or expect dead_code. Delete dead code.
- [ ] Functions return `Result<T, LatticeError>`, not `Option` for errors
- [ ] Panics only for invariant violations (Lattice's fault, not user's)
- [ ] Follow patterns in existing code; maintain consistency
- [ ] Avoid complex Rust features (generics, traits) unless truly necessary
