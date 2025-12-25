use criterion::{criterion_group, criterion_main, Criterion};

fn pipeline_benchmarks(_c: &mut Criterion) {}

criterion_group!(benches, pipeline_benchmarks);
criterion_main!(benches);
