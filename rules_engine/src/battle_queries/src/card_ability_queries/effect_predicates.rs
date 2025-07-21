use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::standard_effect::StandardEffect;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardId, CharacterId, StackCardId, VoidCardId};
use battle_state::battle_cards::card_set::CardSet;
use battle_state::core::effect_source::EffectSource;
use core_data::card_types::CardType;

use crate::battle_card_queries::card_properties;

/// Returns the set of characters on the battlefield matching this `predicate`.
pub fn matching_characters(
    battle: &BattleState,
    source: EffectSource,
    predicate: &Predicate,
    that_card: Option<CardId>,
) -> CardSet<CharacterId> {
    match predicate {
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
        Predicate::YourVoid(_) | Predicate::EnemyVoid(_) => CardSet::default(),
        _ => todo!("Implement {:?}", predicate),
    }
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

/// Returns true if a standard effect requires a target to resolve.
pub fn has_targets(effect: &StandardEffect) -> bool {
    get_target_predicate(effect).is_some()
}

/// Returns the targeting predicate required to resolve a standard effect, if
/// any.
pub fn get_target_predicate(effect: &StandardEffect) -> Option<&Predicate> {
    get_character_target_predicate(effect)
        .or_else(|| get_stack_target_predicate(effect))
        .or_else(|| get_void_target_predicate(effect))
}

/// Extracts a character target predicate from a standard effect, if any.
pub fn get_character_target_predicate(effect: &StandardEffect) -> Option<&Predicate> {
    match effect {
        StandardEffect::AbandonAndGainEnergyForSpark { target, .. } => Some(target),
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
        StandardEffect::PreventDissolveThisTurn { target } => Some(target),
        StandardEffect::PutOnTopOfEnemyDeck { target } => Some(target),
        StandardEffect::ReturnToHand { target } => Some(target),

        StandardEffect::AbandonAtEndOfTurn { .. } => None,
        StandardEffect::BanishCardsFromEnemyVoid { .. } => None,
        StandardEffect::BanishEnemyVoid => None,
        StandardEffect::BanishWhenLeavesPlay { .. } => None,
        StandardEffect::CardsInVoidGainReclaimThisTurn { .. } => None,
        StandardEffect::CopyNextPlayed { .. } => None,
        StandardEffect::Counterspell { .. } => None,
        StandardEffect::CounterspellUnlessPaysCost { .. } => None,
        StandardEffect::CreateTriggerUntilEndOfTurn { .. } => None,
        StandardEffect::DiscardCardFromEnemyHand { .. } => None,
        StandardEffect::DiscardCardFromEnemyHandThenTheyDraw { .. } => None,
        StandardEffect::DiscardCards { .. } => None,
        StandardEffect::Discover { .. } => None,
        StandardEffect::DiscoverAndThenMaterialize { .. } => None,
        StandardEffect::DoubleYourEnergy => None,
        StandardEffect::DrawCards { .. } => None,
        StandardEffect::DrawCardsForEach { .. } => None,
        StandardEffect::DrawMatchingCard { .. } => None,
        StandardEffect::EachMatchingGainsSparkForEach { .. } => None,
        StandardEffect::EachMatchingGainsSparkUntilNextMain { .. } => None,
        StandardEffect::EachPlayerAbandonsCharacters { .. } => None,
        StandardEffect::EachPlayerDiscardCards { .. } => None,
        StandardEffect::EnemyGainsPoints { .. } => None,
        StandardEffect::EnemyGainsPointsEqualToItsSpark => None,
        StandardEffect::EnemyLosesPoints { .. } => None,
        StandardEffect::Foresee { .. } => None,
        StandardEffect::GainEnergy { .. } => None,
        StandardEffect::GainEnergyForEach { .. } => None,
        StandardEffect::GainPoints { .. } => None,
        StandardEffect::GainPointsForEach { .. } => None,
        StandardEffect::GainTwiceThatMuchEnergyInstead => None,
        StandardEffect::Kindle { .. } => None,
        StandardEffect::LosePoints { .. } => None,
        StandardEffect::MaterializeCharacter { .. } => None,
        StandardEffect::MaterializeCharacterFromVoid { .. } => None,
        StandardEffect::MaterializeRandomFromDeck { .. } => None,
        StandardEffect::NoEffect => None,
        StandardEffect::OpponentPaysCost { .. } => None,
        StandardEffect::PayCost { .. } => None,
        StandardEffect::PutCardsFromVoidOnTopOfDeck { .. } => None,
        StandardEffect::PutCardsFromYourDeckIntoVoid { .. } => None,
        StandardEffect::ReturnCharactersToHandDrawCardForEach { .. } => None,
        StandardEffect::ReturnFromYourVoidToHand { .. } => None,
        StandardEffect::ReturnFromYourVoidToPlay { .. } => None,
        StandardEffect::ReturnUpToCountFromYourVoidToHand { .. } => None,
        StandardEffect::ShuffleHandAndDeckAndDraw { .. } => None,
        StandardEffect::SparkBecomes { .. } => None,
        StandardEffect::SpendAllEnergyDissolveEnemy => None,
        StandardEffect::SpendAllEnergyDrawAndDiscard => None,
        StandardEffect::TakeExtraTurn => None,
        StandardEffect::ThenMaterializeIt => None,
        StandardEffect::TriggerJudgmentAbility { .. } => None,
        StandardEffect::YouWinTheGame => None,
    }
}

/// Extracts a stack target predicate from a standard effect, if any.
pub fn get_stack_target_predicate(effect: &StandardEffect) -> Option<&Predicate> {
    match effect {
        StandardEffect::AbandonAtEndOfTurn { target } => Some(target),
        StandardEffect::BanishWhenLeavesPlay { target } => Some(target),
        StandardEffect::Counterspell { target, .. } => Some(target),
        StandardEffect::CounterspellUnlessPaysCost { target, .. } => Some(target),
        _ => None,
    }
}

/// Extracts a void target predicate from a standard effect, if any.
pub fn get_void_target_predicate(effect: &StandardEffect) -> Option<&Predicate> {
    match effect {
        StandardEffect::ReturnFromYourVoidToHand { target } => Some(target),
        StandardEffect::ReturnUpToCountFromYourVoidToHand { target, .. } => Some(target),
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
