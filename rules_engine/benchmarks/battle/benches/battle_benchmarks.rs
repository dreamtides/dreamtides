use criterion::criterion_main;

use crate::playout_benchmarks::playout_benchmarks;

pub mod playout_benchmarks;

criterion_main!(playout_benchmarks);
