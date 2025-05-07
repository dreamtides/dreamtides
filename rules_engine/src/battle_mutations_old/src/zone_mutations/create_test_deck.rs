use ability_data::ability::{Ability, EventAbility};
use ability_data::cost::Cost;
use ability_data::effect::Effect;
use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::quantity_expression_data::QuantityExpression;
use ability_data::standard_effect::StandardEffect;
use battle_data_old::battle::old_battle_data::BattleData;
use battle_data_old::battle::turn_data::TurnData;
use battle_data_old::battle_cards::card_data::CardData;
use battle_data_old::battle_cards::card_id::ObjectId;
use battle_data_old::battle_cards::card_identities;
use battle_data_old::battle_cards::card_properties::CardProperties;
use battle_data_old::battle_cards::zone::Zone;
use core_data::card_types::{CardType, CharacterType};
use core_data::identifiers::CardId;
use core_data::numerics::{Energy, Spark};
use core_data::types::PlayerName;
use rand::seq::SliceRandom;

pub fn add(battle: &mut BattleData, player: PlayerName) {
    let mut cards = create_cards(player);
    cards.shuffle(&mut battle.rng);
    for card in cards {
        battle.cards.create_card(card);
    }
}

fn create_cards(player_name: PlayerName) -> Vec<CardData> {
    let mut cards = Vec::new();

    for _ in 0..6 {
        cards.push(CardData {
            id: CardId::default(),
            identity: card_identities::MINSTREL_OF_FALLING_LIGHT,
            owner: player_name,
            zone: Zone::Deck,
            object_id: ObjectId::default(),
            properties: CardProperties {
                spark: Some(Spark(5)),
                cost: Some(Energy(2)),
                card_type: CardType::Character(CharacterType::Explorer),
                is_fast: false,
            },
            abilities: vec![],
            revealed_to_owner: false,
            revealed_to_opponent: false,
            targets: vec![],
            additional_cost_choices: vec![],
            turn_entered_current_zone: TurnData::default(),
        });
    }

    for _ in 0..3 {
        cards.push(CardData {
            id: CardId::default(),
            identity: card_identities::IMMOLATE,
            owner: player_name,
            zone: Zone::Deck,
            object_id: ObjectId::default(),
            properties: CardProperties {
                spark: None,
                cost: Some(Energy(2)),
                card_type: CardType::Event,
                is_fast: true,
            },
            abilities: vec![Ability::Event(EventAbility {
                additional_cost: None,
                effect: Effect::Effect(StandardEffect::DissolveCharacter {
                    target: Predicate::Enemy(CardPredicate::Character),
                }),
            })],
            revealed_to_owner: false,
            revealed_to_opponent: false,
            targets: vec![],
            additional_cost_choices: vec![],
            turn_entered_current_zone: TurnData::default(),
        });
    }

    for _ in 0..3 {
        cards.push(CardData {
            id: CardId::default(),
            identity: card_identities::ABOLISH,
            owner: player_name,
            zone: Zone::Deck,
            object_id: ObjectId::default(),
            properties: CardProperties {
                spark: None,
                cost: Some(Energy(2)),
                card_type: CardType::Event,
                is_fast: true,
            },
            abilities: vec![Ability::Event(EventAbility {
                additional_cost: None,
                effect: Effect::Effect(StandardEffect::Negate {
                    target: Predicate::Enemy(CardPredicate::Dream),
                }),
            })],
            revealed_to_owner: false,
            revealed_to_opponent: false,
            targets: vec![],
            additional_cost_choices: vec![],
            turn_entered_current_zone: TurnData::default(),
        });
    }

    for _ in 0..3 {
        cards.push(CardData {
            id: CardId::default(),
            identity: card_identities::RIPPLE_OF_DEFIANCE,
            owner: player_name,
            zone: Zone::Deck,
            object_id: ObjectId::default(),
            properties: CardProperties {
                spark: None,
                cost: Some(Energy(1)),
                card_type: CardType::Event,
                is_fast: true,
            },
            abilities: vec![Ability::Event(EventAbility {
                additional_cost: None,
                effect: Effect::Effect(StandardEffect::NegateUnlessPaysCost {
                    target: Predicate::Enemy(CardPredicate::Event),
                    cost: Cost::Energy(Energy(2)),
                }),
            })],
            revealed_to_owner: false,
            revealed_to_opponent: false,
            targets: vec![],
            additional_cost_choices: vec![],
            turn_entered_current_zone: TurnData::default(),
        });
    }

    for _ in 0..3 {
        cards.push(CardData {
            id: CardId::default(),
            identity: card_identities::DREAMSCATTER,
            owner: player_name,
            zone: Zone::Deck,
            object_id: ObjectId::default(),
            properties: CardProperties {
                spark: None,
                cost: Some(Energy(1)),
                card_type: CardType::Event,
                is_fast: true,
            },
            abilities: vec![Ability::Event(EventAbility {
                additional_cost: Some(Cost::SpendOneOrMoreEnergy),
                effect: Effect::Effect(StandardEffect::DrawCardsForEach {
                    count: 1,
                    for_each: QuantityExpression::ForEachEnergySpentOnThisCard,
                }),
            })],
            revealed_to_owner: false,
            revealed_to_opponent: false,
            targets: vec![],
            additional_cost_choices: vec![],
            turn_entered_current_zone: TurnData::default(),
        });
    }

    cards
}
