use criterion::criterion_main;

use crate::parser_benchmarks::parser_benchmarks;
use crate::playout_benchmarks::playout_benchmarks;

pub mod parser_benchmarks;
pub mod playout_benchmarks;

criterion_main!(parser_benchmarks, playout_benchmarks);
