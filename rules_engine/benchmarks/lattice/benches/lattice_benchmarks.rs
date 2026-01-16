use criterion::{Criterion, criterion_group, criterion_main};

fn placeholder_benchmark(c: &mut Criterion) {
    c.bench_function("lattice_placeholder", |b| {
        b.iter(|| {
            let _x = 1 + 1;
        });
    });
}

criterion_group!(benches, placeholder_benchmark);
criterion_main!(benches);
