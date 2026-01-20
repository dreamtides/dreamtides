//! Benchmarks for query operations.
//!
//! Measures performance of:
//! - `lat list` filtering with various criteria
//! - `lat ready` task filtering and claim resolution
//! - `lat overview` ranking algorithm
//!
//! Run with:
//! ```bash
//! just bench-lattice -- list_filtering
//! just bench-lattice -- ready_calculation
//! just bench-lattice -- overview_ranking
//! ```

use std::cmp::Ordering;
use std::time::Duration;

use chrono::{Days, Utc};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use lattice::config::config_schema::OverviewConfig;
use lattice::document::frontmatter_schema::TaskType;
use lattice::index::document_filter::{DocumentFilter, DocumentState, SortColumn, SortOrder};
use lattice::index::{document_queries, view_tracking};
use lattice::task::ready_calculator::{self, ReadyFilter, ReadySortPolicy};
use lattice_benchmarks::test_repo::{
    IndexedRepo, RepoConfig, TestRepo, list_markdown_files, setup_indexed_repo,
    setup_indexed_repo_with_config,
};
use tracing::Level;

// =============================================================================
// List Filtering Benchmarks
// =============================================================================

/// Benchmark list query with no filters (return all documents).
fn list_no_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("list_no_filter");

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        for size in [100, 500, 1000] {
            let IndexedRepo { repo: _repo, fake_git: _fake_git, conn } = setup_indexed_repo(size);

            group.bench_with_input(BenchmarkId::new("docs", size), &conn, |b, conn| {
                b.iter(|| {
                    let filter = DocumentFilter::including_closed();
                    criterion::black_box(document_queries::query(conn, &filter))
                })
            });
        }
    });

    group.finish();
}

/// Benchmark list query with single filter criterion.
fn list_single_criterion(c: &mut Criterion) {
    let mut group = c.benchmark_group("list_single_criterion");

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let IndexedRepo { repo: _repo, fake_git: _fake_git, conn } = setup_indexed_repo(500);

        // Filter by task type
        group.bench_function("type_bug", |b| {
            b.iter(|| {
                let filter = DocumentFilter::new().with_task_type(TaskType::Bug);
                criterion::black_box(document_queries::query(&conn, &filter))
            })
        });

        // Filter by priority
        group.bench_function("priority_0", |b| {
            b.iter(|| {
                let filter = DocumentFilter::new().with_priority(0);
                criterion::black_box(document_queries::query(&conn, &filter))
            })
        });

        // Filter by state (open)
        group.bench_function("state_open", |b| {
            b.iter(|| {
                let filter = DocumentFilter::new().with_state(DocumentState::Open);
                criterion::black_box(document_queries::query(&conn, &filter))
            })
        });

        // Filter by state (closed)
        group.bench_function("state_closed", |b| {
            b.iter(|| {
                let filter = DocumentFilter::including_closed().with_state(DocumentState::Closed);
                criterion::black_box(document_queries::query(&conn, &filter))
            })
        });
    });

    group.finish();
}

/// Benchmark list query with multiple filter criteria combined.
fn list_multiple_criteria(c: &mut Criterion) {
    let mut group = c.benchmark_group("list_multiple_criteria");

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let IndexedRepo { repo: _repo, fake_git: _fake_git, conn } = setup_indexed_repo(500);

        // Two criteria
        group.bench_function("type_and_priority", |b| {
            b.iter(|| {
                let filter =
                    DocumentFilter::new().with_task_type(TaskType::Bug).with_priority_range(0, 2);
                criterion::black_box(document_queries::query(&conn, &filter))
            })
        });

        // Three criteria
        group.bench_function("type_priority_state", |b| {
            b.iter(|| {
                let filter = DocumentFilter::new()
                    .with_task_type(TaskType::Feature)
                    .with_priority_range(0, 3)
                    .with_state(DocumentState::Open);
                criterion::black_box(document_queries::query(&conn, &filter))
            })
        });

        // Four criteria with sort
        group.bench_function("complex_with_sort", |b| {
            b.iter(|| {
                let filter = DocumentFilter::new()
                    .with_task_type(TaskType::Task)
                    .with_priority_range(1, 3)
                    .with_state(DocumentState::Open)
                    .with_in_tasks_dir(true)
                    .sort_by(SortColumn::Priority)
                    .sort_order(SortOrder::Ascending)
                    .limit(50);
                criterion::black_box(document_queries::query(&conn, &filter))
            })
        });
    });

    group.finish();
}

/// Benchmark list query with path prefix filter (hierarchical filtering).
fn list_path_prefix(c: &mut Criterion) {
    let mut group = c.benchmark_group("list_path_prefix");

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let IndexedRepo { repo: _repo, fake_git: _fake_git, conn } = setup_indexed_repo(500);

        // Filter by module directory
        group.bench_function("module_api", |b| {
            b.iter(|| {
                let filter = DocumentFilter::new().with_path_prefix("api/");
                criterion::black_box(document_queries::query(&conn, &filter))
            })
        });

        // Filter by tasks subdirectory
        group.bench_function("module_tasks", |b| {
            b.iter(|| {
                let filter = DocumentFilter::new().with_path_prefix("database/tasks/");
                criterion::black_box(document_queries::query(&conn, &filter))
            })
        });

        // Filter by docs subdirectory
        group.bench_function("module_docs", |b| {
            b.iter(|| {
                let filter = DocumentFilter::new().with_path_prefix("auth/docs/");
                criterion::black_box(document_queries::query(&conn, &filter))
            })
        });
    });

    group.finish();
}

/// Benchmark list query with label filters.
fn list_label_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("list_label_filter");

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let IndexedRepo { repo: _repo, fake_git: _fake_git, conn } = setup_indexed_repo(500);

        // Single label filter
        group.bench_function("single_label", |b| {
            b.iter(|| {
                let filter = DocumentFilter::new().with_labels_all(vec!["urgent".to_string()]);
                criterion::black_box(document_queries::query(&conn, &filter))
            })
        });

        // Multiple labels with AND semantics
        group.bench_function("labels_and", |b| {
            b.iter(|| {
                let filter = DocumentFilter::new()
                    .with_labels_all(vec!["backend".to_string(), "security".to_string()]);
                criterion::black_box(document_queries::query(&conn, &filter))
            })
        });

        // Multiple labels with OR semantics
        group.bench_function("labels_any", |b| {
            b.iter(|| {
                let filter = DocumentFilter::new().with_labels_any(vec![
                    "urgent".to_string(),
                    "performance".to_string(),
                    "refactor".to_string(),
                ]);
                criterion::black_box(document_queries::query(&conn, &filter))
            })
        });
    });

    group.finish();
}

/// Benchmark list query with date range filters.
fn list_date_range(c: &mut Criterion) {
    let mut group = c.benchmark_group("list_date_range");

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let IndexedRepo { repo: _repo, fake_git: _fake_git, conn } = setup_indexed_repo(500);

        let now = Utc::now();
        let week_ago = now.checked_sub_days(Days::new(7)).unwrap_or(now);
        let month_ago = now.checked_sub_days(Days::new(30)).unwrap_or(now);

        // Created after date
        group.bench_function("created_after", |b| {
            b.iter(|| {
                let filter = DocumentFilter::new().with_created_after(week_ago);
                criterion::black_box(document_queries::query(&conn, &filter))
            })
        });

        // Updated after date
        group.bench_function("updated_after", |b| {
            b.iter(|| {
                let filter = DocumentFilter::new().with_updated_after(week_ago);
                criterion::black_box(document_queries::query(&conn, &filter))
            })
        });

        // Date range (between)
        group.bench_function("created_range", |b| {
            b.iter(|| {
                let filter =
                    DocumentFilter::new().with_created_after(month_ago).with_created_before(now);
                criterion::black_box(document_queries::query(&conn, &filter))
            })
        });

        // Combined date and other filters
        group.bench_function("date_and_type", |b| {
            b.iter(|| {
                let filter = DocumentFilter::new()
                    .with_created_after(month_ago)
                    .with_task_type(TaskType::Bug);
                criterion::black_box(document_queries::query(&conn, &filter))
            })
        });
    });

    group.finish();
}

// =============================================================================
// Ready Calculation Benchmarks
// =============================================================================

/// Benchmark basic ready task calculation (not closed, not blocked, not P4).
fn ready_basic(c: &mut Criterion) {
    let mut group = c.benchmark_group("ready_basic");
    group.measurement_time(Duration::from_secs(10));

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        for size in [100, 500, 1000] {
            let IndexedRepo { repo, fake_git: _fake_git, conn } = setup_indexed_repo(size);

            group.bench_with_input(BenchmarkId::new("tasks", size), &size, |b, _| {
                b.iter(|| {
                    let filter = ReadyFilter::new();
                    criterion::black_box(ready_calculator::query_ready_tasks(
                        &conn, &repo.root, &filter,
                    ))
                })
            });
        }
    });

    group.finish();
}

/// Benchmark ready calculation with blocker resolution.
fn ready_with_blockers(c: &mut Criterion) {
    let mut group = c.benchmark_group("ready_with_blockers");
    group.measurement_time(Duration::from_secs(10));

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        // Use high dependency fraction for blocker resolution testing
        let config = RepoConfig {
            document_count: 500,
            task_fraction: 0.8,
            dependency_fraction: 0.6,
            label_fraction: 0.3,
            link_fraction: 0.2,
            avg_links_per_doc: 2,
        };

        let IndexedRepo { repo, fake_git, conn } = setup_indexed_repo_with_config(&config);

        // Track files and rebuild index to ensure blockers are properly indexed
        let files = list_markdown_files(&repo.root);
        for file in &files {
            if let Ok(relative) = file.strip_prefix(&repo.root) {
                fake_git.track_file(relative);
            }
        }

        group.bench_function("high_dependency", |b| {
            b.iter(|| {
                let filter = ReadyFilter::new();
                criterion::black_box(ready_calculator::query_ready_tasks(
                    &conn, &repo.root, &filter,
                ))
            })
        });

        // With include backlog (P4)
        group.bench_function("include_backlog", |b| {
            b.iter(|| {
                let filter = ReadyFilter::new().with_include_backlog();
                criterion::black_box(ready_calculator::query_ready_tasks(
                    &conn, &repo.root, &filter,
                ))
            })
        });
    });

    group.finish();
}

/// Benchmark ready calculation with different sort policies.
fn ready_sort_policies(c: &mut Criterion) {
    let mut group = c.benchmark_group("ready_sort_policies");

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let IndexedRepo { repo, fake_git: _fake_git, conn } = setup_indexed_repo(500);

        group.bench_function("hybrid", |b| {
            b.iter(|| {
                let filter = ReadyFilter::new().with_sort_policy(ReadySortPolicy::Hybrid);
                criterion::black_box(ready_calculator::query_ready_tasks(
                    &conn, &repo.root, &filter,
                ))
            })
        });

        group.bench_function("priority", |b| {
            b.iter(|| {
                let filter = ReadyFilter::new().with_sort_policy(ReadySortPolicy::Priority);
                criterion::black_box(ready_calculator::query_ready_tasks(
                    &conn, &repo.root, &filter,
                ))
            })
        });

        group.bench_function("oldest", |b| {
            b.iter(|| {
                let filter = ReadyFilter::new().with_sort_policy(ReadySortPolicy::Oldest);
                criterion::black_box(ready_calculator::query_ready_tasks(
                    &conn, &repo.root, &filter,
                ))
            })
        });
    });

    group.finish();
}

/// Benchmark ready count (without fetching full documents).
fn ready_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("ready_count");

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        for size in [100, 500, 1000] {
            let IndexedRepo { repo: _repo, fake_git: _fake_git, conn } = setup_indexed_repo(size);

            group.bench_with_input(BenchmarkId::new("tasks", size), &conn, |b, conn| {
                b.iter(|| {
                    let filter = ReadyFilter::new();
                    criterion::black_box(ready_calculator::count_ready_tasks(conn, &filter))
                })
            });
        }
    });

    group.finish();
}

/// Benchmark ready calculation with path filtering.
fn ready_path_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("ready_path_filter");

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let IndexedRepo { repo, fake_git: _fake_git, conn } = setup_indexed_repo(500);

        group.bench_function("module_filter", |b| {
            b.iter(|| {
                let filter = ReadyFilter::new().with_path_prefix("api/");
                criterion::black_box(ready_calculator::query_ready_tasks(
                    &conn, &repo.root, &filter,
                ))
            })
        });

        group.bench_function("tasks_subdir", |b| {
            b.iter(|| {
                let filter = ReadyFilter::new().with_path_prefix("database/tasks/");
                criterion::black_box(ready_calculator::query_ready_tasks(
                    &conn, &repo.root, &filter,
                ))
            })
        });
    });

    group.finish();
}

// =============================================================================
// Overview Ranking Benchmarks
// =============================================================================

/// Benchmark repository-level overview scoring.
fn overview_repository_level(c: &mut Criterion) {
    let mut group = c.benchmark_group("overview_repository_level");
    group.measurement_time(Duration::from_secs(10));

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        for size in [100, 500, 1000] {
            let IndexedRepo { repo, fake_git: _fake_git, conn } = setup_indexed_repo(size);
            let config = OverviewConfig::default();

            // Simulate some view activity
            simulate_view_activity(&conn, &repo, 0.3);

            group.bench_with_input(BenchmarkId::new("docs", size), &size, |b, _| {
                b.iter(|| {
                    let filter = DocumentFilter::new();
                    let docs = document_queries::query(&conn, &filter).unwrap_or_default();
                    criterion::black_box(score_documents(&conn, docs, &config))
                })
            });
        }
    });

    group.finish();
}

/// Benchmark overview with different view count distributions.
fn overview_view_distributions(c: &mut Criterion) {
    let mut group = c.benchmark_group("overview_view_distributions");

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let IndexedRepo { repo, fake_git: _fake_git, conn } = setup_indexed_repo(500);
        let config = OverviewConfig::default();

        // Low view activity (10% of docs have views)
        group.bench_function("low_views", |b| {
            // Reset views and simulate low activity
            let _ = view_tracking::reset_all_views(&conn);
            simulate_view_activity(&conn, &repo, 0.1);

            b.iter(|| {
                let filter = DocumentFilter::new();
                let docs = document_queries::query(&conn, &filter).unwrap_or_default();
                criterion::black_box(score_documents(&conn, docs, &config))
            })
        });

        // Medium view activity (50% of docs have views)
        group.bench_function("medium_views", |b| {
            let _ = view_tracking::reset_all_views(&conn);
            simulate_view_activity(&conn, &repo, 0.5);

            b.iter(|| {
                let filter = DocumentFilter::new();
                let docs = document_queries::query(&conn, &filter).unwrap_or_default();
                criterion::black_box(score_documents(&conn, docs, &config))
            })
        });

        // High view activity (90% of docs have views)
        group.bench_function("high_views", |b| {
            let _ = view_tracking::reset_all_views(&conn);
            simulate_view_activity(&conn, &repo, 0.9);

            b.iter(|| {
                let filter = DocumentFilter::new();
                let docs = document_queries::query(&conn, &filter).unwrap_or_default();
                criterion::black_box(score_documents(&conn, docs, &config))
            })
        });
    });

    group.finish();
}

/// Benchmark overview ranking with path filtering.
fn overview_with_path_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("overview_with_path_filter");

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let IndexedRepo { repo, fake_git: _fake_git, conn } = setup_indexed_repo(500);
        let config = OverviewConfig::default();

        simulate_view_activity(&conn, &repo, 0.3);

        // Full repository
        group.bench_function("full_repo", |b| {
            b.iter(|| {
                let filter = DocumentFilter::new();
                let docs = document_queries::query(&conn, &filter).unwrap_or_default();
                criterion::black_box(score_documents(&conn, docs, &config))
            })
        });

        // Single module
        group.bench_function("single_module", |b| {
            b.iter(|| {
                let filter = DocumentFilter::new().with_path_prefix("api/");
                let docs = document_queries::query(&conn, &filter).unwrap_or_default();
                criterion::black_box(score_documents(&conn, docs, &config))
            })
        });

        // With type filter
        group.bench_function("with_type_filter", |b| {
            b.iter(|| {
                let filter = DocumentFilter::new().with_task_type(TaskType::Bug);
                let docs = document_queries::query(&conn, &filter).unwrap_or_default();
                criterion::black_box(score_documents(&conn, docs, &config))
            })
        });
    });

    group.finish();
}

/// Benchmark overview result limiting.
fn overview_with_limit(c: &mut Criterion) {
    let mut group = c.benchmark_group("overview_with_limit");

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let IndexedRepo { repo, fake_git: _fake_git, conn } = setup_indexed_repo(1000);
        let config = OverviewConfig::default();

        simulate_view_activity(&conn, &repo, 0.5);

        // No limit (score all)
        group.bench_function("no_limit", |b| {
            b.iter(|| {
                let filter = DocumentFilter::new();
                let docs = document_queries::query(&conn, &filter).unwrap_or_default();
                criterion::black_box(score_documents(&conn, docs, &config))
            })
        });

        // Limit 10
        group.bench_function("limit_10", |b| {
            b.iter(|| {
                let filter = DocumentFilter::new();
                let docs = document_queries::query(&conn, &filter).unwrap_or_default();
                let scored = score_documents(&conn, docs, &config);
                criterion::black_box(scored.into_iter().take(10).collect::<Vec<_>>())
            })
        });

        // Limit 50
        group.bench_function("limit_50", |b| {
            b.iter(|| {
                let filter = DocumentFilter::new();
                let docs = document_queries::query(&conn, &filter).unwrap_or_default();
                let scored = score_documents(&conn, docs, &config);
                criterion::black_box(scored.into_iter().take(50).collect::<Vec<_>>())
            })
        });
    });

    group.finish();
}

// =============================================================================
// Helpers
// =============================================================================

/// Simulates view activity on a portion of documents.
fn simulate_view_activity(conn: &rusqlite::Connection, repo: &TestRepo, fraction: f64) {
    let view_count = (repo.document_ids.len() as f64 * fraction).ceil() as usize;
    for id in repo.document_ids.iter().take(view_count) {
        // Simulate 1-10 views per document
        let views = 1 + fastrand::usize(0..10);
        for _ in 0..views {
            let _ = view_tracking::record_view(conn, id);
        }
    }
}

/// Document with computed score for ranking.
struct ScoredDocument {
    score: f64,
}

/// Score documents for overview ranking.
fn score_documents(
    conn: &rusqlite::Connection,
    docs: Vec<lattice::index::document_types::DocumentRow>,
    config: &OverviewConfig,
) -> Vec<ScoredDocument> {
    let now = Utc::now();

    let max_views = docs.iter().map(|d| d.view_count).max().unwrap_or(1).max(1) as f64;

    let mut scored = Vec::with_capacity(docs.len());

    for doc in docs {
        let view_data = view_tracking::get_view_data(conn, &doc.id).ok().flatten();

        let view_count = view_data.as_ref().map(|v| v.view_count).unwrap_or(0);
        let last_viewed = view_data.as_ref().map(|v| v.last_viewed);

        let view_score = if view_count > 0 {
            (1.0 + view_count as f64).ln() / (1.0 + max_views).ln()
        } else {
            0.0
        };

        let recency_score = if let Some(last) = last_viewed {
            let days_ago = (now - last).num_days().max(0) as f64;
            let half_life = config.recency_half_life_days as f64;
            0.5_f64.powf(days_ago / half_life)
        } else {
            0.0
        };

        let root_score = if doc.is_root { 1.0 } else { 0.5 };

        let score = (config.view_weight * view_score)
            + (config.recency_weight * recency_score)
            + (config.root_weight * root_score);

        scored.push(ScoredDocument { score });
    }

    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
    scored
}

// =============================================================================
// Criterion Groups
// =============================================================================

criterion_group!(
    list_filtering,
    list_no_filter,
    list_single_criterion,
    list_multiple_criteria,
    list_path_prefix,
    list_label_filter,
    list_date_range,
);

criterion_group!(
    ready_calculation,
    ready_basic,
    ready_with_blockers,
    ready_sort_policies,
    ready_count,
    ready_path_filter,
);

criterion_group!(
    overview_ranking,
    overview_repository_level,
    overview_view_distributions,
    overview_with_path_filter,
    overview_with_limit,
);

criterion_main!(list_filtering, ready_calculation, overview_ranking);
