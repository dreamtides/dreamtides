use battle_data::battle::battle_data::BattleData;
use battle_data::battle::battle_status::BattleStatus;
use battle_data::battle::battle_turn_step::BattleTurnStep;
use battle_data::battle::request_context::RequestContext;
use battle_data::battle::turn_data::TurnData;
use battle_data::cards::all_cards::AllCards;
use battle_data::cards::card_properties::CardProperties;
use battle_data::cards::zone::Zone;
use battle_data::player::player_data::PlayerData;
use battle_mutations::zones::deck;
use core_data::identifiers::{BattleId, CardIdentity};
use core_data::numerics::{Energy, Spark, TurnNumber};
use core_data::source::Source;
use core_data::types::PlayerName;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use uuid::Uuid;

/// Creates a new battle and starts it.
pub fn create_and_start(id: BattleId) -> BattleData {
    let mut battle = BattleData {
        id,
        user: PlayerData::default(),
        enemy: PlayerData::default(),
        cards: AllCards::default(),
        status: BattleStatus::Setup,
        turn: TurnData { active_player: PlayerName::User, turn_number: TurnNumber::default() },
        step: BattleTurnStep::Judgment,
        rng: Xoshiro256PlusPlus::seed_from_u64(3141592653589793),
        request_context: RequestContext::UserRequest,
    };
    create_cards(&mut battle);
    battle.status = BattleStatus::Playing;
    deck::draw_cards(&mut battle, Source::Game, PlayerName::User, 3);
    deck::draw_cards(&mut battle, Source::Game, PlayerName::Enemy, 4);
    battle
}

fn create_cards(battle: &mut BattleData) {
    for _ in 0..30 {
        let identity = CardIdentity(Uuid::new_v4());
        battle.cards.create_card(identity, PlayerName::User, Zone::Deck, CardProperties {
            spark: Some(Spark(2)),
            cost: Some(Energy(2)),
        });
        battle.cards.create_card(identity, PlayerName::Enemy, Zone::Deck, CardProperties {
            spark: Some(Spark(2)),
            cost: Some(Energy(2)),
        });
    }
}
