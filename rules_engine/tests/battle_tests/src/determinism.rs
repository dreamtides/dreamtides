use action_data::game_action_data::GameAction;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use test_utils::battle::test_battle::TestBattle;

pub struct DeterminismTestResult {
    pub num_runs: usize,
    pub actions_executed: usize,
    pub success: bool,
    pub error_message: Option<String>,
}

pub fn run_determinism_test(
    seed: u64,
    num_runs: usize,
    actions_per_run: usize,
) -> DeterminismTestResult {
    let mut action_sequences: Vec<Vec<(bool, usize, GameAction)>> = Vec::new();
    let mut legal_action_sequences: Vec<Vec<(bool, Vec<GameAction>)>> = Vec::new();

    for _run in 0..num_runs {
        let mut session = TestBattle::builder().seed(seed).connect();

        let mut rng = StdRng::seed_from_u64(seed);
        let mut actions_taken = Vec::new();
        let mut legal_actions_history = Vec::new();

        for _ in 0..actions_per_run {
            let is_user_turn = session.user_client.me.can_act();
            let is_enemy_turn = session.enemy_client.me.can_act();

            // Get legal actions from the appropriate client
            let all_legal_actions = if is_user_turn {
                session.user_client.legal_actions()
            } else if is_enemy_turn {
                session.enemy_client.legal_actions()
            } else {
                vec![]
            };

            // Filter out Undo and Debug actions to avoid test issues
            let legal_actions: Vec<GameAction> = all_legal_actions
                .into_iter()
                .filter(|action| {
                    !matches!(action, GameAction::Undo(_) | GameAction::DebugAction(_))
                })
                .collect();

            legal_actions_history.push((is_user_turn, legal_actions.clone()));

            if legal_actions.is_empty() {
                return DeterminismTestResult {
                    num_runs,
                    actions_executed: actions_taken.len(),
                    success: false,
                    error_message: Some(format!(
                        "No legal actions found at step {}",
                        actions_taken.len()
                    )),
                };
            }

            let chosen_idx = rng.random_range(0..legal_actions.len());
            let action = legal_actions[chosen_idx].clone();
            actions_taken.push((is_user_turn, chosen_idx, action.clone()));

            if is_user_turn {
                session.perform_user_action(action);
            } else {
                session.perform_enemy_action(action);
            }

            if session.user_client.is_game_over() {
                break;
            }
        }

        action_sequences.push(actions_taken);
        legal_action_sequences.push(legal_actions_history);
    }

    // Validate determinism
    for run in 1..num_runs {
        if action_sequences[0].len() != action_sequences[run].len() {
            return DeterminismTestResult {
                num_runs,
                actions_executed: action_sequences[0].len(),
                success: false,
                error_message: Some(format!(
                    "Run {} produced different number of actions ({} vs {})",
                    run,
                    action_sequences[0].len(),
                    action_sequences[run].len()
                )),
            };
        }

        for (action_idx, ((is_user0, idx0, action0), (is_user_n, idx_n, action_n))) in
            action_sequences[0].iter().zip(action_sequences[run].iter()).enumerate()
        {
            if is_user0 != is_user_n {
                return DeterminismTestResult {
                    num_runs,
                    actions_executed: action_idx,
                    success: false,
                    error_message: Some(format!(
                        "Run {run} action {action_idx} was for different player"
                    )),
                };
            }

            if idx0 != idx_n {
                return DeterminismTestResult {
                    num_runs,
                    actions_executed: action_idx,
                    success: false,
                    error_message: Some(format!(
                        "Run {run} action {action_idx} chose different index ({idx0} vs {idx_n})"
                    )),
                };
            }

            if format!("{action0:?}") != format!("{action_n:?}") {
                return DeterminismTestResult {
                    num_runs,
                    actions_executed: action_idx,
                    success: false,
                    error_message: Some(format!(
                        "Run {run} action {action_idx} produced different action"
                    )),
                };
            }
        }

        if legal_action_sequences[0].len() != legal_action_sequences[run].len() {
            return DeterminismTestResult {
                num_runs,
                actions_executed: action_sequences[0].len(),
                success: false,
                error_message: Some(format!(
                    "Run {run} produced different legal action history length"
                )),
            };
        }

        for (step, ((is_user0, legal0), (is_user_n, legal_n))) in
            legal_action_sequences[0].iter().zip(legal_action_sequences[run].iter()).enumerate()
        {
            if is_user0 != is_user_n {
                return DeterminismTestResult {
                    num_runs,
                    actions_executed: step,
                    success: false,
                    error_message: Some(format!("Run {run} step {step} was for different player")),
                };
            }

            if legal0.len() != legal_n.len() {
                return DeterminismTestResult {
                    num_runs,
                    actions_executed: step,
                    success: false,
                    error_message: Some(format!(
                        "Run {} step {} had different number of legal actions ({} vs {})",
                        run,
                        step,
                        legal0.len(),
                        legal_n.len()
                    )),
                };
            }

            for (i, (action0, action_n)) in legal0.iter().zip(legal_n.iter()).enumerate() {
                if format!("{action0:?}") != format!("{action_n:?}") {
                    return DeterminismTestResult {
                        num_runs,
                        actions_executed: step,
                        success: false,
                        error_message: Some(format!(
                            "Run {run} step {step} legal action {i} differs"
                        )),
                    };
                }
            }
        }
    }

    DeterminismTestResult {
        num_runs,
        actions_executed: action_sequences[0].len(),
        success: true,
        error_message: None,
    }
}
