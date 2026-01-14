# Appendix: Indexing Performance

This appendix documents research findings on SQLite performance optimization for
Lattice's index, targeting repositories with 10,000+ documents while maintaining
sub-100ms response times for common queries. See
[Appendix: Indexing Strategy](appendix_indexing_strategy.md) for the
reconciliation algorithm and schema design.

## Executive Summary

For CLI applications like Lattice where each command creates a short-lived
database connection, the key optimizations are:

1. **WAL mode with NORMAL synchronous**: 30-60% latency reduction
2. **Memory-mapped I/O**: Faster reads for databases under 2GB
3. **Prepared statement caching**: Essential for repeated queries
4. **PRAGMA optimize on close**: Improves query planner decisions
5. **FTS5 automerge tuning**: Balance write performance vs query speed

## Recommended PRAGMA Configuration

Execute these PRAGMAs on every connection open:

```sql
-- Journal mode (persists across connections)
PRAGMA journal_mode = WAL;

-- Synchronous mode: NORMAL is safe with WAL, 30-60% faster than FULL
PRAGMA synchronous = NORMAL;

-- Use memory for temp tables (sorting, joins)
PRAGMA temp_store = MEMORY;

-- Memory-map up to 256MB of database for faster reads
PRAGMA mmap_size = 268435456;

-- 5 second busy timeout for concurrent access
PRAGMA busy_timeout = 5000;
```

### Rationale

**WAL Mode**: Write-Ahead Logging provides concurrent read/write access. Writers
don't block readers and vice versa. WAL is the clear winner for mixed workloads.

**Synchronous NORMAL**: With WAL, NORMAL mode separates logical commits from
durable commits. Database corruption is impossible, but commits in the last few
milliseconds before power loss may roll back. This tradeoff is acceptable for
an ephemeral cache like Lattice's index.

**Memory-Mapped I/O**: Reduces syscall overhead for read operations. The 256MB
limit is conservative; increase to match database size for larger repositories.
Set to 0 on network filesystems (NFS, SMB) where mmap is unreliable.

**Busy Timeout**: Prevents immediate SQLITE_BUSY errors during concurrent
access. 5 seconds allows time for transient locks to clear.

## Connection Lifecycle for CLI Applications

Since Lattice doesn't run a daemon, each command creates a fresh connection.
This pattern requires specific handling:

### On Connection Open

```rust
fn configure_connection(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA temp_store = MEMORY;
        PRAGMA mmap_size = 268435456;
        PRAGMA busy_timeout = 5000;
    ")?;
    Ok(())
}
```

### On Connection Close

```rust
fn cleanup_connection(conn: &Connection) -> Result<()> {
    // Help query planner with statistics from this session
    conn.execute_batch("PRAGMA optimize;")?;
    Ok(())
}
```

`PRAGMA optimize` analyzes tables that have been queried since the last
optimize, helping the query planner choose better execution plans. This is
specifically recommended for applications with short-lived connections.

### Prepared Statement Caching

Rusqlite's `prepare_cached()` maintains an LRU cache of compiled statements
within a connection. For CLI tools, the cache only helps within a single
command invocation, but this still saves recompilation for repeated queries:

```rust
// Use prepare_cached for frequently-executed queries
let mut stmt = conn.prepare_cached(
    "SELECT id, path, name FROM documents WHERE status = ?"
)?;
```

The default cache size of 16 statements is adequate for most operations.
Increase via `set_prepared_statement_cache_capacity()` if profiling shows
repeated cache misses.

## WAL Checkpoint Management

### Default Behavior

SQLite auto-checkpoints when the WAL exceeds 1000 pages (~4MB). This happens
during COMMIT, adding latency to the committing transaction.

### Checkpoint Starvation

If readers continuously hold read transactions, checkpointing cannot complete
and the WAL file grows unbounded. For Lattice, this is unlikely since each
command runs a short operation and exits.

### Recommended Strategy

For Lattice's usage pattern:

1. **Rely on auto-checkpoint**: Default 1000-page threshold is reasonable
2. **Run TRUNCATE checkpoint on rebuild**: After full index rebuild, run
   `PRAGMA wal_checkpoint(TRUNCATE)` to reset WAL to zero size
3. **Monitor WAL size**: If WAL exceeds 10MB frequently, investigate whether
   read transactions are held too long

```rust
fn post_rebuild_cleanup(conn: &Connection) -> Result<()> {
    // Checkpoint and truncate WAL after bulk operations
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
    Ok(())
}
```

## FTS5 Optimization

### Index Structure

FTS5 maintains a forest of B-trees that are incrementally merged. The tradeoff:

- More trees = faster inserts, slower queries
- Fewer trees = slower inserts, faster queries

### Automerge Configuration

The `automerge` parameter (default 4) controls when merging occurs:

```sql
-- Create FTS5 table with custom automerge
CREATE VIRTUAL TABLE fts_content USING fts5(
    document_id,
    body,
    content='documents',
    content_rowid='rowid',
    automerge=4
);
```

For Lattice's workload (bulk inserts during rebuild, occasional single inserts):

- Keep default `automerge=4` for normal operation
- After full rebuild, run optimize:

```sql
INSERT INTO fts_content(fts_content) VALUES('optimize');
```

### When to Optimize FTS

Run FTS optimize:

- After full index rebuild
- Optionally after reconciliation touches 100+ documents
- Never during interactive operations (blocks other queries)

Optimization merges all B-trees into one, minimizing query time. For a 10,000
document repository, expect optimization to take 1-5 seconds.

### External Content Tables

Lattice uses external content mode (`content='documents'`), meaning FTS5
doesn't store duplicate body text. Benefits:

- Smaller database size
- Single source of truth for content
- Automatic synchronization via triggers

The tradeoff is slightly slower queries since FTS must join with the content
table for snippet generation.

## Index Size Projections

Based on typical markdown document characteristics:

| Documents | Body Text | Index Size | FTS Size | Total |
|-----------|-----------|------------|----------|-------|
| 1,000 | 2MB avg | ~5MB | ~3MB | ~8MB |
| 10,000 | 2MB avg | ~50MB | ~30MB | ~80MB |
| 50,000 | 2MB avg | ~250MB | ~150MB | ~400MB |

These projections assume average documents of 200-500 lines (~2KB) with
typical link density (3-5 links per document).

### Memory-Mapped I/O Sizing

For optimal mmap performance, set `mmap_size` to at least the expected index
size. For repositories up to 10,000 documents, 256MB is sufficient. For larger
repositories, consider 512MB or 1GB:

```sql
-- For large repositories
PRAGMA mmap_size = 1073741824;  -- 1GB
```

Never set mmap_size larger than available RAM, as this degrades to disk paging.

## VACUUM and Auto-Vacuum Strategy

### Recommendation: No Auto-Vacuum

For Lattice's ephemeral index, auto-vacuum adds overhead without significant
benefit:

1. The index can be fully rebuilt at any time
2. Full rebuild is preferable to incremental compaction
3. Auto-vacuum fragments the database

### Manual VACUUM

Run VACUUM only when:

- Index size significantly exceeds expected size
- After deleting large numbers of documents
- User explicitly requests via `lat check --rebuild-index`

VACUUM requires 2x database size in free disk space and should never run
during normal operations.

## Benchmarking Targets

For a 10,000 document repository, target performance:

| Operation | Target | Notes |
|-----------|--------|-------|
| Connection open + configure | <5ms | Including all PRAGMAs |
| Document lookup by ID | <1ms | Primary key query |
| FTS search (simple) | <20ms | Single-term query |
| FTS search (complex) | <50ms | Multi-term with ranking |
| Full reconciliation | <500ms | Incremental from git |
| Full rebuild | <30s | 10K documents from disk |

### Profiling Approach

Add timing instrumentation to `.lattice/logs.jsonl`:

```json
{
    "ts": "2025-01-13T10:00:00.000000Z",
    "op": "sqlite_query",
    "query": "SELECT ... FROM fts_content WHERE body MATCH ?",
    "duration_ms": 12,
    "rows_returned": 5
}
```

Monitor p50, p95, and p99 latencies to identify regressions.

## Error Recovery

### WAL File Corruption

If WAL file becomes corrupted (rare), recovery is automatic:

1. Delete `.lattice/index.sqlite-wal` and `-shm` files
2. Next `lat` command triggers full rebuild

### Index Corruption

Any unrecoverable SQLite error triggers full rebuild:

```rust
fn handle_sqlite_error(err: rusqlite::Error) -> Result<()> {
    log_error("sqlite", &err);
    // Trigger rebuild on next command
    fs::remove_file(".lattice/index.sqlite")?;
    Err(LatticeError::IndexCorrupted)
}
```

## References

- [SQLite Write-Ahead Logging](https://sqlite.org/wal.html)
- [SQLite FTS5 Extension](https://sqlite.org/fts5.html)
- [SQLite Memory-Mapped I/O](https://sqlite.org/mmap.html)
- [SQLite PRAGMA Statements](https://www.sqlite.org/pragma.html)
- [SQLite Performance Tuning](https://phiresky.github.io/blog/2020/sqlite-performance-tuning/)
- [PowerSync SQLite Optimizations](https://www.powersync.com/blog/sqlite-optimizations-for-ultra-high-performance)
- [Rusqlite CachedStatement](https://docs.rs/rusqlite/latest/rusqlite/struct.CachedStatement.html)
