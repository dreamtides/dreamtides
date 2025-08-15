use std::hint::black_box;

use ai_uct::uct_search;
use battle_state::battle::battle_state::BattleState;
use core_data::types::PlayerName;
use iai_callgrind::{library_benchmark, library_benchmark_group, main};

#[library_benchmark]
#[bench::eval(benchmark_battles::core_11_battle::generate_core_11_battle())]
fn bench_core11_evaluate(mut battle: BattleState) -> f64 {
    black_box(uct_search::evaluate_for_benchmarking(&mut battle, PlayerName::One)).into_inner()
}

library_benchmark_group!(
    name = bench_group;
    benchmarks = bench_core11_evaluate
);

main!(library_benchmark_groups = bench_group);
