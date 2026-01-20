//! Benchmarks for index operations.
//!
//! Measures performance of:
//! - Full index rebuild at various scales
//! - Incremental reconciliation after file changes
//! - Document lookups by ID, name, and path
//! - FTS5 full-text search queries
//!
//! Run with:
//! ```bash
//! just bench-lattice -- index_rebuild
//! just bench-lattice -- incremental_sync
//! just bench-lattice -- document_lookup
//! just bench-lattice -- fts_search
//! ```

use std::time::Duration;

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use lattice::index::reconciliation::change_detection::ChangeInfo;
use lattice::index::reconciliation::{reconciliation_coordinator, sync_strategies};
use lattice::index::{connection_pool, document_queries, fulltext_search, schema_definition};
use lattice_benchmarks::test_repo::{IndexedRepo, list_markdown_files, setup_indexed_repo};
use tracing::Level;

/// Benchmark full index rebuild from scratch at different scales.
///
/// Measures the time to parse all documents and populate the SQLite index.
/// Tests at 10, 100, 500, and 1000 documents to identify scaling behavior.
fn full_index_rebuild(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_index_rebuild");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        for size in [10, 100, 500, 1000] {
            group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
                b.iter_batched(
                    || setup_indexed_repo(size),
                    |IndexedRepo { repo, fake_git, conn: _conn }| {
                        // Create a fresh connection to measure rebuild from scratch
                        let conn = connection_pool::open_memory_connection()
                            .expect("Failed to open memory connection");
                        schema_definition::create_schema(&conn).expect("Failed to create schema");

                        criterion::black_box(reconciliation_coordinator::reconcile(
                            &repo.root, &fake_git, &conn,
                        ))
                    },
                    BatchSize::SmallInput,
                );
            });
        }
    });

    group.finish();
}

/// Benchmark incremental reconciliation after single file change.
///
/// Measures the time to sync after modifying one document in an indexed
/// repository, comparing against full rebuild overhead.
fn incremental_sync_single_file(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_sync_single_file");
    group.measurement_time(Duration::from_secs(10));

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        for size in [100, 500] {
            group.bench_with_input(BenchmarkId::new("base_docs", size), &size, |b, &size| {
                b.iter_batched(
                    || {
                        let IndexedRepo { repo, fake_git, conn } = setup_indexed_repo(size);
                        // Pick a file to "modify"
                        let files = list_markdown_files(&repo.root);
                        let modified_file = files.first().cloned().unwrap_or_default();
                        let relative_path = modified_file
                            .strip_prefix(&repo.root)
                            .map(|p| p.to_path_buf())
                            .unwrap_or_default();

                        let change_info = ChangeInfo {
                            modified_files: vec![relative_path],
                            deleted_files: vec![],
                            uncommitted_files: vec![],
                            current_head: Some("abc123".to_string()),
                            last_indexed_commit: Some("abc122".to_string()),
                        };
                        (repo, fake_git, conn, change_info)
                    },
                    |(repo, fake_git, conn, change_info)| {
                        criterion::black_box(sync_strategies::incremental_sync(
                            &repo.root,
                            &fake_git,
                            &conn,
                            &change_info,
                        ))
                    },
                    BatchSize::SmallInput,
                );
            });
        }
    });

    group.finish();
}

/// Benchmark incremental reconciliation with multiple file changes.
///
/// Tests sync performance after 10, 50, and 100 file changes to find
/// the break-even point where full rebuild becomes more efficient.
fn incremental_sync_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_sync_batch");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(20);

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let base_size = 500;

        for change_count in [10, 50, 100] {
            group.bench_with_input(
                BenchmarkId::new("changed_files", change_count),
                &change_count,
                |b, &change_count| {
                    b.iter_batched(
                        || {
                            let IndexedRepo { repo, fake_git, conn } =
                                setup_indexed_repo(base_size);
                            let files = list_markdown_files(&repo.root);

                            // Select files to "modify"
                            let modified_files: Vec<_> = files
                                .iter()
                                .take(change_count)
                                .filter_map(|f| f.strip_prefix(&repo.root).ok())
                                .map(|p| p.to_path_buf())
                                .collect();

                            let change_info = ChangeInfo {
                                modified_files,
                                deleted_files: vec![],
                                uncommitted_files: vec![],
                                current_head: Some("abc123".to_string()),
                                last_indexed_commit: Some("abc122".to_string()),
                            };
                            (repo, fake_git, conn, change_info)
                        },
                        |(repo, fake_git, conn, change_info)| {
                            criterion::black_box(sync_strategies::incremental_sync(
                                &repo.root,
                                &fake_git,
                                &conn,
                                &change_info,
                            ))
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
        }
    });

    group.finish();
}

/// Benchmark document lookup by Lattice ID (primary key).
fn lookup_by_id(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_by_id");

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let IndexedRepo { repo, fake_git: _fake_git, conn } = setup_indexed_repo(500);

        // Get some document IDs to look up
        let ids: Vec<_> = repo.document_ids.iter().take(100).cloned().collect();

        // Single lookup
        if let Some(id) = ids.first() {
            let id = id.clone();
            group.bench_function("single", |b| {
                b.iter(|| criterion::black_box(document_queries::lookup_by_id(&conn, &id)))
            });
        }

        // Batch lookup (100 documents)
        group.bench_function("batch_100", |b| {
            b.iter(|| {
                for id in &ids {
                    criterion::black_box(document_queries::lookup_by_id(&conn, id)).ok();
                }
            })
        });
    });

    group.finish();
}

/// Benchmark document lookup by file path.
fn lookup_by_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_by_path");

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let IndexedRepo { repo, fake_git: _fake_git, conn } = setup_indexed_repo(500);
        let files = list_markdown_files(&repo.root);

        // Collect relative paths for lookup
        let paths: Vec<String> = files
            .iter()
            .take(100)
            .filter_map(|f| f.strip_prefix(&repo.root).ok())
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        // Single lookup
        if let Some(path) = paths.first() {
            let path = path.clone();
            group.bench_function("single", |b| {
                b.iter(|| criterion::black_box(document_queries::lookup_by_path(&conn, &path)))
            });
        }

        // Batch lookup (100 documents)
        group.bench_function("batch_100", |b| {
            b.iter(|| {
                for path in &paths {
                    criterion::black_box(document_queries::lookup_by_path(&conn, path)).ok();
                }
            })
        });
    });

    group.finish();
}

/// Benchmark document lookup by name.
fn lookup_by_name(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_by_name");

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let IndexedRepo { repo: _repo, fake_git: _fake_git, conn } = setup_indexed_repo(500);

        // Use known name patterns from generated repos
        let names = vec![
            "api".to_string(),
            "database".to_string(),
            "api-task-0".to_string(),
            "database-doc-0".to_string(),
        ];

        // Single lookup
        group.bench_function("single", |b| {
            b.iter(|| criterion::black_box(document_queries::lookup_by_name(&conn, "api")))
        });

        // Multiple lookups
        group.bench_function("batch_4", |b| {
            b.iter(|| {
                for name in &names {
                    criterion::black_box(document_queries::lookup_by_name(&conn, name)).ok();
                }
            })
        });
    });

    group.finish();
}

/// Benchmark FTS5 simple single-word search.
fn fts_single_word(c: &mut Criterion) {
    let mut group = c.benchmark_group("fts_single_word");

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let IndexedRepo { repo: _repo, fake_git: _fake_git, conn } = setup_indexed_repo(500);

        // Search for common words that appear in generated documents
        let queries = vec!["overview", "module", "documentation", "task", "requirements"];

        group.bench_function("single_query", |b| {
            b.iter(|| criterion::black_box(fulltext_search::search(&conn, "overview")))
        });

        group.bench_function("multiple_queries", |b| {
            b.iter(|| {
                for query in &queries {
                    criterion::black_box(fulltext_search::search(&conn, query)).ok();
                }
            })
        });
    });

    group.finish();
}

/// Benchmark FTS5 multi-word phrase search.
fn fts_phrase_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("fts_phrase_search");

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let IndexedRepo { repo: _repo, fake_git: _fake_git, conn } = setup_indexed_repo(500);

        // Phrase queries using FTS5 syntax
        let phrase_queries =
            vec!["\"knowledge base\"", "\"module root\"", "\"documentation content\""];

        group.bench_function("single_phrase", |b| {
            b.iter(|| criterion::black_box(fulltext_search::search(&conn, "\"module root\"")))
        });

        group.bench_function("multiple_phrases", |b| {
            b.iter(|| {
                for query in &phrase_queries {
                    criterion::black_box(fulltext_search::search(&conn, query)).ok();
                }
            })
        });
    });

    group.finish();
}

/// Benchmark FTS5 search with result limit.
fn fts_with_limit(c: &mut Criterion) {
    let mut group = c.benchmark_group("fts_with_limit");

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let IndexedRepo { repo: _repo, fake_git: _fake_git, conn } = setup_indexed_repo(500);

        // Compare limited vs unlimited search
        group.bench_function("unlimited", |b| {
            b.iter(|| criterion::black_box(fulltext_search::search(&conn, "module")))
        });

        group.bench_function("limit_10", |b| {
            b.iter(|| {
                criterion::black_box(fulltext_search::search_with_limit(&conn, "module", Some(10)))
            })
        });

        group.bench_function("limit_50", |b| {
            b.iter(|| {
                criterion::black_box(fulltext_search::search_with_limit(&conn, "module", Some(50)))
            })
        });
    });

    group.finish();
}

/// Benchmark FTS5 search with boolean operators.
fn fts_boolean_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("fts_boolean_search");

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let IndexedRepo { repo: _repo, fake_git: _fake_git, conn } = setup_indexed_repo(500);

        // Boolean queries using FTS5 syntax
        group.bench_function("and_query", |b| {
            b.iter(|| criterion::black_box(fulltext_search::search(&conn, "module AND overview")))
        });

        group.bench_function("or_query", |b| {
            b.iter(|| criterion::black_box(fulltext_search::search(&conn, "task OR documentation")))
        });

        group.bench_function("not_query", |b| {
            b.iter(|| criterion::black_box(fulltext_search::search(&conn, "module NOT task")))
        });

        group.bench_function("prefix_query", |b| {
            b.iter(|| criterion::black_box(fulltext_search::search(&conn, "docu*")))
        });
    });

    group.finish();
}

criterion_group!(
    index_benches,
    full_index_rebuild,
    incremental_sync_single_file,
    incremental_sync_batch,
    lookup_by_id,
    lookup_by_path,
    lookup_by_name,
    fts_single_word,
    fts_phrase_search,
    fts_with_limit,
    fts_boolean_search,
);
criterion_main!(index_benches);
