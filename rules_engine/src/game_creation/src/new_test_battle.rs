use battle_data::battle::battle_data::BattleData;
use battle_data::battle::battle_history::BattleHistory;
use battle_data::battle::battle_status::BattleStatus;
use battle_data::battle::battle_tracing::BattleTracing;
use battle_data::battle::battle_turn_step::BattleTurnStep;
use battle_data::battle::effect_source::EffectSource;
use battle_data::battle::request_context::RequestContext;
use battle_data::battle::turn_data::TurnData;
use battle_data::battle_cards::all_cards::AllCards;
use battle_data::battle_player::player_data::{PlayerData, PlayerType};
use battle_mutations::turn_step_mutations::start_turn;
use battle_mutations::zone_mutations::{create_test_deck, deck};
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
) -> BattleData {
    let mut battle = BattleData {
        id,
        player_one: PlayerData {
            name: PlayerName::One,
            player_type: user,
            points: Points(0),
            spark_bonus: Spark(0),
            current_energy: Energy(0),
            produced_energy: Energy(0),
        },
        player_two: PlayerData {
            name: PlayerName::Two,
            player_type: enemy,
            points: Points(0),
            spark_bonus: Spark(0),
            current_energy: Energy(0),
            produced_energy: Energy(0),
        },
        cards: AllCards::default(),
        status: BattleStatus::Setup,
        priority: PlayerName::One,
        turn: TurnData { active_player: PlayerName::One, turn_id: TurnId::default() },
        step: BattleTurnStep::Judgment,
        seed,
        rng: Xoshiro256PlusPlus::seed_from_u64(seed),
        request_context: RequestContext::UserRequest,
        animations: None,
        prompt: None,
        prompt_resume_action: None,
        tracing: Some(BattleTracing::default()),
        history: Some(BattleHistory::default()),
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
    start_turn::run(&mut battle, PlayerName::One);
    battle
}
