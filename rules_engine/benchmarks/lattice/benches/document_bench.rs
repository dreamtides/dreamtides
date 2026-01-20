//! Benchmarks for document operations.
//!
//! Measures performance of:
//! - Frontmatter parsing at various complexity levels
//! - Markdown body parsing at various sizes
//! - Link extraction from body and frontmatter
//! - Document formatting operations
//!
//! Run with:
//! ```bash
//! just bench-lattice -- frontmatter_parsing
//! just bench-lattice -- body_parsing
//! just bench-lattice -- link_extraction
//! just bench-lattice -- formatting
//! ```

use std::path::Path;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use lattice::document::frontmatter_parser;
use lattice::format::markdown_formatter::{self, FormatConfig};
use lattice::link::{frontmatter_links, link_extractor};
use lattice_benchmarks::document_generators::{
    generate_body_with_lines, generate_body_with_links, generate_frontmatter_links_document,
    generate_frontmatter_with_dependencies, generate_frontmatter_with_labels,
    generate_full_frontmatter, generate_minimal_frontmatter, generate_unformatted_content,
};
use lattice_benchmarks::test_repo::{RepoConfig, generate_repo};
use tracing::Level;

/// Benchmark parsing minimal frontmatter (required fields only).
fn parse_minimal_frontmatter(c: &mut Criterion) {
    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let content = generate_minimal_frontmatter();
        let path = Path::new("test.md");

        c.bench_function("frontmatter_parse_minimal", |b| {
            b.iter(|| criterion::black_box(frontmatter_parser::parse(&content, path)))
        });
    });
}

/// Benchmark parsing full frontmatter (all optional fields populated).
fn parse_full_frontmatter(c: &mut Criterion) {
    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let content = generate_full_frontmatter();
        let path = Path::new("test.md");

        c.bench_function("frontmatter_parse_full", |b| {
            b.iter(|| criterion::black_box(frontmatter_parser::parse(&content, path)))
        });
    });
}

/// Benchmark parsing frontmatter with varying numbers of labels.
fn parse_frontmatter_with_labels(c: &mut Criterion) {
    let mut group = c.benchmark_group("frontmatter_parse_labels");

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let path = Path::new("test.md");

        for label_count in [10, 50, 100] {
            let content = generate_frontmatter_with_labels(label_count);

            group.bench_with_input(
                BenchmarkId::from_parameter(label_count),
                &content,
                |b, content| {
                    b.iter(|| criterion::black_box(frontmatter_parser::parse(content, path)))
                },
            );
        }
    });

    group.finish();
}

/// Benchmark parsing frontmatter with varying numbers of dependencies.
fn parse_frontmatter_with_dependencies(c: &mut Criterion) {
    let mut group = c.benchmark_group("frontmatter_parse_dependencies");

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let path = Path::new("test.md");

        for dep_count in [10, 50] {
            let content = generate_frontmatter_with_dependencies(dep_count);

            group.bench_with_input(
                BenchmarkId::from_parameter(dep_count),
                &content,
                |b, content| {
                    b.iter(|| criterion::black_box(frontmatter_parser::parse(content, path)))
                },
            );
        }
    });

    group.finish();
}

/// Benchmark parsing documents with varying body sizes.
fn parse_body_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("body_parse_by_size");

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let path = Path::new("test.md");

        for line_count in [50, 200, 500] {
            let content = generate_body_with_lines(line_count);

            group.bench_with_input(
                BenchmarkId::new("lines", line_count),
                &content,
                |b, content| {
                    b.iter(|| criterion::black_box(frontmatter_parser::parse(content, path)))
                },
            );
        }
    });

    group.finish();
}

/// Benchmark link extraction from documents with few links.
fn extract_links_few(c: &mut Criterion) {
    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let content = generate_body_with_links(5);
        let path = Path::new("test.md");
        let parsed = frontmatter_parser::parse(&content, path).expect("parse failed");

        c.bench_function("link_extract_5", |b| {
            b.iter(|| criterion::black_box(link_extractor::extract(&parsed.body)))
        });
    });
}

/// Benchmark link extraction from documents with many links.
fn extract_links_many(c: &mut Criterion) {
    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let content = generate_body_with_links(50);
        let path = Path::new("test.md");
        let parsed = frontmatter_parser::parse(&content, path).expect("parse failed");

        c.bench_function("link_extract_50", |b| {
            b.iter(|| criterion::black_box(link_extractor::extract(&parsed.body)))
        });
    });
}

/// Benchmark link extraction scaling with link count.
fn extract_links_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("link_extract_scaling");

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let path = Path::new("test.md");

        for link_count in [5, 25, 50, 100] {
            let content = generate_body_with_links(link_count);
            let parsed = frontmatter_parser::parse(&content, path).expect("parse failed");

            group.bench_with_input(
                BenchmarkId::from_parameter(link_count),
                &parsed.body,
                |b, body| b.iter(|| criterion::black_box(link_extractor::extract(body))),
            );
        }
    });

    group.finish();
}

/// Benchmark frontmatter link extraction.
fn extract_frontmatter_links(c: &mut Criterion) {
    let mut group = c.benchmark_group("frontmatter_link_extract");

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let path = Path::new("test.md");

        for link_count in [5, 25, 50] {
            let content = generate_frontmatter_links_document(link_count, link_count);
            let parsed = frontmatter_parser::parse(&content, path).expect("parse failed");

            group.bench_with_input(
                BenchmarkId::new("total_links", link_count * 2),
                &parsed.frontmatter,
                |b, frontmatter| {
                    b.iter(|| criterion::black_box(frontmatter_links::extract(frontmatter)))
                },
            );
        }
    });

    group.finish();
}

/// Benchmark formatting a single document.
fn format_single_document(c: &mut Criterion) {
    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let content = generate_unformatted_content();
        let config = FormatConfig::default();

        c.bench_function("format_single_document", |b| {
            b.iter(|| criterion::black_box(markdown_formatter::format_content(&content, &config)))
        });
    });
}

/// Benchmark formatting with different line widths.
fn format_with_line_widths(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_line_widths");

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let content = generate_unformatted_content();

        for width in [60, 80, 100, 120] {
            let config = FormatConfig::new(width);

            group.bench_with_input(BenchmarkId::from_parameter(width), &content, |b, content| {
                b.iter(|| {
                    criterion::black_box(markdown_formatter::format_content(content, &config))
                })
            });
        }
    });

    group.finish();
}

/// Benchmark formatting documents of varying sizes.
fn format_by_document_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_by_size");

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        let path = Path::new("test.md");
        let config = FormatConfig::default();

        for line_count in [50, 200, 500] {
            let content = generate_body_with_lines(line_count);
            let parsed = frontmatter_parser::parse(&content, path).expect("parse failed");

            group.bench_with_input(
                BenchmarkId::new("lines", line_count),
                &parsed.body,
                |b, body| {
                    b.iter(|| {
                        criterion::black_box(markdown_formatter::format_content(body, &config))
                    })
                },
            );
        }
    });

    group.finish();
}

/// Benchmark formatting a directory of documents.
fn format_directory(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_directory");
    group.sample_size(10);

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    tracing::subscriber::with_default(error_subscriber, || {
        for doc_count in [10, 50] {
            let repo = generate_repo(&RepoConfig::with_document_count(doc_count));
            let config = FormatConfig::default().with_dry_run(true);

            group.bench_with_input(
                BenchmarkId::new("documents", doc_count),
                &repo.root,
                |b, root| {
                    b.iter(|| {
                        criterion::black_box(markdown_formatter::format_directory(root, &config))
                    })
                },
            );
        }
    });

    group.finish();
}

criterion_group!(
    frontmatter_parsing,
    parse_minimal_frontmatter,
    parse_full_frontmatter,
    parse_frontmatter_with_labels,
    parse_frontmatter_with_dependencies,
);

criterion_group!(body_parsing, parse_body_sizes,);

criterion_group!(
    link_extraction,
    extract_links_few,
    extract_links_many,
    extract_links_scaling,
    extract_frontmatter_links,
);

criterion_group!(
    formatting,
    format_single_document,
    format_with_line_widths,
    format_by_document_size,
    format_directory,
);

criterion_main!(frontmatter_parsing, body_parsing, link_extraction, formatting);
