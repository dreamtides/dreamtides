# Appendix: Indexing Strategy

This appendix documents the SQLite index reconciliation algorithm and schema.
See [Lattice Design](lattice_design.md#index-architecture) for an overview,
and [Appendix: Indexing Performance](appendix_indexing_performance.md) for
SQLite tuning guidance.

## Design Principles

The SQLite index is a performance cache, never a source of truth. Any index
state can be discarded and rebuilt from git. This principle simplifies
error recovery: when in doubt, rebuild.

## Index Location

The index lives at `.lattice/index.sqlite` in the repository root. This
file should be gitignored. The `.lattice/` directory also contains:

- `logs.jsonl`: Operation log
- `config.toml`: Local configuration overrides (optional)

## Schema Overview

### documents Table

Primary document metadata storage.

**Columns:**
- `id` TEXT PRIMARY KEY: Lattice ID
- `parent_id` TEXT: Parent document ID (from directory root)
- `path` TEXT UNIQUE: Relative path from repo root
- `name` TEXT: Document name from frontmatter
- `description` TEXT: Document description
- `issue_type` TEXT: NULL for knowledge base docs
- `status` TEXT: Issue status
- `priority` INTEGER: Issue priority (0-4)
- `created_at` TEXT: ISO 8601 timestamp
- `updated_at` TEXT: ISO 8601 timestamp
- `closed_at` TEXT: ISO 8601 timestamp
- `body_hash` TEXT: SHA-256 of document body for change detection
- `indexed_at` TEXT: When this row was last updated
- `content_length` INTEGER: Body length in characters
- `link_count` INTEGER: Outgoing link count (maintained by trigger)
- `backlink_count` INTEGER: Incoming link count (maintained by trigger)
- `view_count` INTEGER: Local view count (from views.json)

### links Table

Document cross-references.

**Columns:**
- `source_id` TEXT: Linking document ID
- `target_id` TEXT: Linked document ID
- `link_type` TEXT: 'body' or 'frontmatter'
- `position` INTEGER: Order in source document

**Indexes:** On source_id, target_id for bidirectional queries.

### labels Table

Document label associations.

**Columns:**
- `document_id` TEXT
- `label` TEXT

**Indexes:** On document_id, on label for label-based queries.

### index_metadata Table

Single-row table tracking index state.

**Columns:**
- `id` INTEGER PRIMARY KEY CHECK (id = 1)
- `schema_version` INTEGER
- `last_commit` TEXT: Git commit hash at last full reconciliation
- `last_indexed` TEXT: Timestamp of last index update

### client_counters Table

Per-client document counter state.

**Columns:**
- `client_id` TEXT PRIMARY KEY
- `next_counter` INTEGER

### directory_roots Table

Precomputed root document chain for hierarchy queries.

**Columns:**
- `directory_path` TEXT PRIMARY KEY: Directory relative path
- `root_id` TEXT: Root document ID (or NULL if no root)
- `parent_path` TEXT: Parent directory path
- `depth` INTEGER: Depth from repository root

**Indexes:** On depth for ordered traversal.

Populated during reconciliation. Invalidated when root documents change.

### content_cache Table

Persistent cache for document body content (L2 cache layer).

**Columns:**
- `document_id` TEXT PRIMARY KEY: Document Lattice ID
- `content` TEXT: Cached body text
- `content_hash` TEXT: SHA-256 for validation
- `accessed_at` INTEGER: Unix timestamp of last access
- `file_mtime` INTEGER: File modification time when cached

**Indexes:** On accessed_at for LRU eviction.

## Reconciliation Algorithm

### Trigger Points

Reconciliation runs at the start of every `lat` command. The algorithm
is designed to be fast for the common case (no changes) and correct for
all cases.

### Fast Path: No Changes

1. Read `index_metadata.last_commit`
2. Compare to current `git rev-parse HEAD`
3. If equal and no uncommitted changes to .md files, skip reconciliation

### Incremental Update Path

When HEAD has changed since last reconciliation:

1. Run `git diff --name-only <last_commit>..HEAD -- '*.md'`
2. For each modified file:
   a. If file exists: re-parse and update index
   b. If file deleted: remove from index
3. Run `git status --porcelain -- '*.md'` for uncommitted changes
4. Update `index_metadata.last_commit` to HEAD

### Full Rebuild Path

Triggered when:
- No index exists
- Schema version mismatch
- Reconciliation encounters unexpected state
- User runs `lat check --rebuild-index`
- Shallow clone boundary prevents incremental diff
- Partial clone triggers unexpected network errors

See [Appendix: Git Edge Cases](appendix_git_edge_cases.md) for detailed
fallback behavior in non-standard repository configurations.

Full rebuild process:
1. Delete existing index
2. Create fresh schema
3. Enumerate all .md files from git
4. Parse each file and insert into index
5. Set `last_commit` to HEAD

### Error Recovery

Any error during reconciliation triggers full rebuild. Errors include:
- SQLite constraint violations
- Parse errors in documents
- Git operation failures
- Unexpected NULL values

## Change Detection

### Document Hashing

Each document's body (after frontmatter) is hashed with SHA-256. During
reconciliation, if a document's hash hasn't changed, its links aren't
re-parsed. Only frontmatter is re-read for metadata changes.

### Git-Based Detection

Using git for change detection provides:
- Automatic exclusion of gitignored files
- Consistent behavior with what's committed
- Awareness of file renames
- Detection of uncommitted modifications

### Timestamp Comparison

As a secondary check, the index stores `indexed_at` per document and
compares against file mtime. This catches changes that git hasn't yet
tracked (unstaged modifications).

## Full-Text Search

### FTS5 Configuration

The index uses SQLite FTS5 for body text search with external content:

```sql
CREATE VIRTUAL TABLE fts_content USING fts5(
    document_id,
    body,
    content='documents',
    content_rowid='rowid',
    automerge=4
);
```

External content mode (`content='documents'`) avoids storing duplicate body
text, reducing index size by approximately 40%. The `automerge=4` setting
balances insert performance with query speed.

### Triggers for Sync

Insert/update/delete triggers on the documents table keep FTS5 in sync.
This happens automatically within SQLite transactions.

### Query Interface

FTS queries use the MATCH operator:

```sql
SELECT document_id FROM fts_content WHERE body MATCH ?
```

Results are ranked by BM25. The `lat list --name-contains` flag uses
FTS for text search.

### FTS Optimization

After full index rebuilds, run optimization to merge all FTS B-trees:

```sql
INSERT INTO fts_content(fts_content) VALUES('optimize');
```

This reduces query latency at the cost of a one-time rebuild (1-5 seconds
for 10,000 documents). See [Appendix: Indexing Performance](appendix_indexing_performance.md)
for detailed tuning guidance.

## Transaction Handling

### Write Operations

All index modifications occur within transactions:
1. BEGIN IMMEDIATE (acquire write lock)
2. Perform updates
3. COMMIT or ROLLBACK on error

### Read Operations

Reads use default transaction mode for consistency. Long-running reads
don't block writers due to WAL mode.

### WAL Mode

The index uses WAL (Write-Ahead Logging) mode with these benefits:
- Concurrent readers and writers (readers don't block writers)
- 30-60% faster commits with NORMAL synchronous mode
- Automatic checkpointing at 1000 pages (~4MB)

WAL files (`.lattice/index.sqlite-wal` and `-shm`) are also gitignored.

After bulk operations, run `PRAGMA wal_checkpoint(TRUNCATE)` to reset the
WAL file size. See [Appendix: Indexing Performance](appendix_indexing_performance.md)
for checkpoint management details.

## Performance Considerations

### Connection Configuration

Each `lat` command opens a fresh connection with optimized PRAGMAs:

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA temp_store = MEMORY;
PRAGMA mmap_size = 268435456;  -- 256MB
PRAGMA busy_timeout = 5000;
```

Before closing, run `PRAGMA optimize` to help the query planner.

### Prepared Statement Caching

Use rusqlite's `prepare_cached()` for frequently-executed queries. While
the cache doesn't persist across commands, it helps within a single
command that executes the same query multiple times.

### Batch Operations

Bulk operations (like `lat update` with multiple IDs) batch changes in a
single transaction for atomicity and performance. Without explicit
transactions, SQLite creates an implicit transaction per statement,
adding significant overhead.

### Index Size

For a repository with N documents and M links:
- Documents table: O(N) rows
- Links table: O(M) rows, typically M = 3-5N
- FTS index: O(N × average_body_size)

Projections for typical repositories:
- 1,000 documents: ~8MB total
- 10,000 documents: ~80MB total
- 50,000 documents: ~400MB total

See [Appendix: Indexing Performance](appendix_indexing_performance.md) for
detailed benchmarking targets and tuning guidance.

## Migration Strategy

### Schema Versioning

`index_metadata.schema_version` tracks the current schema version. When
the code expects a different version than what's stored, full rebuild
is triggered.

### Forward Compatibility

Old versions of Lattice refuse to use indexes with newer schema versions,
forcing rebuild. This prevents corruption from version mismatches.
