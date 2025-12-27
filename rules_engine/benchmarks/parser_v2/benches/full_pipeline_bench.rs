use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use parser_v2_benchmarks::benchmark_utils;

fn pipeline_benchmarks(c: &mut Criterion) {
    let cards_file = benchmark_utils::load_cards_toml();

    c.bench_function("parse_all_cards_toml", |b| {
        b.iter_batched(
            || cards_file.clone(),
            benchmark_utils::parse_all_cards,
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, pipeline_benchmarks);
criterion_main!(benches);
