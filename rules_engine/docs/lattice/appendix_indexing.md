# Appendix: Indexing

This appendix documents the SQLite index schema, reconciliation algorithm, and
performance tuning. See [Lattice Design](lattice_design.md#index-architecture)
for an overview.

## Design Principles

The SQLite index is a performance cache, never a source of truth. Any index
state can be discarded and rebuilt from git. When in doubt, rebuild.

Index location: `.lattice/index.sqlite` (gitignored).

## Schema

### documents

| Column | Type | Description |
|--------|------|-------------|
| id | TEXT PK | Lattice ID |
| parent_id | TEXT | Parent document ID |
| path | TEXT UNIQUE | Relative path from repo root |
| name | TEXT | Document name |
| description | TEXT | Document description |
| task_type | TEXT | NULL for knowledge base docs |
| is_closed | INTEGER | 1 if path contains `/.closed/`, 0 otherwise |
| priority | INTEGER | 0-4 |
| created_at | TEXT | ISO 8601 |
| updated_at | TEXT | ISO 8601 |
| closed_at | TEXT | ISO 8601 |
| body_hash | TEXT | SHA-256 for change detection |
| indexed_at | TEXT | Last index update |
| content_length | INTEGER | Body length |
| link_count | INTEGER | Outgoing links (trigger-maintained) |
| backlink_count | INTEGER | Incoming links (trigger-maintained) |
| view_count | INTEGER | Local view count (denormalized from views table) |
| is_root | INTEGER | 1 if filename matches directory name, 0 otherwise |
| in_tasks_dir | INTEGER | 1 if path contains `/tasks/`, 0 otherwise |
| in_docs_dir | INTEGER | 1 if path contains `/docs/`, 0 otherwise |

The `is_closed` column is computed from the `path` column: if the path contains
`/tasks/.closed/` or `/.closed/` anywhere, the document is closed. This is set
during indexing and updated when documents are moved via `lat close` or
`lat reopen`.

The `is_root` column is computed by comparing the filename (without `.md`) to
the containing directory name. For example, `api/api.md` has `is_root = 1`
because the filename `api` matches the directory name `api`.

The `in_tasks_dir` and `in_docs_dir` columns support lint rule validation for
the standard directory structure.

### links

| Column | Type | Description |
|--------|------|-------------|
| source_id | TEXT | Linking document |
| target_id | TEXT | Linked document |
| link_type | TEXT | 'body' or 'frontmatter' |
| position | INTEGER | Order in source |

Indexed on source_id and target_id.

### labels

| Column | Type |
|--------|------|
| document_id | TEXT |
| label | TEXT |

Indexed on both columns.

### index_metadata

Single-row table: schema_version, last_commit (git hash), last_indexed.

### client_counters

Per-client document counter: client_id (PK), next_counter.

### directory_roots

Precomputed hierarchy: directory_path (PK), root_id, parent_path, depth.

### content_cache

L2 cache for document body content, reducing filesystem reads for frequently
accessed documents. This cache is particularly useful for:

- `lat show` when displaying multiple documents (avoids repeated file reads)
- `lat search` result snippets (avoids reading entire files for context)
- Template composition (caches ancestor document content for efficient lookup)

| Column | Type | Description |
|--------|------|-------------|
| document_id | TEXT PK | Lattice ID |
| content | TEXT | Full document body |
| content_hash | TEXT | SHA-256 for invalidation |
| accessed_at | TEXT | Last access timestamp |
| file_mtime | INTEGER | File mtime at cache time |

**Cache policy:** Entries are validated by comparing `file_mtime` against
current filesystem mtime. Stale entries are refreshed on access. Least-recently
accessed entries are evicted when cache exceeds 100 documents.

**Why "L2":** The L1 cache is the operating system's filesystem cache. This L2
cache provides an additional layer within SQLite, avoiding syscall overhead for
hot documents and enabling efficient queries that need body content.

### views

| Column | Type | Description |
|--------|------|-------------|
| document_id | TEXT PK | Lattice ID |
| view_count | INTEGER | Total views |
| last_viewed | TEXT | ISO 8601 timestamp |

View tracking is stored in SQLite (not a separate JSON file) for concurrent
safety. SQLite handles concurrent reads/writes via WAL mode. See
[Appendix: Overview Command](appendix_overview_command.md) for how view data
influences document ranking.

**Denormalization note:** The `documents.view_count` column is a denormalized
copy of `views.view_count`, maintained by triggers for query performance. This
allows `lat overview` ranking queries to sort by view count without joining
tables. The `views` table is the source of truth; triggers update
`documents.view_count` when views are recorded.

Template content (Context and Acceptance Criteria sections) is resolved at
query time by walking the `directory_roots` hierarchy. No additional tables
are needed for template storage.

## Reconciliation

Runs at the start of every `lat` command.

**Fast path**: If HEAD unchanged and no uncommitted .md changes, skip.

**Incremental path**: `git diff --name-only <last_commit>..HEAD -- '*.md'`
to find changed files. Re-parse modified, remove deleted. Check `git status`
for uncommitted changes.

**Full rebuild**: Triggered by missing index, schema mismatch, errors, or
`lat check --rebuild-index`. Delete index, create schema, parse all .md files.
Schema version is stored in `index_metadata`; version mismatch triggers rebuild.

Any error during reconciliation triggers full rebuild.

## Full-Text Search

FTS5 with external content mode (no duplicate storage):

```sql
CREATE VIRTUAL TABLE fts_content USING fts5(
    document_id, body,
    content='documents', content_rowid='rowid',
    automerge=4
);
```

Triggers keep FTS in sync. After rebuild, optimize:

```sql
INSERT INTO fts_content(fts_content) VALUES('optimize');
```

### Search Syntax

`lat search` supports FTS5 query syntax:

| Syntax | Example | Matches |
|--------|---------|---------|
| Word | `error` | Documents containing "error" |
| Phrase | `"login bug"` | Exact phrase |
| AND | `error AND login` | Both terms |
| OR | `error OR warning` | Either term |
| NOT | `error NOT test` | First without second |
| Prefix | `auth*` | Words starting with "auth" |
| NEAR | `NEAR(error login, 5)` | Terms within 5 words |

Queries are case-insensitive. Stemming is not enabled by default.

## SQLite Configuration

Execute on every connection open:

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA temp_store = MEMORY;
PRAGMA mmap_size = 268435456;  -- 256MB
PRAGMA busy_timeout = 5000;
```

Run `PRAGMA optimize` before closing. After bulk operations:
`PRAGMA wal_checkpoint(TRUNCATE)`.

**WAL mode**: Concurrent readers/writers, 30-60% faster commits. Auto-checkpoints
at ~4MB.

**Synchronous NORMAL**: Safe with WAL. Commits in last few ms before power loss
may roll back—acceptable for a rebuildable cache.

**Memory-mapped I/O**: Set mmap_size to at least expected index size. Disable
(set to 0) on network filesystems.

## Performance Targets

For 10,000 documents:

| Operation | Target |
|-----------|--------|
| Connection open + configure | <5ms |
| Document lookup by ID | <1ms |
| FTS search (simple) | <20ms |
| FTS search (complex) | <50ms |
| Incremental reconciliation | <500ms |
| Full rebuild | <30s |

## Size Projections

| Documents | Index Size |
|-----------|------------|
| 1,000 | ~8MB |
| 10,000 | ~80MB |
| 50,000 | ~400MB |

## Error Recovery

- WAL corruption: Delete `-wal` and `-shm` files; next command rebuilds
- Index corruption: Delete index file; next command rebuilds
