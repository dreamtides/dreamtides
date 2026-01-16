use criterion::{Criterion, criterion_group, criterion_main};
use lattice::cli::hello_command;

fn hello_benchmark(c: &mut Criterion) {
    c.bench_function("lattice_hello", |b| {
        b.iter(|| {
            let _result = hello_command::hello_world();
        });
    });
}

criterion_group!(benches, hello_benchmark);
criterion_main!(benches);
