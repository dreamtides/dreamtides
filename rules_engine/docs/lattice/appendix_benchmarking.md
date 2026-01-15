# Appendix: Performance Benchmarking

## Overview

Lattice uses [criterion](https://docs.rs/criterion) for performance benchmarking,
measuring latency of common operations to detect regressions and guide optimization.

## Benchmark Categories

**Index Operations:**
- Full index rebuild from git repository
- Incremental reconciliation after file changes
- Document lookup by ID, name, and path
- FTS5 full-text search queries

**Document Operations:**
- Parsing YAML frontmatter and markdown body
- Link extraction and normalization
- `lat fmt` on single documents and directories

**Query Operations:**
- `lat list` with various filter combinations
- `lat ready` task filtering
- `lat overview` ranking algorithm

## Implementation

Benchmarks live in `rules_engine/benches/lattice/` with one file per category:

```rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn index_rebuild(c: &mut Criterion) {
    let repo = setup_test_repo(100); // 100 documents
    c.bench_function("index_rebuild_100", |b| {
        b.iter(|| Index::rebuild(&repo))
    });
}

fn index_rebuild_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("index_rebuild");
    for size in [10, 100, 500, 1000] {
        let repo = setup_test_repo(size);
        group.bench_with_input(BenchmarkId::from_parameter(size), &repo, |b, repo| {
            b.iter(|| Index::rebuild(repo))
        });
    }
    group.finish();
}

criterion_group!(benches, index_rebuild, index_rebuild_scaling);
criterion_main!(benches);
```

## Running Benchmarks

```bash
just bench-lattice              # Run all lattice benchmarks
just bench-lattice -- index     # Run only index benchmarks
```

Criterion generates HTML reports in `target/criterion/` with historical comparisons.
