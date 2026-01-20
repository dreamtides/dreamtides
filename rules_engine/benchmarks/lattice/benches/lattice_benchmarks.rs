//! Performance benchmarks for the Lattice document management system.
//!
//! Run all benchmarks:
//! ```bash
//! just bench-lattice
//! ```
//!
//! Run specific benchmark group:
//! ```bash
//! just bench-lattice -- index_rebuild
//! just bench-lattice -- document_parsing
//! ```

use std::time::Duration;

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use lattice::document::frontmatter_parser;
use lattice::index::reconciliation::reconciliation_coordinator;
use lattice::index::{connection_pool, schema_definition};
use lattice::test::fake_git::FakeGit;
use lattice_benchmarks::test_repo::{RepoConfig, generate_repo, list_markdown_files};
use tracing::Level;

/// Benchmark full index rebuild from scratch.
///
/// Measures the time to parse all documents and populate the SQLite index
/// when starting from an empty database.
fn index_rebuild(c: &mut Criterion) {
    let mut group = c.benchmark_group("index_rebuild");
    group.measurement_time(Duration::from_secs(10));

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        // Benchmark different repository sizes
        for size in [10, 100, 500] {
            group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
                b.iter_batched(
                    || {
                        let config = RepoConfig::with_document_count(size);
                        let repo = generate_repo(&config);
                        let fake_git = FakeGit::new();

                        // Track all markdown files with fake git
                        let files = list_markdown_files(&repo.root);
                        for file in &files {
                            if let Ok(relative) = file.strip_prefix(&repo.root) {
                                fake_git.track_file(relative);
                            }
                        }

                        (repo, fake_git)
                    },
                    |(repo, fake_git)| {
                        // Create a new in-memory database for each iteration
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

/// Benchmark document parsing (frontmatter extraction and YAML parsing).
///
/// Measures the time to parse individual document files into structured
/// frontmatter data.
fn document_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("document_parsing");
    group.measurement_time(Duration::from_secs(5));

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        // Generate a repo once and read files into memory for parsing benchmarks
        let config = RepoConfig::with_document_count(100);
        let repo = generate_repo(&config);
        let files = list_markdown_files(&repo.root);

        // Read file contents into memory to isolate parsing performance
        let file_contents: Vec<(std::path::PathBuf, String)> = files
            .iter()
            .filter_map(|path| {
                std::fs::read_to_string(path).ok().map(|content| (path.clone(), content))
            })
            .collect();

        // Benchmark parsing a single document
        if let Some((path, content)) = file_contents.first() {
            group.bench_function("single_document", |b| {
                b.iter(|| criterion::black_box(frontmatter_parser::parse(content, path)));
            });
        }

        // Benchmark parsing all documents (batch operation)
        group.bench_function("batch_100_documents", |b| {
            b.iter(|| {
                for (path, content) in &file_contents {
                    criterion::black_box(frontmatter_parser::parse(content, path)).ok();
                }
            });
        });
    });

    group.finish();
}

/// Benchmark index rebuild scaling behavior.
///
/// Tests how rebuild time scales with repository size to identify
/// potential O(n²) algorithms or other scaling issues.
fn index_rebuild_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("index_rebuild_scaling");
    group.sample_size(10); // Fewer samples for large repos
    group.measurement_time(Duration::from_secs(15));

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        for size in [10, 50, 100, 250, 500, 1000] {
            group.bench_with_input(BenchmarkId::new("documents", size), &size, |b, &size| {
                b.iter_batched(
                    || {
                        let config = RepoConfig::with_document_count(size);
                        let repo = generate_repo(&config);
                        let fake_git = FakeGit::new();

                        let files = list_markdown_files(&repo.root);
                        for file in &files {
                            if let Ok(relative) = file.strip_prefix(&repo.root) {
                                fake_git.track_file(relative);
                            }
                        }

                        (repo, fake_git)
                    },
                    |(repo, fake_git)| {
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

/// Benchmark SQLite connection setup and schema creation.
///
/// Measures the overhead of creating a new database connection and
/// initializing the schema, which happens at the start of every command.
fn connection_setup(c: &mut Criterion) {
    let mut group = c.benchmark_group("connection_setup");

    group.bench_function("memory_connection", |b| {
        b.iter(|| {
            let conn = connection_pool::open_memory_connection()
                .expect("Failed to open memory connection");
            criterion::black_box(conn)
        });
    });

    group.bench_function("schema_creation", |b| {
        b.iter(|| {
            let conn = connection_pool::open_memory_connection()
                .expect("Failed to open memory connection");
            schema_definition::create_schema(&conn).expect("Failed to create schema");
            criterion::black_box(conn)
        });
    });

    group.finish();
}

criterion_group!(benches, index_rebuild, document_parsing, index_rebuild_scaling, connection_setup,);
criterion_main!(benches);
