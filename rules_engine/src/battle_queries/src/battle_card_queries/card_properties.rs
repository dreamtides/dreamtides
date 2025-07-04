use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardIdType, CharacterId};
use core_data::card_types::{CardType, CharacterType};
use core_data::identifiers::CardName;
use core_data::numerics::{Energy, Spark};
use core_data::types::PlayerName;

use crate::battle_card_queries::card;

/// Returns the display name for a card
pub fn display_name(card_name: CardName) -> String {
    match card_name {
        CardName::TestVanillaCharacter => "Test Character".to_string(),
        CardName::TestDissolve => "Test Dissolve".to_string(),
        CardName::TestCounterspellUnlessPays => "Test Counterspell Unless Pays".to_string(),
        CardName::TestCounterspell => "Test Counterspell".to_string(),
        CardName::TestVariableEnergyDraw => "Test Variable Energy Draw".to_string(),
        CardName::TestDrawOne => "Test Draw One".to_string(),
        CardName::TestTriggerGainSparkOnPlayCardEnemyTurn => "Test Gain Trigger".to_string(),
    }
}

pub fn cost(battle: &BattleState, card_id: impl CardIdType) -> Option<Energy> {
    match card::get(battle, card_id).name {
        CardName::TestVanillaCharacter => Some(Energy(2)),
        CardName::TestDissolve => Some(Energy(2)),
        CardName::TestCounterspellUnlessPays => Some(Energy(1)),
        CardName::TestCounterspell => Some(Energy(2)),
        CardName::TestVariableEnergyDraw => Some(Energy(1)),
        CardName::TestDrawOne => Some(Energy(0)),
        CardName::TestTriggerGainSparkOnPlayCardEnemyTurn => Some(Energy(2)),
    }
}

/// Returns the player who currently controls a given card.
pub fn controller(battle: &BattleState, card_id: impl CardIdType) -> PlayerName {
    card::get(battle, card_id).owner
}

pub fn spark(battle: &BattleState, controller: PlayerName, id: CharacterId) -> Option<Spark> {
    battle.cards.spark(controller, id)
}

pub fn base_spark(battle: &BattleState, card_id: impl CardIdType) -> Option<Spark> {
    match card::get(battle, card_id).name {
        CardName::TestVanillaCharacter => Some(Spark(5)),
        CardName::TestTriggerGainSparkOnPlayCardEnemyTurn => Some(Spark(5)),
        _ => None,
    }
}

pub fn card_type(battle: &BattleState, card_id: impl CardIdType) -> CardType {
    match card::get(battle, card_id).name {
        CardName::TestVanillaCharacter => CardType::Character(CharacterType::Musician),
        CardName::TestDissolve => CardType::Event,
        CardName::TestCounterspellUnlessPays => CardType::Event,
        CardName::TestCounterspell => CardType::Event,
        CardName::TestVariableEnergyDraw => CardType::Event,
        CardName::TestDrawOne => CardType::Event,
        CardName::TestTriggerGainSparkOnPlayCardEnemyTurn => {
            CardType::Character(CharacterType::Visitor)
        }
    }
}

pub fn is_fast(battle: &BattleState, card_id: impl CardIdType) -> bool {
    match card::get(battle, card_id).name {
        CardName::TestVanillaCharacter => false,
        CardName::TestDissolve => true,
        CardName::TestCounterspellUnlessPays => true,
        CardName::TestCounterspell => true,
        CardName::TestVariableEnergyDraw => true,
        CardName::TestDrawOne => true,
        CardName::TestTriggerGainSparkOnPlayCardEnemyTurn => true,
    }
}
