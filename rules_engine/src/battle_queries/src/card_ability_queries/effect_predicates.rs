use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::standard_effect::StandardEffect;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CharacterId, StackCardId};
use battle_state::battle_cards::card_set::CardSet;
use battle_state::core::effect_source::EffectSource;
use core_data::card_types::CardType;

use crate::battle_card_queries::card_properties;

/// Returns the set of characters on the battlefield matching this `predicate`.
pub fn matching_characters(
    battle: &BattleState,
    source: EffectSource,
    predicate: &Predicate,
) -> CardSet<CharacterId> {
    match predicate {
        Predicate::Enemy(card_predicate) => {
            let battlefield = battle.cards.battlefield(source.controller().opponent()).clone();
            on_battlefield(battle, source, battlefield, card_predicate)
        }
        _ => todo!("Implement {:?}", predicate),
    }
}

/// Returns the set of cards on the stack matching this `predicate`.
pub fn matching_cards_on_stack(
    battle: &BattleState,
    source: EffectSource,
    predicate: &Predicate,
) -> CardSet<StackCardId> {
    match predicate {
        Predicate::Enemy(card_predicate) => {
            let battlefield = battle.cards.stack_set(source.controller().opponent()).clone();
            on_stack(battle, source, battlefield, card_predicate)
        }
        _ => todo!("Implement {:?}", predicate),
    }
}

/// Returns true if a standard effect requires a target to resolve.
pub fn has_targets(effect: &StandardEffect) -> bool {
    get_target_predicate(effect).is_some()
}

/// Returns the targeting predicate required to resolve a standard effect, if
/// any.
pub fn get_target_predicate(effect: &StandardEffect) -> Option<&Predicate> {
    get_character_target_predicate(effect).or_else(|| get_stack_target_predicate(effect))
}

/// Extracts a character target predicate from a standard effect, if any.
pub fn get_character_target_predicate(effect: &StandardEffect) -> Option<&Predicate> {
    match effect {
        StandardEffect::AbandonAndGainEnergyForSpark { target, .. } => Some(target),
        StandardEffect::AbandonAtEndOfTurn { target } => Some(target),
        StandardEffect::BanishCharacter { target } => Some(target),
        StandardEffect::BanishCharacterUntilLeavesPlay { target, .. } => Some(target),
        StandardEffect::BanishUntilNextMain { target } => Some(target),
        StandardEffect::BanishCollection { target, .. } => Some(target),
        StandardEffect::Copy { target } => Some(target),
        StandardEffect::DisableActivatedAbilitiesWhileInPlay { target } => Some(target),
        StandardEffect::DissolveCharacter { target } => Some(target),
        StandardEffect::DissolveCharactersCount { target, .. } => Some(target),
        StandardEffect::DissolveCharactersQuantity { target, .. } => Some(target),
        StandardEffect::GainControl { target } => Some(target),
        StandardEffect::GainsAegisThisTurn { target } => Some(target),
        StandardEffect::GainsReclaimUntilEndOfTurn { target, .. } => Some(target),
        StandardEffect::GainsSpark { target, .. } => Some(target),
        StandardEffect::GainsSparkForQuantity { target, .. } => Some(target),
        StandardEffect::GainsSparkUntilYourNextMainForEach { target, .. } => Some(target),
        StandardEffect::MaterializeCharacterAtEndOfTurn { target } => Some(target),
        StandardEffect::MaterializeSilentCopy { target, .. } => Some(target),
        StandardEffect::PutOnTopOfEnemyDeck { target } => Some(target),
        StandardEffect::ReturnToHand { target } => Some(target),
        _ => None,
    }
}

/// Extracts a stack target predicate from a standard effect, if any.
pub fn get_stack_target_predicate(effect: &StandardEffect) -> Option<&Predicate> {
    match effect {
        StandardEffect::Counterspell { target, .. } => Some(target),
        StandardEffect::CounterspellUnlessPaysCost { target, .. } => Some(target),
        _ => None,
    }
}

/// Returns all characters from `collection` which match a `predicate`.
fn on_battlefield(
    _battle: &BattleState,
    _source: EffectSource,
    collection: CardSet<CharacterId>,
    predicate: &CardPredicate,
) -> CardSet<CharacterId> {
    match predicate {
        CardPredicate::Card | CardPredicate::Character => collection,
        _ => todo!("Implement {:?}", predicate),
    }
}

/// Returns all stack cards from `collection` which match a `predicate`.
fn on_stack(
    battle: &BattleState,
    _source: EffectSource,
    collection: CardSet<StackCardId>,
    predicate: &CardPredicate,
) -> CardSet<StackCardId> {
    match predicate {
        CardPredicate::Card | CardPredicate::CardOnStack => collection,
        CardPredicate::Event => {
            let mut events = CardSet::default();
            for id in collection.iter() {
                if card_properties::card_type(battle, id) == CardType::Event {
                    events.insert(id);
                }
            }
            events
        }
        _ => todo!("Implement {:?}", predicate),
    }
}
