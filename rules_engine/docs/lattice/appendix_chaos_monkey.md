# Appendix: Chaos Monkey

This appendix documents the chaos monkey fuzz testing system. See
[Lattice Design](lattice_design.md#chaos-monkey) for an overview and
[Appendix: Testing Strategy](appendix_testing_strategy.md) for the broader
testing architecture.

The `lat chaosmonkey` command performs automated fuzz testing to discover bugs.
It executes random sequences of operations until an invariant violation occurs.

## What Constitutes a Bug

The chaos monkey detects **system errors**, not **expected errors**. This
distinction follows the error handling philosophy in
[Lattice Design](lattice_design.md#error-handling): if a problem is *Lattice's
fault*, it's a bug; if it's the *user's fault*, it's expected behavior.

**System errors (bugs):**
- `lat update` crashes with a panic
- Index contains an ID that doesn't exist in the filesystem
- Git operation fails (Lattice should ensure valid git state)
- Operation claims success but state is wrong

**Expected errors (not bugs):**
- `lat update` returns an error because the document doesn't exist
- A document contains a link to a deleted document
- Invalid syntax, missing fields, or permission problems

Expected errors produce clear error messages with exit code 2 or 3. System
errors are:

1. **Panics**: Any Rust panic (Lattice's fault, never user's)
2. **Corrupt state**: Index doesn't match filesystem after any operation
3. **Invariant violations**: Duplicate IDs, malformed IDs in index
4. **Git failures**: Git operations that fail when Lattice should have ensured
   valid state
5. **Silent failures**: Operation claims success but state is wrong

## Operations

### High-Level (lat commands)

Standard lat commands with random valid and invalid arguments:

- `lat create` with random paths, types, metadata
- `lat update` with random field changes
- `lat close` on random tasks (moves to `tasks/.closed/`)
- `lat reopen` on random closed tasks (moves from `tasks/.closed/`)
- `lat prune` with and without `--force`
- `lat mv` to random destinations
- `lat search` with random queries
- `lat check --rebuild-index` at random points

### Low-Level (filesystem)

Direct filesystem manipulation to simulate external tools, user edits, and
recovery scenarios:

- Create files with valid/invalid Lattice headers
- Delete `.md` files directly (bypassing lat)
- Modify file contents (header corruption, body changes)
- Rename/move files via filesystem
- Create duplicate IDs across files
- Delete/corrupt `.lattice/index.sqlite`
- Modify index database directly

### Git Operations

Real git operations (not simulated):

- `git add`, `git commit` at random points
- `git checkout` to different branches
- `git merge` with potential conflicts
- `git stash` and `git stash pop`

## Invariants

After every operation, validate:

1. **Index-filesystem consistency**: Every ID in the index has a corresponding
   file, and every file with a Lattice ID is in the index
2. **ID uniqueness**: No two files share the same Lattice ID
3. **ID format**: All IDs in the index match the Lattice ID format
4. **Git state validity**: Git operations succeed when Lattice initiates them
   (Lattice should ensure valid git state before operating)
5. **No panics**: Previous operation completed without a panic or system error
6. **Closed state consistency**: Index `is_closed` matches path containing
   `/tasks/.closed/` or `/.closed/`
7. **Root document consistency**: Index `is_root` matches filename = directory name
8. **Directory structure consistency**: Index `in_tasks_dir` and `in_docs_dir`
   match path components
9. **Link path validity**: After `lat close`/`lat reopen`, all links to moved
   documents point to current paths

Note: Broken links are **not** invariant violations. Documents may reference
IDs that no longer exist—this is an expected error (user's responsibility to
maintain link integrity, or use `lat prune --force`).

## Execution

```
lat chaosmonkey [OPTIONS]

OPTIONS:
  --seed <N>           Random seed for reproducibility
  --max-ops <N>        Maximum operations (default: 10000)
  --operations <list>  Include only these operation types
  --exclude <list>     Exclude these operation types
  --stop-before-last   Stop before the final operation that would cause failure
```

On invariant violation, output includes the seed, operation count, failing
operation details, and the specific invariant that failed. Re-run with the same
seed to reproduce.

The `--stop-before-last` flag is useful for debugging: it runs the sequence until
just before the failing operation, leaving the repository in a state where you can
manually inspect and reproduce the issue.