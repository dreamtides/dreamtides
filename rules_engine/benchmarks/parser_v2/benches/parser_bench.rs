use criterion::{criterion_group, criterion_main, Criterion};

fn parser_benchmarks(_c: &mut Criterion) {}

criterion_group!(benches, parser_benchmarks);
criterion_main!(benches);
