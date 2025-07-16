use std::collections::VecDeque;

use battle_mutations::card_mutations::{create_test_deck, deck};
use battle_mutations::phase_mutations::turn;
use battle_state::battle::all_cards::AllCards;
use battle_state::battle::battle_state::{BattleState, RequestContext};
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use battle_state::battle::turn_data::TurnData;
use battle_state::battle::turn_history::TurnHistory;
use battle_state::battle_cards::ability_state::AbilityState;
use battle_state::battle_player::battle_player_state::{BattlePlayerState, PlayerType};
use battle_state::battle_player::player_map::PlayerMap;
use battle_state::core::effect_source::EffectSource;
use battle_state::triggers::trigger_state::TriggerState;
use core_data::identifiers::BattleId;
use core_data::numerics::{Energy, Points, Spark, TurnId};
use core_data::types::PlayerName;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;

/// Creates a new test battle between two Agents and starts it.
pub fn create_and_start(
    id: BattleId,
    seed: u64,
    user: PlayerType,
    enemy: PlayerType,
    request_context: RequestContext,
) -> BattleState {
    let mut battle = BattleState {
        id,
        players: PlayerMap {
            one: BattlePlayerState {
                player_type: user,
                points: Points(0),
                spark_bonus: Spark(0),
                current_energy: Energy(0),
                produced_energy: Energy(0),
            },
            two: BattlePlayerState {
                player_type: enemy,
                points: Points(0),
                spark_bonus: Spark(0),
                current_energy: Energy(0),
                produced_energy: Energy(0),
            },
        },
        cards: AllCards::default(),
        status: BattleStatus::Setup,
        stack_priority: None,
        turn: TurnData { active_player: PlayerName::One, turn_id: TurnId::default() },
        phase: BattleTurnPhase::Judgment,
        seed,
        rng: Xoshiro256PlusPlus::seed_from_u64(seed),
        animations: None,
        prompt: None,
        triggers: TriggerState::default(),
        activated_abilities: PlayerMap::default(),
        ability_state: AbilityState::default(),
        pending_effects: VecDeque::new(),
        tracing: None,
        action_history: None,
        turn_history: TurnHistory::default(),
        request_context,
    };

    create_test_deck::add(&mut battle, PlayerName::One);
    create_test_deck::add(&mut battle, PlayerName::Two);

    battle.status = BattleStatus::Playing;
    deck::draw_cards(
        &mut battle,
        EffectSource::Game { controller: PlayerName::One },
        PlayerName::One,
        5,
    );
    deck::draw_cards(
        &mut battle,
        EffectSource::Game { controller: PlayerName::Two },
        PlayerName::Two,
        5,
    );
    turn::start_turn(&mut battle, PlayerName::One);
    battle
}
