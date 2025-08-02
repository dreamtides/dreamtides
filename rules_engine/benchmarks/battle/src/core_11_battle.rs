use ai_data::game_ai::GameAI;
use battle_mutations::actions::apply_battle_action;
use battle_queries::battle_card_queries::card;
use battle_queries::legal_action_queries::legal_actions;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::{BattleState, LoggingOptions, RequestContext};
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use battle_state::battle::card_id::{CardId, HandCardId};
use battle_state::battle_player::battle_player_state::{
    CreateBattlePlayer, PlayerType, TestDeckName,
};
use core_data::identifiers::BattleId;
use core_data::types::PlayerName;
use game_creation::new_test_battle;
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
                card_data.name, character_id.0.0, character_state.spark.0
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
    let seed = 12345678912345;
    let mut battle = new_test_battle::create_and_start(
        BattleId(Uuid::new_v4()),
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

    let mut turn_count = 0;
    let max_turns = 1000;

    loop {
        turn_count += 1;
        if turn_count > max_turns {
            if enable_logging {
                println!(
                    "Battle ended without Player 1 ever having 6+ legal actions (max turn limit reached)"
                );
            }
            break;
        }

        let next_player = legal_actions::next_to_act(&battle);
        if next_player.is_none() {
            if enable_logging {
                println!("Battle ended without Player 1 ever having 6+ legal actions (game over)");
            }
            break;
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

            // Assert that these exact 6 actions are always returned
            let expected_actions = [
                BattleAction::EndTurn,
                BattleAction::PlayCardFromHand(HandCardId(CardId(18))),
                BattleAction::PlayCardFromHand(HandCardId(CardId(19))),
                BattleAction::PlayCardFromHand(HandCardId(CardId(22))),
                BattleAction::PlayCardFromHand(HandCardId(CardId(25))),
                BattleAction::PlayCardFromHand(HandCardId(CardId(29))),
            ];

            assert_eq!(
                all_actions.len(),
                expected_actions.len(),
                "Expected exactly {} legal actions, got {}",
                expected_actions.len(),
                all_actions.len()
            );

            for (i, expected_action) in expected_actions.iter().enumerate() {
                assert_eq!(
                    &all_actions[i],
                    expected_action,
                    "Action {} does not match expected. Got {:?}, expected {:?}",
                    i + 1,
                    all_actions[i],
                    expected_action
                );
            }

            assert_eq!(
                battle.turn.turn_id.0, 7,
                "Expected turn ID to be 7, got {}",
                battle.turn.turn_id.0
            );

            assert_eq!(
                battle.turn.active_player,
                PlayerName::One,
                "Expected active player to be Player One, got {:?}",
                battle.turn.active_player
            );

            assert_eq!(
                battle.phase,
                BattleTurnPhase::Main,
                "Expected phase to be Main, got {:?}",
                battle.phase
            );

            assert_eq!(
                battle.cards.battlefield_state(PlayerName::One).len(),
                0,
                "Expected Player One battlefield to be empty, got {} characters",
                battle.cards.battlefield_state(PlayerName::One).len()
            );

            assert_eq!(
                battle.cards.battlefield_state(PlayerName::Two).len(),
                0,
                "Expected Player Two battlefield to be empty, got {} characters",
                battle.cards.battlefield_state(PlayerName::Two).len()
            );

            if enable_logging {
                println!("✓ All assertions passed - deterministic battle state validated");
            }
            break;
        }

        if legal.is_empty() {
            if enable_logging {
                println!(
                    "Battle ended without Player 1 ever having 6+ legal actions (no legal actions)"
                );
            }
            break;
        }

        let action = legal.all()[0];
        apply_battle_action::execute(&mut battle, current_player, action);
    }

    battle
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
