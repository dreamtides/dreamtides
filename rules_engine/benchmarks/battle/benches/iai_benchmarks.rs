use std::hint::black_box;

use ai_agents::agent_search;
use ai_data::game_ai::GameAI;
use ai_uct::uct_search;
use battle_mutations::player_mutations::player_state;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use core_data::types::PlayerName;
#[cfg(target_os = "linux")]
use iai_callgrind::main;
use iai_callgrind::{library_benchmark, library_benchmark_group};

#[library_benchmark]
#[bench::eval(benchmark_battles::core_11_battle::generate_core_11_battle())]
fn bench_core11_evaluate(mut battle: BattleState) -> f64 {
    black_box(uct_search::evaluate_for_benchmarking(&mut battle, PlayerName::One)).into_inner()
}

#[library_benchmark]
#[bench::eval(benchmark_battles::core_11_battle::generate_core_11_battle())]
fn bench_core11_search_action_candidate(battle: BattleState) -> BattleAction {
    black_box(uct_search::search_first_action_candidate_for_benchmarking(&battle, PlayerName::One))
}

#[library_benchmark]
#[bench::eval(benchmark_battles::core_11_battle::generate_core_11_battle())]
fn bench_core11_randomize_battle_player(battle: BattleState) -> BattleState {
    black_box(player_state::randomize_battle_player(&battle, PlayerName::One, 31415926535897))
}

#[library_benchmark]
#[bench::eval(benchmark_battles::core_11_battle::generate_core_11_battle())]
fn bench_core11_select(battle: BattleState) -> BattleAction {
    black_box(agent_search::select_action_unchecked(
        &battle,
        PlayerName::One,
        &GameAI::MonteCarloSingleThreaded(1),
    ))
}

library_benchmark_group!(
    name = bench_group;
    benchmarks = bench_core11_evaluate, bench_core11_search_action_candidate
);

#[cfg(target_os = "linux")]
main!(library_benchmark_groups = bench_group);

#[cfg(not(target_os = "linux"))]
fn main() {}
