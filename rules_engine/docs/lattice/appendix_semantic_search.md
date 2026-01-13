# Appendix: Semantic Search

## Overview

Semantic search enables finding documents by meaning rather than exact keyword
matching. This is a complex feature requiring embedding generation, vector
storage, and similarity computation.

## Architecture

### Components

1. **Embedding Generator**: Converts document text to vector representations
2. **Vector Store**: Indexes embeddings for similarity search
3. **Query Engine**: Handles search requests and ranking

### Data Flow

```
Document → Tokenize → Embed → Store Vector
Query → Embed → Compare → Rank → Return Results
```

## Rust Crates

### Embedding Generation

**Option 1: rust-bert** (recommended)
- Full-featured transformer models in Rust
- Supports sentence-transformers models
- Local inference, no API calls
- GPU support via tch-rs

```toml
[dependencies]
rust-bert = "0.22"
tch = "0.14"  # PyTorch bindings
```

Considerations:
- Large binary size (~500MB with models)
- Requires libtorch installation
- First-run model download

**Option 2: candle** (lighter weight)
- Hugging Face's pure-Rust ML framework
- Smaller footprint than rust-bert
- Growing model support

```toml
[dependencies]
candle-core = "0.4"
candle-transformers = "0.4"
candle-nn = "0.4"
```

Considerations:
- Newer, less mature
- Fewer pre-built model integrations
- Better for resource-constrained environments

**Option 3: External API**
- OpenAI embeddings API
- Anthropic embeddings (when available)
- Cohere, Voyage AI, etc.

```toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
```

Considerations:
- No local model management
- Requires API keys and network
- Per-request costs
- Privacy implications for document content

### Vector Storage

**Option 1: qdrant-client** (recommended for scale)
- Purpose-built vector database
- Supports filtering and metadata
- Can run embedded or as service

```toml
[dependencies]
qdrant-client = "1.7"
```

Considerations:
- Additional service to manage if not embedded
- Excellent query performance
- Rich filtering capabilities

**Option 2: hora** (embedded, pure Rust)
- Lightweight vector search library
- No external dependencies
- Good for smaller collections

```toml
[dependencies]
hora = "0.1"
```

Considerations:
- Limited to ~100K vectors efficiently
- No persistence (must rebuild from source)
- Simple API

**Option 3: SQLite with sqlite-vss**
- Extension for SQLite vector search
- Keeps everything in one database
- Reasonable performance for moderate scale

```toml
[dependencies]
rusqlite = { version = "0.31", features = ["bundled", "vtab"] }
```

Considerations:
- Requires VSS extension compiled in
- Performance degrades past ~50K vectors
- Simpler deployment (single file)

## Recommended Configuration

For Lattice, the recommended approach balances simplicity with capability:

**Embedding**: rust-bert with `all-MiniLM-L6-v2` model
- Good quality/size tradeoff
- 384-dimensional vectors
- Fast inference

**Storage**: SQLite with custom FTS5 + vector hybrid
- Primary: FTS5 for keyword search
- Secondary: Embedded vector store for semantic
- Hybrid ranking combines both scores

## Implementation Strategy

### Phase 1: Keyword-Only (Initial)

Use SQLite FTS5 for full-text search:

```sql
SELECT document_id, rank
FROM fts_content
WHERE body MATCH ?
ORDER BY rank
```

This provides immediate search capability without ML complexity.

### Phase 2: Hybrid Search (Future)

Add semantic search alongside keyword:

1. Generate embeddings during index reconciliation
2. Store vectors in dedicated table or extension
3. Query combines FTS5 rank with cosine similarity
4. Configurable weighting between keyword and semantic

### Embedding Generation

Embeddings generated:
- On document creation/modification
- During full index rebuild
- In background task (not blocking user commands)

Storage:
- Separate `embeddings` table or file
- Keyed by document ID
- Re-embedded when body content changes (via hash comparison)

## CLI Interface

### Search Command

```
lat search "query text" [options]
```

**Options:**
- `--mode keyword|semantic|hybrid`: Search mode (default: hybrid)
- `--limit N`: Maximum results (default: 20)
- `--threshold F`: Minimum similarity score (0.0-1.0)
- `--path <prefix>`: Restrict to documents under path
- `--type <type>`: Filter by issue type
- `--status <status>`: Filter by status

**Output:**
```
Found 5 documents matching "authentication timeout":

LXXXX  0.92  auth-timeout-handling     bug/P1/open
LYYYY  0.87  authentication-design     doc
LZZZZ  0.81  session-management        doc
LWWWW  0.76  login-bug-2024-01        bug/closed
LAAAA  0.71  api-error-codes          doc
```

### Similar Command

```
lat similar <id> [options]
```

Find documents semantically similar to a given document.

**Options:**
- `--limit N`: Maximum results (default: 10)
- `--threshold F`: Minimum similarity

**Output:**
```
Documents similar to LXXXX (auth-timeout-handling):

LYYYY  0.89  authentication-design
LZZZZ  0.82  session-timeout-config
LWWWW  0.78  api-authentication
```

## Resource Requirements

### Model Size

| Model | Dimensions | Size | Quality |
|-------|-----------|------|---------|
| all-MiniLM-L6-v2 | 384 | ~80MB | Good |
| all-mpnet-base-v2 | 768 | ~420MB | Better |
| e5-large-v2 | 1024 | ~1.3GB | Best |

Recommendation: Start with MiniLM for balance.

### Storage Overhead

Per document:
- 384 dimensions × 4 bytes = 1.5KB per embedding
- 10,000 documents ≈ 15MB additional storage

### Performance

Embedding generation:
- MiniLM: ~50ms per document (CPU)
- With GPU: ~5ms per document

Search:
- Keyword (FTS5): <10ms for 10K documents
- Vector similarity: <50ms for 10K documents
- Hybrid: <100ms total

## Privacy Considerations

If using external embedding APIs:
- Document content sent to third party
- Consider content sensitivity
- May need data processing agreements
- Offer opt-out or local-only mode

For sensitive repositories, local embedding (rust-bert/candle) is preferred.

## Future Enhancements

1. **Query expansion**: Automatically expand queries with synonyms
2. **Relevance feedback**: Learn from user clicks/selections
3. **Clustering**: Group similar documents automatically
4. **Cross-reference suggestions**: "You might also want to link to..."
