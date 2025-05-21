use criterion::criterion_main;

use crate::file_parsing_benchmarks::parser_benchmarks;

pub mod file_parsing_benchmarks;

criterion_main!(parser_benchmarks);
