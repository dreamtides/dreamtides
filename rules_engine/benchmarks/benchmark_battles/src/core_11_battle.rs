use ai_data::game_ai::GameAI;
use battle_mutations::actions::apply_battle_action;
use battle_queries::battle_card_queries::card;
use battle_queries::legal_action_queries::legal_actions;
use battle_state::battle::battle_state::{BattleState, LoggingOptions, RequestContext};
use battle_state::battle_player::battle_player_state::{
    CreateBattlePlayer, PlayerType, TestDeckName,
};
use core_data::identifiers::BattleId;
use core_data::types::PlayerName;
use game_creation::new_test_battle;
use state_provider::display_state_provider::DisplayStateProvider;
use state_provider::state_provider::StateProvider;
use state_provider::test_state_provider::TestStateProvider;
use uuid::Uuid;

fn print_battlefield_state(battle: &BattleState) {
    println!("Turn ID: {}", battle.turn.turn_id.0);
    println!("Active Player: {:?}", battle.turn.active_player);
    println!("Phase: {:?}", battle.phase);

    for player in [PlayerName::One, PlayerName::Two] {
        let battlefield = battle.cards.battlefield_state(player);
        println!("Player {:?} battlefield ({} characters):", player, battlefield.len());

        for (character_id, character_state) in battlefield.iter() {
            let card_data = card::get(battle, *character_id);
            println!(
                "  - {:?} (ID: {}, Spark: {})",
                card_data.identity, character_id.0.0, character_state.spark.0
            );
        }

        if battlefield.is_empty() {
            println!("  (empty)");
        }
    }
    println!();
}

pub fn generate_core_11_battle() -> BattleState {
    generate_core_11_battle_with_logging(false)
}

fn generate_core_11_battle_with_logging(enable_logging: bool) -> BattleState {
    let seed = 1234567891234;
    let provider = TestStateProvider::new();
    let streaming_assets_path = logging::get_developer_mode_streaming_assets_path();
    let _ = provider.initialize("/tmp/test", &streaming_assets_path);
    let mut battle = new_test_battle::create_and_start(
        BattleId(Uuid::new_v4()),
        provider.tabula(),
        seed,
        CreateBattlePlayer {
            player_type: PlayerType::Agent(GameAI::AlwaysPanic),
            deck_name: TestDeckName::CoreEleven,
        },
        CreateBattlePlayer {
            player_type: PlayerType::Agent(GameAI::AlwaysPanic),
            deck_name: TestDeckName::CoreEleven,
        },
        RequestContext { logging_options: LoggingOptions::default() },
    );

    let mut action_count = 0;
    let max_actions = 999;

    loop {
        action_count += 1;
        if action_count > max_actions {
            panic!(
                "Battle ended on action {action_count:?} without Player 1 ever having 6+ legal actions (max turn limit reached)"
            );
        }

        let next_player = legal_actions::next_to_act(&battle);
        if next_player.is_none() {
            panic!(
                "Battle ended on action {action_count:?} without Player 1 ever having 6+ legal actions (game over)"
            );
        }

        let current_player = next_player.unwrap();
        let legal = legal_actions::compute(&battle, current_player);

        if current_player == PlayerName::One && legal.len() >= 6 {
            let all_actions = legal.all();
            if enable_logging {
                println!("=== BATTLE STATE WHEN PLAYER 1 HAS 6+ LEGAL ACTIONS ===");
                print_battlefield_state(&battle);
                println!("Player 1 has {} legal actions:", all_actions.len());
                for (i, action) in all_actions.iter().take(6).enumerate() {
                    println!("  {}: {:?}", i + 1, action);
                }
            }

            return battle;
        }

        if legal.is_empty() {
            panic!("Battle ended without Player 1 ever having 6+ legal actions (no legal actions)");
        }

        let action = legal.all()[0];
        apply_battle_action::execute(&mut battle, current_player, action);
    }
}

pub fn main() {
    use std::env;

    let args: Vec<String> = env::args().collect();
    let silent = args.len() > 1 && args[1] == "--silent";

    if silent {
        println!("Running silent version (assertions still validate):");
        generate_core_11_battle_with_logging(false);
        println!("Silent run completed successfully.");
    } else {
        generate_core_11_battle_with_logging(true);
    }
}
