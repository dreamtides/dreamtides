use battle_tests::determinism::run_determinism_test;

#[test]
fn test_battle_actions_deterministic() {
    const SEED: u64 = 42;
    const NUM_RUNS: usize = 3;
    const ACTIONS_PER_RUN: usize = 100;

    let result = run_determinism_test(SEED, NUM_RUNS, ACTIONS_PER_RUN);

    if !result.success {
        panic!(
            "Determinism test failed: {}",
            result.error_message.unwrap_or_else(|| "Unknown error".to_string())
        );
    }

    println!(
        "Successfully validated {} runs with {} actions each were deterministic",
        result.num_runs, result.actions_executed
    );
}
