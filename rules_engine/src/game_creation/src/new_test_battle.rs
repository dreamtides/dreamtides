use ability_data::ability::Ability;
use ability_data::cost::Cost;
use ability_data::effect::Effect;
use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::standard_effect::StandardEffect;
use ai_data::game_ai::GameAI;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle::battle_status::BattleStatus;
use battle_data::battle::battle_turn_step::BattleTurnStep;
use battle_data::battle::effect_source::EffectSource;
use battle_data::battle::request_context::RequestContext;
use battle_data::battle::turn_data::TurnData;
use battle_data::battle_cards::all_cards::AllCards;
use battle_data::battle_cards::card_data::CardData;
use battle_data::battle_cards::card_id::ObjectId;
use battle_data::battle_cards::card_properties::CardProperties;
use battle_data::battle_cards::zone::Zone;
use battle_data::battle_player::player_data::PlayerData;
use battle_mutations::zone_mutations::deck;
use core_data::card_types::{CardType, CharacterType};
use core_data::identifiers::{BattleId, CardId};
use core_data::numerics::{Energy, Points, Spark, TurnId};
use core_data::types::PlayerName;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

/// Creates a new test battle between two Agents and starts it.
pub fn create_and_start(
    id: BattleId,
    user_agent: Option<GameAI>,
    enemy_agent: Option<GameAI>,
) -> BattleData {
    let mut battle = BattleData {
        id,
        user: PlayerData {
            name: PlayerName::User,
            ai: user_agent,
            points: Points(0),
            spark_bonus: Spark(0),
            current_energy: Energy(2),
            produced_energy: Energy(2),
        },
        enemy: PlayerData {
            name: PlayerName::Enemy,
            ai: enemy_agent,
            points: Points(0),
            spark_bonus: Spark(0),
            current_energy: Energy(2),
            produced_energy: Energy(2),
        },
        cards: AllCards::default(),
        status: BattleStatus::Setup,
        turn: TurnData { active_player: PlayerName::User, turn_id: TurnId::default() },
        step: BattleTurnStep::Judgment,
        rng: Xoshiro256PlusPlus::seed_from_u64(3141592653589793),
        request_context: RequestContext::UserRequest,
        animations: None,
        prompt: None,
    };
    create_cards(&mut battle, PlayerName::User);
    create_cards(&mut battle, PlayerName::Enemy);
    battle.status = BattleStatus::Playing;
    deck::draw_cards(
        &mut battle,
        EffectSource::Game { controller: PlayerName::User },
        PlayerName::User,
        5,
    );
    deck::draw_cards(
        &mut battle,
        EffectSource::Game { controller: PlayerName::Enemy },
        PlayerName::Enemy,
        5,
    );
    battle
}

fn create_cards(battle: &mut BattleData, player_name: PlayerName) {
    for _ in 0..30 {
        battle.cards.create_card(CardData {
            id: CardId::default(),
            owner: player_name,
            zone: Zone::Deck,
            object_id: ObjectId::default(),
            properties: CardProperties {
                spark: Some(Spark(rand::rng().random_range(1..=5))),
                cost: Some(Energy(rand::rng().random_range(1..=5))),
                card_type: CardType::Character(CharacterType::Explorer),
                is_fast: false,
            },
            abilities: vec![],
            revealed_to_owner: false,
            revealed_to_opponent: false,
            targets: vec![],
            turn_entered_current_zone: TurnData::default(),
        });
    }

    battle.cards.create_card(CardData {
        id: CardId::default(),
        owner: player_name,
        zone: Zone::Deck,
        object_id: ObjectId::default(),
        properties: CardProperties {
            spark: None,
            cost: Some(Energy(2)),
            card_type: CardType::Event,
            is_fast: true,
        },
        abilities: vec![Ability::Event(Effect::Effect(StandardEffect::DissolveCharacter {
            target: Predicate::Enemy(CardPredicate::Character),
        }))],
        revealed_to_owner: false,
        revealed_to_opponent: false,
        targets: vec![],
        turn_entered_current_zone: TurnData::default(),
    });

    battle.cards.create_card(CardData {
        id: CardId::default(),
        owner: player_name,
        zone: Zone::Deck,
        object_id: ObjectId::default(),
        properties: CardProperties {
            spark: None,
            cost: Some(Energy(1)),
            card_type: CardType::Event,
            is_fast: true,
        },
        abilities: vec![Ability::Event(Effect::Effect(StandardEffect::Negate {
            target: Predicate::Enemy(CardPredicate::Dream),
        }))],
        revealed_to_owner: false,
        revealed_to_opponent: false,
        targets: vec![],
        turn_entered_current_zone: TurnData::default(),
    });

    battle.cards.create_card(CardData {
        id: CardId::default(),
        owner: player_name,
        zone: Zone::Deck,
        object_id: ObjectId::default(),
        properties: CardProperties {
            spark: None,
            cost: Some(Energy(1)),
            card_type: CardType::Event,
            is_fast: true,
        },
        abilities: vec![Ability::Event(Effect::Effect(StandardEffect::NegateUnlessPaysCost {
            target: Predicate::Enemy(CardPredicate::Event),
            cost: Cost::Energy(Energy(2)),
        }))],
        revealed_to_owner: false,
        revealed_to_opponent: false,
        targets: vec![],
        turn_entered_current_zone: TurnData::default(),
    });
}
