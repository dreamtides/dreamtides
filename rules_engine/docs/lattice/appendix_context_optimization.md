# Appendix: Context Algorithm Optimization

This appendix documents research findings on optimizing the context algorithm
for large document sets (10,000+ documents) with complex link graphs.

## Executive Summary

For CLI applications where context assembly directly impacts user-perceived
latency, the key optimizations are:

1. **Precomputed graph structures**: Store adjacency lists for O(1) neighbor lookup
2. **Tiered caching**: LRU cache for hot documents, mmap for warm, disk for cold
3. **Budget-aware traversal**: Skip subtrees that can't fit remaining budget
4. **Parallel document loading**: Concurrent I/O for selected context documents
5. **Incremental context building**: Stream results as they're computed

Target performance: <50ms context assembly for typical queries (5-10 documents).

## Graph Structure Analysis

### Document Graph Characteristics

Lattice document graphs typically exhibit:

- **Low average degree**: 3-5 links per document (body + frontmatter)
- **Power-law distribution**: Few hub documents with many backlinks
- **Locality clustering**: Documents in the same directory are more connected
- **Shallow depth**: Most paths between documents are 2-4 hops

These characteristics inform optimization strategy:

| Property | Implication |
|----------|-------------|
| Low degree | Adjacency lists more efficient than matrices |
| Power-law | Cache hub documents aggressively |
| Locality | Directory-prefixed indices valuable |
| Shallow depth | BFS works well, no need for approximations |

### Graph Storage Schema

Current schema uses a links table:

```sql
CREATE TABLE links (
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    link_type TEXT NOT NULL,  -- 'body', 'frontmatter', 'blocking', etc.
    position INTEGER,          -- Order within source document
    PRIMARY KEY (source_id, target_id, link_type)
);
CREATE INDEX idx_links_target ON links(target_id);
```

**Optimization: Precomputed adjacency counts**

Add materialized counts for common queries:

```sql
ALTER TABLE documents ADD COLUMN link_count INTEGER DEFAULT 0;
ALTER TABLE documents ADD COLUMN backlink_count INTEGER DEFAULT 0;
ALTER TABLE documents ADD COLUMN content_length INTEGER DEFAULT 0;

-- Triggers maintain counts on link changes
CREATE TRIGGER update_link_counts AFTER INSERT ON links
BEGIN
    UPDATE documents SET link_count = link_count + 1
    WHERE id = NEW.source_id;
    UPDATE documents SET backlink_count = backlink_count + 1
    WHERE id = NEW.target_id;
END;
```

Benefits:
- Instant hub document identification
- Budget estimation without content loading
- Cheaper sorting by connectivity

## Traversal Optimizations

### Current Algorithm Complexity

The greedy context algorithm has complexity:

- **Candidate generation**: O(L) where L = number of links from target
- **Label matching**: O(D) where D = total documents (for doc-context-for scan)
- **Sorting**: O(C log C) where C = candidate count
- **Selection**: O(C) with lazy loading

The label matching step is the bottleneck for large repositories.

### Optimization 1: Label Index

Replace full-table scan with indexed lookup:

```sql
CREATE TABLE document_labels (
    document_id TEXT NOT NULL,
    label TEXT NOT NULL,
    PRIMARY KEY (document_id, label)
);
CREATE INDEX idx_labels_label ON document_labels(label);

-- Query for doc-context-for matches
SELECT DISTINCT d.id, d.name, d.content_length
FROM documents d
JOIN document_labels dl ON d.id = dl.document_id
JOIN document_labels target_labels ON dl.label = target_labels.label
WHERE target_labels.document_id = ?
  AND d.doc_context_for IS NOT NULL;
```

Complexity: O(T * M) where T = target's label count, M = avg matches per label.
For typical documents (2-5 labels, 5-20 matches each): <100 rows examined.

### Optimization 2: Budget-Aware Pruning

Skip candidates that can't possibly fit:

```python
def select_context_optimized(target, budget):
    candidates = gather_candidates(target)

    # Pre-filter by size
    candidates = [c for c in candidates
                  if c.content_length <= budget]

    # Sort remaining by priority
    candidates.sort(key=lambda c: c.priority, reverse=True)

    included = []
    remaining = budget

    for doc in candidates:
        if doc.content_length <= remaining:
            included.append(doc)
            remaining -= doc.content_length

    return included
```

The `content_length` check uses the precomputed column, avoiding content load.

### Optimization 3: Directory Root Caching

Directory roots are accessed frequently and rarely change. Cache the root chain:

```sql
CREATE TABLE directory_roots (
    directory_path TEXT PRIMARY KEY,
    root_id TEXT,
    parent_path TEXT,
    depth INTEGER
);
CREATE INDEX idx_roots_depth ON directory_roots(depth);

-- Query: roots from path to repository root
SELECT root_id FROM directory_roots
WHERE ? LIKE directory_path || '%'
ORDER BY depth DESC;
```

Populated during index reconciliation. Invalidated when:
- Root document created/deleted
- Document moved between directories

### Optimization 4: Early Termination

For large candidate sets, stop when budget is effectively exhausted:

```python
def select_context_early_term(target, budget):
    candidates = gather_candidates(target)

    # Sort by priority (already done)
    included = []
    remaining = budget
    skipped_count = 0

    for doc in candidates:
        if doc.content_length <= remaining:
            included.append(doc)
            remaining -= doc.content_length
            skipped_count = 0
        else:
            skipped_count += 1

        # If we've skipped many consecutive docs, likely none will fit
        if skipped_count > 20 and remaining < 500:
            break

    return included
```

This avoids scanning thousands of large documents when budget is nearly exhausted.

## Caching Strategies

### Cache Hierarchy

Implement three-tier caching for document content:

```
┌─────────────────────────────────────────────────────────────┐
│ L1: In-Memory LRU Cache (per-command)                       │
│     - Hot documents loaded this session                     │
│     - ~50 documents, evict on memory pressure               │
│     - Hit rate: 60-80% for related document queries         │
├─────────────────────────────────────────────────────────────┤
│ L2: Memory-Mapped Content Cache (persistent)                │
│     - Recent documents accessed across commands             │
│     - ~500 documents in .lattice/content_cache              │
│     - Invalidated by git hash change                        │
├─────────────────────────────────────────────────────────────┤
│ L3: Filesystem (source of truth)                            │
│     - Git-tracked markdown files                            │
│     - Always consistent with repository state               │
└─────────────────────────────────────────────────────────────┘
```

### L1: Per-Command LRU Cache

```rust
struct ContentCache {
    cache: LruCache<LatticeId, DocumentContent>,
    max_size: usize,
}

impl ContentCache {
    fn get_or_load(&mut self, id: &LatticeId, path: &Path) -> &DocumentContent {
        if !self.cache.contains(id) {
            let content = load_document_content(path);
            self.cache.put(id.clone(), content);
        }
        self.cache.get(id).unwrap()
    }
}
```

Within a single `lat show --brief` command, the same document may be referenced
multiple times (as blocker, as linked doc, as context-for match). L1 prevents
redundant file reads.

### L2: Persistent Content Cache

Store recently-accessed document content in a SQLite table:

```sql
CREATE TABLE content_cache (
    document_id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    content_hash TEXT NOT NULL,  -- SHA256 of content
    accessed_at INTEGER NOT NULL,
    file_mtime INTEGER NOT NULL
);
CREATE INDEX idx_cache_accessed ON content_cache(accessed_at);
```

**Cache Validation:**

```python
def get_cached_content(doc_id, file_path):
    cached = query("SELECT content, file_mtime FROM content_cache WHERE document_id = ?", doc_id)
    if cached:
        current_mtime = os.path.getmtime(file_path)
        if cached.file_mtime == current_mtime:
            update_access_time(doc_id)
            return cached.content

    # Cache miss or stale - reload
    content = read_file(file_path)
    upsert_cache(doc_id, content, current_mtime)
    return content
```

**Cache Eviction:**

Run eviction when cache exceeds size limit:

```sql
DELETE FROM content_cache
WHERE document_id IN (
    SELECT document_id FROM content_cache
    ORDER BY accessed_at ASC
    LIMIT (SELECT COUNT(*) - 500 FROM content_cache)
);
```

### Cache Warming

For predictable access patterns, warm the cache proactively:

1. **On `lat prime`**: Pre-load current issue and its blockers
2. **On `lat show`**: Pre-load documents linked from target (in parallel)
3. **On index reconciliation**: Pre-load modified documents

```rust
async fn warm_cache_for_show(target_id: &LatticeId) {
    let linked_ids = get_linked_document_ids(target_id);

    // Load in parallel, up to 10 concurrent
    let futures: Vec<_> = linked_ids.iter()
        .take(10)
        .map(|id| load_document_async(id))
        .collect();

    join_all(futures).await;
}
```

### Cache Invalidation

The cache must be invalidated when source files change:

1. **Git-based invalidation**: Compare HEAD hash with cached hash
2. **mtime-based validation**: Check file modification time before use
3. **Full invalidation on branch switch**: Clear cache on checkout/merge

```rust
fn validate_cache(conn: &Connection) -> Result<bool> {
    let cached_head = get_cached_git_head(conn)?;
    let current_head = get_current_git_head()?;

    if cached_head != current_head {
        // Check if cached docs are still valid
        let changed_files = git_diff_files(&cached_head, &current_head)?;
        invalidate_cached_documents(conn, &changed_files)?;
        set_cached_git_head(conn, &current_head)?;
    }

    Ok(true)
}
```

## Parallel Loading

### Async Document Loading

When multiple documents are selected for inclusion, load them concurrently:

```rust
use tokio::task::JoinSet;

async fn load_context_documents(ids: Vec<LatticeId>) -> Vec<DocumentContent> {
    let mut tasks = JoinSet::new();

    for id in ids {
        tasks.spawn(async move {
            load_document_content(&id).await
        });
    }

    let mut results = Vec::with_capacity(tasks.len());
    while let Some(result) = tasks.join_next().await {
        if let Ok(content) = result {
            results.push(content);
        }
    }

    results
}
```

**Concurrency limit**: Cap at 10 concurrent file reads to avoid fd exhaustion.

### Parallel Graph Traversal

For commands like `lat impact` that traverse the full backlink graph:

```rust
use rayon::prelude::*;

fn find_all_dependents(root_id: &LatticeId) -> HashSet<LatticeId> {
    let mut visited = HashSet::new();
    let mut frontier = vec![root_id.clone()];

    while !frontier.is_empty() {
        // Process frontier in parallel
        let next_frontier: Vec<_> = frontier.par_iter()
            .flat_map(|id| get_backlinks(id))
            .filter(|id| !visited.contains(id))
            .collect();

        visited.extend(frontier);
        frontier = next_frontier;
    }

    visited
}
```

This is beneficial for hub documents with many backlinks.

## Path Finding Optimization

### Bidirectional BFS

For `lat path <id1> <id2>`, bidirectional BFS halves the search space:

```python
def find_path_bidirectional(source, target):
    if source == target:
        return [source]

    # Search from both ends
    forward = {source: None}
    backward = {target: None}
    forward_frontier = [source]
    backward_frontier = [target]

    while forward_frontier and backward_frontier:
        # Expand smaller frontier
        if len(forward_frontier) <= len(backward_frontier):
            forward_frontier, meeting = expand(forward_frontier, forward, backward)
        else:
            backward_frontier, meeting = expand(backward_frontier, backward, forward)

        if meeting:
            return reconstruct_path(forward, backward, meeting)

    return None  # No path exists

def expand(frontier, visited, other_visited):
    next_frontier = []
    for node in frontier:
        for neighbor in get_neighbors(node):
            if neighbor in other_visited:
                return [], neighbor  # Found meeting point
            if neighbor not in visited:
                visited[neighbor] = node
                next_frontier.append(neighbor)
    return next_frontier, None
```

For typical document graphs (shallow, sparse), this reduces iterations from
O(b^d) to O(b^(d/2)) where b is branching factor and d is path length.

### Path Caching

Cache recently computed paths for repeated queries:

```sql
CREATE TABLE path_cache (
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    path TEXT NOT NULL,  -- JSON array of IDs
    computed_at INTEGER NOT NULL,
    PRIMARY KEY (source_id, target_id)
);
```

Invalidate on any link change. For stable repositories, this provides O(1)
path lookups for common document pairs.

## Benchmarking Guidelines

### Synthetic Test Data

Generate test repositories with controlled characteristics:

```python
def generate_test_repo(doc_count, avg_links, hub_count):
    """Generate a test repository with specified characteristics."""
    docs = []

    # Create hub documents (high connectivity)
    hubs = [create_document(f"hub-{i}") for i in range(hub_count)]

    # Create regular documents with links
    for i in range(doc_count - hub_count):
        doc = create_document(f"doc-{i}")

        # Add random links (power-law distribution)
        link_count = min(int(random.paretovariate(1.5)), avg_links * 3)
        targets = random.sample(docs + hubs, min(link_count, len(docs) + len(hubs)))
        doc.links = targets

        docs.append(doc)

    return docs + hubs
```

### Benchmark Scenarios

| Scenario | Documents | Links/Doc | Hubs | Target Time |
|----------|-----------|-----------|------|-------------|
| Small repo | 100 | 3 | 2 | <10ms |
| Medium repo | 1,000 | 4 | 10 | <20ms |
| Large repo | 10,000 | 5 | 50 | <50ms |
| Hub-heavy | 10,000 | 5 | 500 | <100ms |
| Dense graph | 1,000 | 20 | 50 | <100ms |

### Profiling Commands

```bash
# Single document show with context
lat show LXXXX --context 5000

# Briefing mode (maximum context)
lat show LXXXX --brief

# Path finding (worst case: distant documents)
lat path LXXXX LYYYY

# Impact analysis on hub document
lat impact LROOT

# Full repository context-for scan
lat list --context-for authentication
```

### Metrics to Track

Log these metrics to `.lattice/logs.jsonl`:

```json
{
    "op": "context_assembly",
    "target_id": "LXXXX",
    "candidates_generated": 47,
    "candidates_selected": 8,
    "budget_used": 4823,
    "cache_hits": 5,
    "cache_misses": 3,
    "duration_ms": 23
}
```

Aggregate metrics:
- P50/P95/P99 latencies per operation type
- Cache hit rate over time
- Budget utilization (used vs allocated)
- Candidate efficiency (selected / generated)

## Implementation Priorities

### Phase 1: Index Optimizations (Required)

1. Add `content_length` column to documents table
2. Add `link_count` and `backlink_count` columns
3. Create `document_labels` table with indexes
4. Create `directory_roots` table

**Estimated impact**: 2-5x improvement for label matching queries.

### Phase 2: Caching (High Value)

1. Implement L1 in-memory cache
2. Add content_cache table
3. Implement cache warming on `lat show`
4. Add git-based cache invalidation

**Estimated impact**: 3-10x improvement for repeated queries.

### Phase 3: Parallel Loading (Medium Value)

1. Add async document loading
2. Implement concurrent file I/O
3. Add concurrency limits

**Estimated impact**: 2-3x improvement for briefing mode.

### Phase 4: Advanced Graph Algorithms (Low Priority)

1. Bidirectional BFS for path finding
2. Parallel graph traversal for impact analysis
3. Path caching

**Estimated impact**: 2-5x improvement for graph commands only.

## References

- [The Art of Multiprocessor Programming](https://dl.acm.org/doi/book/10.5555/2385452) - Graph algorithm parallelization
- [LRU Cache Design](https://leetcode.com/problems/lru-cache/) - Standard cache implementation
- [Bidirectional Search](https://www.aaai.org/Papers/AAAI/1994/AAAI94-035.pdf) - Path finding optimization
- [SQLite Query Optimization](https://www.sqlite.org/optoverview.html) - Index design
- [Tokio Task Spawning](https://tokio.rs/tokio/tutorial/spawning) - Async Rust patterns
