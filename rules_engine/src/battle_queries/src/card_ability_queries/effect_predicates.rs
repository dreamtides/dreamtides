use ability_data::predicate::{CardPredicate, Predicate};
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardId, CharacterId, StackCardId, VoidCardId};
use battle_state::battle_cards::card_set::CardSet;
use battle_state::core::effect_source::EffectSource;
use core_data::card_types::CardType;

use crate::battle_card_queries::card_properties;
use crate::card_ability_queries::{could_dissolve, effect_queries};

/// Flags for querying cards matching a character targeting predicate.
#[derive(Debug, Clone, Copy, Default)]
pub struct CharacterTargetingFlags {
    /// Whether this predicate is for targeting a dissolve effect.
    ///
    /// Dissolve effects can be prevented on certain characters, causing those
    /// targets to be invalid.
    pub for_dissolve: bool,
}

/// Returns the set of characters on the battlefield matching this `predicate`.
pub fn matching_characters(
    battle: &BattleState,
    source: EffectSource,
    predicate: &Predicate,
    that_card: Option<CardId>,
    flags: CharacterTargetingFlags,
) -> CardSet<CharacterId> {
    let mut result = match predicate {
        Predicate::This => CardSet::of_maybe(
            source.card_id().and_then(|id| battle.cards.to_character_id(source.controller(), id)),
        ),
        Predicate::That => CardSet::of_maybe(
            that_card
                .and_then(|id| battle.cards.to_character_id(source.controller().opponent(), id)),
        ),
        Predicate::Your(card_predicate) => {
            let battlefield = battle.cards.battlefield(source.controller()).clone();
            on_battlefield(battle, source, battlefield, card_predicate)
        }
        Predicate::Enemy(card_predicate) => {
            let battlefield = battle.cards.battlefield(source.controller().opponent()).clone();
            on_battlefield(battle, source, battlefield, card_predicate)
        }
        Predicate::Another(card_predicate) => {
            let mut battlefield = battle.cards.battlefield(source.controller()).clone();
            if let Some(id) = source.card_id() {
                battlefield.remove(CharacterId(id));
            }
            on_battlefield(battle, source, battlefield, card_predicate)
        }
        Predicate::YourVoid(_) | Predicate::EnemyVoid(_) => CardSet::default(),
        _ => todo!("Implement {:?}", predicate),
    };

    if flags.for_dissolve {
        result.difference_with(&effect_queries::prevent_dissolved_set(battle));
    }

    result
}

/// Returns the set of cards on the stack matching this `predicate`.
pub fn matching_cards_on_stack(
    battle: &BattleState,
    source: EffectSource,
    predicate: &Predicate,
    that_card: Option<CardId>,
) -> CardSet<StackCardId> {
    match predicate {
        Predicate::This => CardSet::of_maybe(
            source.card_id().and_then(|id| battle.cards.to_stack_card_id(source.controller(), id)),
        ),
        Predicate::That => CardSet::of_maybe(
            that_card
                .and_then(|id| battle.cards.to_stack_card_id(source.controller().opponent(), id)),
        ),
        Predicate::Your(card_predicate) => {
            let battlefield = battle.cards.stack_set(source.controller()).clone();
            on_stack(battle, source, battlefield, card_predicate)
        }
        Predicate::Enemy(card_predicate) => {
            let battlefield = battle.cards.stack_set(source.controller().opponent()).clone();
            on_stack(battle, source, battlefield, card_predicate)
        }
        Predicate::YourVoid(_) | Predicate::EnemyVoid(_) => CardSet::default(),
        _ => todo!("Implement {:?}", predicate),
    }
}

/// Returns the set of cards in the void matching this `predicate`.
pub fn matching_cards_in_void(
    battle: &BattleState,
    source: EffectSource,
    predicate: &Predicate,
    that_card: Option<CardId>,
) -> CardSet<VoidCardId> {
    match predicate {
        Predicate::This => CardSet::of_maybe(
            source.card_id().and_then(|id| battle.cards.to_void_card_id(source.controller(), id)),
        ),
        Predicate::That => CardSet::of_maybe(
            that_card
                .and_then(|id| battle.cards.to_void_card_id(source.controller().opponent(), id)),
        ),
        Predicate::Your(_) => CardSet::default(),
        Predicate::YourVoid(card_predicate) => {
            let void = battle.cards.void(source.controller()).clone();
            in_void(battle, source, void, card_predicate)
        }
        Predicate::EnemyVoid(card_predicate) => {
            let void = battle.cards.void(source.controller().opponent()).clone();
            in_void(battle, source, void, card_predicate)
        }
        _ => todo!("Implement {:?}", predicate),
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
    source: EffectSource,
    collection: CardSet<StackCardId>,
    predicate: &CardPredicate,
) -> CardSet<StackCardId> {
    match predicate {
        CardPredicate::Card => collection,
        CardPredicate::Event => {
            let mut events = CardSet::default();
            for id in collection.iter() {
                if card_properties::card_type(battle, id) == CardType::Event {
                    events.insert(id);
                }
            }
            events
        }
        CardPredicate::Character => {
            let mut characters = CardSet::default();
            for id in collection.iter() {
                if card_properties::card_type(battle, id) == CardType::Character {
                    characters.insert(id);
                }
            }
            characters
        }
        CardPredicate::CouldDissolve { target } => {
            could_dissolve::filter_could_dissolve(battle, source, collection, target)
        }
        _ => todo!("Implement {:?}", predicate),
    }
}

/// Returns all void cards from `collection` which match a `predicate`.
fn in_void(
    battle: &BattleState,
    _source: EffectSource,
    collection: CardSet<VoidCardId>,
    predicate: &CardPredicate,
) -> CardSet<VoidCardId> {
    match predicate {
        CardPredicate::Card => collection,
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
