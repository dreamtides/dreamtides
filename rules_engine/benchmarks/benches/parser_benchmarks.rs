use std::fs;
use std::path::Path;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use parser::ability_parser;
use tracing::{subscriber, Level};

criterion_group!(parser_benchmarks, parse_abilities);
criterion_main!(parser_benchmarks);

pub fn parse_abilities(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_abilities");
    group
        .significance_level(0.01)
        .sample_size(10)
        .noise_threshold(0.03)
        .measurement_time(Duration::from_secs(60));

    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let examples_path = Path::new(manifest_dir).join("benches").join("ability_text_examples.txt");

    let ability_texts = fs::read_to_string(examples_path)
        .expect("Should be able to read the ability text examples file");

    let ability_expressions: Vec<String> = ability_texts
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.to_lowercase())
        .collect();

    subscriber::with_default(error_subscriber, || {
        group.bench_function("parse_abilities", |b| {
            b.iter_batched(
                || ability_expressions.clone(),
                |expressions| {
                    for expr in expressions {
                        ability_parser::parse(&expr);
                    }
                },
                BatchSize::SmallInput,
            );
        });
    });
}
