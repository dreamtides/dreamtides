use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::standard_effect::StandardEffect;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle::effect_source::EffectSource;
use battle_data::battle_cards::card_id::{CharacterId, StackCardId};
use core_data::card_types::CardType;

#[derive(Debug, Clone)]
pub struct CharacterPredicate(pub Predicate);

#[derive(Debug, Clone)]
pub struct StackPredicate(pub Predicate);

/// Returns the set of characters on the battlefield matching this `predicate`.
pub fn matching_characters(
    battle: &BattleData,
    source: EffectSource,
    predicate: CharacterPredicate,
) -> Vec<CharacterId> {
    match predicate.0 {
        Predicate::Enemy(card_predicate) => on_battlefield(
            battle,
            source,
            battle
                .cards
                .battlefield(source.controller().opponent())
                .iter()
                .cloned()
                .collect::<Vec<_>>(),
            card_predicate,
        ),
        _ => todo!("Implement {:?}", predicate),
    }
}

/// Returns the set of cards on the stack matching this `predicate`.
pub fn matching_cards_on_stack(
    battle: &BattleData,
    source: EffectSource,
    predicate: StackPredicate,
) -> Vec<StackCardId> {
    match predicate.0 {
        Predicate::Enemy(card_predicate) => {
            let enemy_cards = battle
                .cards
                .stack()
                .iter()
                .filter(|&&id| {
                    battle
                        .cards
                        .card(id)
                        .map_or(false, |card| card.controller() == source.controller().opponent())
                })
                .cloned()
                .collect::<Vec<_>>();
            on_stack(battle, source, enemy_cards, card_predicate)
        }
        _ => todo!("Implement {:?}", predicate),
    }
}

/// Returns true if a standard effect requires a target to resolve.
pub fn has_targets(effect: &StandardEffect) -> bool {
    get_character_target_predicate(effect).is_some() || get_stack_target_predicate(effect).is_some()
}

/// Extracts a character target predicate from a standard effect, if any.
pub fn get_character_target_predicate(effect: &StandardEffect) -> Option<CharacterPredicate> {
    let predicate = match effect {
        StandardEffect::AbandonAndGainEnergyForSpark { target, .. } => Some(target.clone()),
        StandardEffect::AbandonAtEndOfTurn { target } => Some(target.clone()),
        StandardEffect::BanishCharacter { target } => Some(target.clone()),
        StandardEffect::BanishCharacterUntilLeavesPlay { target, .. } => Some(target.clone()),
        StandardEffect::BanishUntilNextMain { target } => Some(target.clone()),
        StandardEffect::BanishCollection { target, .. } => Some(target.clone()),
        StandardEffect::Copy { target } => Some(target.clone()),
        StandardEffect::DisableActivatedAbilitiesWhileInPlay { target } => Some(target.clone()),
        StandardEffect::DissolveCharacter { target } => Some(target.clone()),
        StandardEffect::DissolveCharactersCount { target, .. } => Some(target.clone()),
        StandardEffect::DissolveCharactersQuantity { target, .. } => Some(target.clone()),
        StandardEffect::GainControl { target } => Some(target.clone()),
        StandardEffect::GainsAegisThisTurn { target } => Some(target.clone()),
        StandardEffect::GainsReclaimUntilEndOfTurn { target, .. } => Some(target.clone()),
        StandardEffect::GainsSpark { target, .. } => Some(target.clone()),
        StandardEffect::GainsSparkForQuantity { target, .. } => Some(target.clone()),
        StandardEffect::GainsSparkUntilYourNextMainForEach { target, .. } => Some(target.clone()),
        StandardEffect::MaterializeCharacterAtEndOfTurn { target } => Some(target.clone()),
        StandardEffect::MaterializeSilentCopy { target, .. } => Some(target.clone()),
        StandardEffect::PutOnTopOfEnemyDeck { target } => Some(target.clone()),
        StandardEffect::ReturnToHand { target } => Some(target.clone()),
        _ => None,
    }?;
    Some(CharacterPredicate(predicate))
}

/// Extracts a stack target predicate from a standard effect, if any.
pub fn get_stack_target_predicate(effect: &StandardEffect) -> Option<StackPredicate> {
    let predicate = match effect {
        StandardEffect::Negate { target, .. } => Some(target.clone()),
        StandardEffect::NegateUnlessPaysCost { target, .. } => Some(target.clone()),
        _ => None,
    }?;
    Some(StackPredicate(predicate))
}

/// Returns the set of characters on the battlefield from `collection` which
/// match `predicate`.
fn on_battlefield(
    _battle: &BattleData,
    _source: EffectSource,
    collection: Vec<CharacterId>,
    predicate: CardPredicate,
) -> Vec<CharacterId> {
    match predicate {
        CardPredicate::Card | CardPredicate::Character => collection,
        _ => todo!("Implement {:?}", predicate),
    }
}

/// Returns the set of cards on the stack from `collection` which match
/// `predicate`.
fn on_stack(
    battle: &BattleData,
    _source: EffectSource,
    collection: Vec<StackCardId>,
    predicate: CardPredicate,
) -> Vec<StackCardId> {
    match predicate {
        CardPredicate::Card | CardPredicate::Dream => collection,
        CardPredicate::Event => collection
            .iter()
            .filter(|&&id| {
                battle
                    .cards
                    .card(id)
                    .map_or(false, |card| card.properties.card_type == CardType::Event)
            })
            .cloned()
            .collect(),
        _ => todo!("Implement {:?}", predicate),
    }
}
