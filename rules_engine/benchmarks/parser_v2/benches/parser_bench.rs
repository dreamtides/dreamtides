use criterion::{black_box, criterion_group, criterion_main, Criterion};
use parser_v2_benchmarks::benchmark_utils;

fn parser_benchmarks(c: &mut Criterion) {
    c.bench_function("parse_draw_cards", |b| {
        b.iter(|| {
            benchmark_utils::parse_single_card(black_box("Draw {cards}."), black_box("cards: 2"));
        });
    });
}

criterion_group!(benches, parser_benchmarks);
criterion_main!(benches);
