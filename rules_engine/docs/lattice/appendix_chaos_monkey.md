# Appendix: Chaos Monkey

The `lat chaosmonkey` command performs automated fuzz testing to discover bugs.
It executes random sequences of operations until an invariant violation occurs.

## What Constitutes a Bug

The chaos monkey detects **system failures**, not **user errors**. Key
distinction:

- **Bug**: `lat modify` crashes with a panic
- **Not a bug**: `lat modify` returns an error because the file doesn't exist
- **Bug**: Index contains an ID that doesn't exist in the filesystem
- **Not a bug**: A document contains a link to a deleted document

User-facing errors (exit code 1 with a clear error message) are expected
outcomes for invalid operations. Bugs are:

1. **Panics**: Any Rust panic is a bug
2. **Corrupt state**: Index doesn't match filesystem after any operation
3. **Invariant violations**: Duplicate IDs, malformed IDs in index
4. **Silent failures**: Operation claims success but state is wrong

## Operations

### High-Level (lat commands)

Standard lat commands with random valid and invalid arguments:

- `lat create` with random paths, types, metadata
- `lat modify` with random field changes
- `lat delete` on random documents
- `lat move` to random destinations
- `lat search` with random queries
- `lat link` and `lat unlink` between random documents
- `lat rebuild` at random points

### Low-Level (filesystem)

Direct filesystem manipulation to simulate external tools, user edits, and
recovery scenarios:

- Create files with valid/invalid Lattice headers
- Delete `.md` files directly (bypassing lat)
- Modify file contents (header corruption, body changes)
- Rename/move files via filesystem
- Create duplicate IDs across files
- Delete/corrupt `.lattice/index.db`
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
4. **No panics**: Previous operation completed without crashing

Note: Broken links are **not** invariant violations. Documents may reference
IDs that no longer exist.

## Execution

```
lat chaosmonkey [OPTIONS]

OPTIONS:
  --seed <N>          Random seed for reproducibility
  --max-ops <N>       Maximum operations (default: 10000)
  --operations <list> Include only these operation types
  --exclude <list>    Exclude these operation types
```

On invariant violation, output includes the seed, operation count, failing
operation details, and the specific invariant that failed. Re-run with the same
seed to reproduce.