use criterion::criterion_main;

use crate::old_playout_benchmarks::old_playout_benchmarks;
use crate::parser_benchmarks::parser_benchmarks;
use crate::playout_benchmarks::playout_benchmarks;

pub mod old_playout_benchmarks;
pub mod parser_benchmarks;
pub mod playout_benchmarks;

criterion_main!(parser_benchmarks, old_playout_benchmarks, playout_benchmarks);
