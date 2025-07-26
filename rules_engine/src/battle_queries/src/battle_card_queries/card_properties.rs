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
        CardName::TestVanillaCharacter => "Character".to_string(),
        CardName::TestDissolve => "Dissolve".to_string(),
        CardName::TestNamedDissolve => "Immolate".to_string(),
        CardName::TestCounterspellUnlessPays => "Ripple of Defiance".to_string(),
        CardName::TestCounterspell => "Abolish".to_string(),
        CardName::TestVariableEnergyDraw => "Dreamscatter".to_string(),
        CardName::TestDrawOne => "Draw One".to_string(),
        CardName::TestTriggerGainSparkWhenMaterializeAnotherCharacter => {
            "Materialize Gain Spark".to_string()
        }
        CardName::TestTriggerGainSparkOnPlayCardEnemyTurn => "Sundown Surfer".to_string(),
        CardName::TestTriggerGainTwoSparkOnPlayCardEnemyTurn => "Play Enemy Gain Spark".to_string(),
        CardName::TestActivatedAbilityDrawCard => "Activated".to_string(),
        CardName::TestMultiActivatedAbilityDrawCardCharacter => "Multi Activated".to_string(),
        CardName::TestFastActivatedAbilityDrawCardCharacter => "Fast Activated".to_string(),
        CardName::TestFastMultiActivatedAbilityDrawCardCharacter => {
            "Minstrel of Falling Light".to_string()
        }
        CardName::TestActivatedAbilityDissolveCharacter => "Dissolve Character".to_string(),
        CardName::TestDualActivatedAbilityCharacter => "Dual Abilities".to_string(),
        CardName::TestForeseeOne => "Foresee 1".to_string(),
        CardName::TestForeseeTwo => "Foresee 2".to_string(),
        CardName::TestForeseeOneDrawACard => "Foresee 1 Draw 1".to_string(),
        CardName::TestDrawOneReclaim => "Draw 1 Reclaim".to_string(),
        CardName::TestReturnVoidCardToHand => "Return Void Card".to_string(),
        CardName::TestReturnOneOrTwoVoidEventCardsToHand => "Archive of the Forgotten".to_string(),
        CardName::TestModalDrawOneOrDrawTwo => "Modal Draw".to_string(),
        CardName::TestModalDrawOneOrDissolveEnemy => "Modal Draw & Dissolve".to_string(),
        CardName::TestReturnToHand => "Return to Hand".to_string(),
        CardName::TestPreventDissolveThisTurn => "Together Against the Tide".to_string(),
        CardName::TestForeseeOneReclaim => "Foresee 1 Reclaim".to_string(),
        CardName::TestForeseeOneDrawReclaim => "Guiding Light".to_string(),
        CardName::TestModalReturnToHandOrDrawTwo => "Break the Sequence".to_string(),
        CardName::TestCounterspellCharacter => "Cragfall".to_string(),
    }
}

/// Returns the energy cost of a card, or 0 if it has no energy cost.
///
/// Cards may not have an energy cost due to their card type (e.g. dreamwell
/// cards) or may not have a cost due to their ability (e.g. modal cards).
pub fn converted_energy_cost(battle: &BattleState, card_id: impl CardIdType) -> Energy {
    base_energy_cost_for_id(battle, card_id).unwrap_or_default()
}

/// Returns the base energy cost of a card as in [base_energy_cost].
pub fn base_energy_cost_for_id(battle: &BattleState, card_id: impl CardIdType) -> Option<Energy> {
    base_energy_cost(card::get(battle, card_id).name)
}

/// Returns the base energy cost of a card specified in the card definition, or
/// None if it has no energy cost (e.g. modal cards).
pub fn base_energy_cost(name: CardName) -> Option<Energy> {
    match name {
        CardName::TestVanillaCharacter => Some(Energy(2)),
        CardName::TestDissolve => Some(Energy(2)),
        CardName::TestNamedDissolve => Some(Energy(2)),
        CardName::TestCounterspellUnlessPays => Some(Energy(1)),
        CardName::TestCounterspell => Some(Energy(2)),
        CardName::TestCounterspellCharacter => Some(Energy(2)),
        CardName::TestVariableEnergyDraw => Some(Energy(2)),
        CardName::TestDrawOne => Some(Energy(0)),
        CardName::TestTriggerGainSparkWhenMaterializeAnotherCharacter => Some(Energy(0)),
        CardName::TestTriggerGainSparkOnPlayCardEnemyTurn => Some(Energy(2)),
        CardName::TestTriggerGainTwoSparkOnPlayCardEnemyTurn => Some(Energy(2)),
        CardName::TestActivatedAbilityDrawCard => Some(Energy(2)),
        CardName::TestMultiActivatedAbilityDrawCardCharacter => Some(Energy(0)),
        CardName::TestFastActivatedAbilityDrawCardCharacter => Some(Energy(0)),
        CardName::TestFastMultiActivatedAbilityDrawCardCharacter => Some(Energy(2)),
        CardName::TestActivatedAbilityDissolveCharacter => Some(Energy(3)),
        CardName::TestDualActivatedAbilityCharacter => Some(Energy(4)),
        CardName::TestForeseeOne => Some(Energy(1)),
        CardName::TestForeseeTwo => Some(Energy(1)),
        CardName::TestForeseeOneDrawACard => Some(Energy(1)),
        CardName::TestDrawOneReclaim => Some(Energy(2)),
        CardName::TestForeseeOneReclaim => Some(Energy(1)),
        CardName::TestForeseeOneDrawReclaim => Some(Energy(1)),
        CardName::TestReturnVoidCardToHand => Some(Energy(1)),
        CardName::TestReturnOneOrTwoVoidEventCardsToHand => Some(Energy(4)),
        CardName::TestModalDrawOneOrDrawTwo => None,
        CardName::TestModalDrawOneOrDissolveEnemy => None,
        CardName::TestModalReturnToHandOrDrawTwo => None,
        CardName::TestReturnToHand => Some(Energy(1)),
        CardName::TestPreventDissolveThisTurn => Some(Energy(1)),
    }
}

/// Returns the player who currently controls a given card.
pub fn controller(battle: &BattleState, card_id: impl CardIdType) -> PlayerName {
    card::get(battle, card_id).owner
}

pub fn spark(battle: &BattleState, controller: PlayerName, id: CharacterId) -> Option<Spark> {
    battle.cards.spark(controller, id)
}

pub fn base_spark_for_id(battle: &BattleState, card_id: impl CardIdType) -> Option<Spark> {
    base_spark(card::get(battle, card_id).name)
}

pub fn base_spark(name: CardName) -> Option<Spark> {
    match name {
        CardName::TestVanillaCharacter => Some(Spark(5)),
        CardName::TestTriggerGainSparkWhenMaterializeAnotherCharacter => Some(Spark(5)),
        CardName::TestTriggerGainSparkOnPlayCardEnemyTurn => Some(Spark(1)),
        CardName::TestTriggerGainTwoSparkOnPlayCardEnemyTurn => Some(Spark(5)),
        CardName::TestActivatedAbilityDrawCard => Some(Spark(3)),
        CardName::TestMultiActivatedAbilityDrawCardCharacter => Some(Spark(3)),
        CardName::TestFastActivatedAbilityDrawCardCharacter => Some(Spark(3)),
        CardName::TestFastMultiActivatedAbilityDrawCardCharacter => Some(Spark(2)),
        CardName::TestActivatedAbilityDissolveCharacter => Some(Spark(4)),
        CardName::TestDualActivatedAbilityCharacter => Some(Spark(3)),
        _ => None,
    }
}

pub fn card_type(battle: &BattleState, card_id: impl CardIdType) -> CardType {
    match card::get(battle, card_id).name {
        CardName::TestVanillaCharacter => CardType::Character(CharacterType::Musician),
        CardName::TestDissolve => CardType::Event,
        CardName::TestNamedDissolve => CardType::Event,
        CardName::TestCounterspellUnlessPays => CardType::Event,
        CardName::TestCounterspell => CardType::Event,
        CardName::TestVariableEnergyDraw => CardType::Event,
        CardName::TestDrawOne => CardType::Event,
        CardName::TestTriggerGainSparkWhenMaterializeAnotherCharacter => {
            CardType::Character(CharacterType::Musician)
        }
        CardName::TestTriggerGainSparkOnPlayCardEnemyTurn => {
            CardType::Character(CharacterType::Visitor)
        }
        CardName::TestTriggerGainTwoSparkOnPlayCardEnemyTurn => {
            CardType::Character(CharacterType::Visitor)
        }
        CardName::TestActivatedAbilityDrawCard => CardType::Character(CharacterType::Warrior),
        CardName::TestMultiActivatedAbilityDrawCardCharacter => {
            CardType::Character(CharacterType::Warrior)
        }
        CardName::TestFastActivatedAbilityDrawCardCharacter => {
            CardType::Character(CharacterType::Warrior)
        }
        CardName::TestFastMultiActivatedAbilityDrawCardCharacter => {
            CardType::Character(CharacterType::Warrior)
        }
        CardName::TestActivatedAbilityDissolveCharacter => CardType::Character(CharacterType::Mage),
        CardName::TestDualActivatedAbilityCharacter => CardType::Character(CharacterType::Ancient),
        CardName::TestForeseeOne => CardType::Event,
        CardName::TestForeseeTwo => CardType::Event,
        CardName::TestForeseeOneDrawACard => CardType::Event,
        CardName::TestDrawOneReclaim => CardType::Event,
        CardName::TestReturnVoidCardToHand => CardType::Event,
        CardName::TestReturnOneOrTwoVoidEventCardsToHand => CardType::Event,
        CardName::TestModalDrawOneOrDrawTwo => CardType::Event,
        CardName::TestModalDrawOneOrDissolveEnemy => CardType::Event,
        CardName::TestReturnToHand => CardType::Event,
        CardName::TestPreventDissolveThisTurn => CardType::Event,
        CardName::TestCounterspellCharacter => CardType::Event,
        CardName::TestForeseeOneReclaim => CardType::Event,
        CardName::TestForeseeOneDrawReclaim => CardType::Event,
        CardName::TestModalReturnToHandOrDrawTwo => CardType::Event,
    }
}

pub fn is_fast(battle: &BattleState, card_id: impl CardIdType) -> bool {
    match card::get(battle, card_id).name {
        CardName::TestVanillaCharacter => false,
        CardName::TestDissolve => true,
        CardName::TestNamedDissolve => true,
        CardName::TestCounterspellUnlessPays => true,
        CardName::TestCounterspell => true,
        CardName::TestVariableEnergyDraw => true,
        CardName::TestDrawOne => true,
        CardName::TestTriggerGainSparkWhenMaterializeAnotherCharacter => false,
        CardName::TestTriggerGainSparkOnPlayCardEnemyTurn => true,
        CardName::TestTriggerGainTwoSparkOnPlayCardEnemyTurn => true,
        CardName::TestActivatedAbilityDrawCard => false,
        CardName::TestMultiActivatedAbilityDrawCardCharacter => false,
        CardName::TestFastActivatedAbilityDrawCardCharacter => false,
        CardName::TestFastMultiActivatedAbilityDrawCardCharacter => false,
        CardName::TestActivatedAbilityDissolveCharacter => false,
        CardName::TestDualActivatedAbilityCharacter => false,
        CardName::TestForeseeOne => true,
        CardName::TestForeseeTwo => true,
        CardName::TestForeseeOneDrawACard => true,
        CardName::TestDrawOneReclaim => true,
        CardName::TestReturnVoidCardToHand => true,
        CardName::TestReturnOneOrTwoVoidEventCardsToHand => false,
        CardName::TestModalDrawOneOrDrawTwo => false,
        CardName::TestModalDrawOneOrDissolveEnemy => true,
        CardName::TestReturnToHand => true,
        CardName::TestPreventDissolveThisTurn => true,
        CardName::TestCounterspellCharacter => true,
        CardName::TestForeseeOneReclaim => true,
        CardName::TestForeseeOneDrawReclaim => true,
        CardName::TestModalReturnToHandOrDrawTwo => true,
    }
}
