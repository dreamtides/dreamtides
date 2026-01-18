# Appendix: Startup Operations

This appendix documents the operations that run automatically before every `lat`
command. See [Lattice Design](lattice_design.md#git-as-source-of-truth) for the
design philosophy behind this "fake daemon" approach.

## Design Philosophy

Lattice never runs background daemon processes. Instead, common maintenance
operations execute opportunistically at the start of every `lat` command
invocation. This approach trades slight startup latency for operational
simplicity—no daemon management, no stale cache problems, no zombie processes.

All startup operations must complete quickly. The combined overhead should be
imperceptible for typical repositories (<50ms for most operations).

## Concurrency Requirements

Multiple `lat` processes may run simultaneously (multiple terminals, CI jobs,
worktrees). All startup operations must be:

**Idempotent:** Running twice produces the same result as running once. No
operation should fail or corrupt state if another process runs concurrently.

**Atomic:** Use SQLite transactions, temp file + rename, or atomic
create/delete patterns. Never read-modify-write without locking.

**Tolerant:** If another process is mid-operation, either wait (SQLite
busy_timeout) or skip gracefully (symlinks, log rotation).

## Operations

### Index Reconciliation

The SQLite index is validated and updated if needed. See
[Appendix: Indexing](appendix_indexing.md#reconciliation) for the algorithm.

**Fast path** (~1ms): If HEAD is unchanged and no uncommitted `.md` changes
exist, skip entirely.

**Incremental path** (~50-500ms): Query git for changed files since last index
update, re-parse modified documents, remove deleted entries.

**Full rebuild** (seconds): Triggered by missing index, schema mismatch, or
corruption. Delete and recreate from scratch.

### Skill Symlink Synchronization

Documents with `skill: true` in their frontmatter automatically become Claude
Skills. Lattice generates symlinks in `.claude/skills/` pointing to the actual
Lattice documents, enabling Claude Code to discover them without file
duplication.

**Operation**: Scan index for skill-enabled documents, create/update/remove
symlinks to match current state.

**Performance**: O(n) where n is skill document count (typically <10).

### Claim Cleanup

Stale claims are detected and cleaned up. A claim is stale if:

- The referenced task no longer exists
- The task is in a `.closed/` directory
- The worktree path no longer exists
- The claim age exceeds threshold (default 7 days, configurable via
  `stale_days` in `ClaimConfig`)

**Operation**: Scan `~/.lattice/claims/<repo-hash>/` directory, validate each
claim file against current repository state. Check task paths in index to
determine if they are closed.

**Performance**: O(n) where n is active claim count (typically <5).

### Log Rotation

The `.lattice/logs.jsonl` file is rotated when it exceeds 10MB.

**Operation**: If file size > 10MB, rename to `logs.jsonl.1` (overwriting any
existing backup) and create fresh log file.

**Performance**: Single stat() call in common case, file rename when triggered.

## SQLite Connection Setup

Every database connection executes WAL mode and performance pragmas. See
[Appendix: Indexing](appendix_indexing.md#sqlite-configuration) for the
complete configuration.

## Performance Budget

Target startup overhead for a 10,000-document repository:

| Operation | Target | Notes |
|-----------|--------|-------|
| Index reconciliation (fast path) | <5ms | HEAD unchanged |
| Index reconciliation (incremental) | <500ms | Typical daily use |
| Skill symlink sync | <10ms | Few skill documents |
| Claim cleanup | <5ms | Few active claims |
| Log rotation check | <1ms | Single stat() |
| SQLite connection | <5ms | Pragma execution |
| **Total (fast path)** | **<20ms** | |

If startup exceeds 100ms, emit a warning log entry for investigation.

## Skipping Startup Operations

The `--no-startup` flag (hidden, for debugging) skips all startup operations.
This is useful for profiling individual commands but may cause stale data.
