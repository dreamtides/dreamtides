use battle_data::battle::battle_data::BattleData;
use battle_data::battle::battle_status::BattleStatus;
use battle_data::battle::battle_turn_step::BattleTurnStep;
use battle_data::battle::request_context::RequestContext;
use battle_data::battle::turn_data::TurnData;
use battle_data::battle_animations::animation_data::AnimationData;
use battle_data::battle_cards::all_cards::AllCards;
use battle_data::battle_cards::card_properties::CardProperties;
use battle_data::battle_cards::card_types::{CardType, CharacterType};
use battle_data::battle_cards::zone::Zone;
use battle_data::battle_player::player_data::PlayerData;
use battle_mutations::zone_mutations::deck;
use core_data::identifiers::{BattleId, CardIdentity};
use core_data::numerics::{Energy, Points, Spark, TurnId};
use core_data::source::Source;
use core_data::types::PlayerName;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use uuid::Uuid;

/// Creates a new battle and starts it.
pub fn create_and_start(id: BattleId) -> BattleData {
    let mut battle = BattleData {
        id,
        user: PlayerData {
            points: Points(0),
            current_energy: Energy(50),
            produced_energy: Energy(50),
            spark_bonus: Spark(0),
        },
        enemy: PlayerData {
            points: Points(0),
            current_energy: Energy(50),
            produced_energy: Energy(50),
            spark_bonus: Spark(0),
        },
        cards: AllCards::default(),
        status: BattleStatus::Setup,
        turn: TurnData { active_player: PlayerName::User, turn_id: TurnId::default() },
        step: BattleTurnStep::Judgment,
        rng: Xoshiro256PlusPlus::seed_from_u64(3141592653589793),
        request_context: RequestContext::UserRequest,
        animations: Some(AnimationData::default()),
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
            card_type: CardType::Character(CharacterType::Explorer),
            is_fast: false,
        });
        battle.cards.create_card(identity, PlayerName::Enemy, Zone::Deck, CardProperties {
            spark: Some(Spark(2)),
            cost: Some(Energy(2)),
            card_type: CardType::Character(CharacterType::Explorer),
            is_fast: false,
        });
    }
}
