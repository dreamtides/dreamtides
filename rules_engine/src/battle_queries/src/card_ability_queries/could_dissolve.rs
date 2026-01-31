use ability_data::effect::Effect;
use ability_data::predicate::Predicate;
use ability_data::standard_effect::StandardEffect;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CharacterId, StackCardId};
use battle_state::battle_cards::card_set::CardSet;
use battle_state::core::effect_source::EffectSource;

use crate::battle_card_queries::{card, card_properties};
use crate::card_ability_queries::effect_predicates::CharacterTargetingFlags;
use crate::card_ability_queries::{effect_predicates, effect_queries, target_predicates};

/// Returns the subset of `stack_cards` which have a dissolve effect that could
/// target characters matching `target_predicate`.
///
/// This is used for cards like "Prevent a played event which could dissolve an
/// ally" - we need to find stack cards with dissolve effects that could target
/// the specified characters.
pub fn filter_could_dissolve(
    battle: &BattleState,
    source: EffectSource,
    stack_cards: CardSet<StackCardId>,
    target_predicate: &Predicate,
) -> CardSet<StackCardId> {
    let target_characters = effect_predicates::matching_characters(
        battle,
        source,
        target_predicate,
        None,
        CharacterTargetingFlags::default(),
    );

    if target_characters.is_empty() {
        return CardSet::default();
    }

    let mut result = CardSet::default();
    for stack_id in stack_cards.iter() {
        if could_dissolve_any(battle, stack_id, &target_characters) {
            result.insert(stack_id);
        }
    }
    result
}

/// Returns true if the given stack card has a dissolve effect that could
/// target any of the characters in `target_characters`.
fn could_dissolve_any(
    battle: &BattleState,
    stack_id: StackCardId,
    target_characters: &CardSet<CharacterId>,
) -> bool {
    let ability_list = card::ability_list(battle, stack_id);
    let controller = card_properties::controller(battle, stack_id);

    for data in &ability_list.event_abilities {
        let stack_source = EffectSource::Event {
            controller,
            stack_card_id: stack_id,
            ability_number: data.ability_number,
        };

        if has_dissolve_effect_targeting(
            battle,
            stack_source,
            &data.ability.effect,
            target_characters,
        ) {
            return true;
        }
    }

    false
}

/// Returns true if the given effect contains a dissolve effect that could
/// target any of the characters in `target_characters`.
fn has_dissolve_effect_targeting(
    battle: &BattleState,
    source: EffectSource,
    effect: &Effect,
    target_characters: &CardSet<CharacterId>,
) -> bool {
    match effect {
        Effect::Effect(standard_effect) => has_dissolve_effect_targeting_standard(
            battle,
            source,
            standard_effect,
            target_characters,
        ),
        Effect::WithOptions(options) => has_dissolve_effect_targeting_standard(
            battle,
            source,
            &options.effect,
            target_characters,
        ),
        Effect::List(effects) => effects.iter().any(|e| {
            has_dissolve_effect_targeting_standard(battle, source, &e.effect, target_characters)
        }),
        Effect::ListWithOptions(list) => list.effects.iter().any(|e| {
            has_dissolve_effect_targeting_standard(battle, source, &e.effect, target_characters)
        }),
        Effect::Modal(choices) => choices.iter().any(|choice| {
            has_dissolve_effect_targeting(battle, source, &choice.effect, target_characters)
        }),
    }
}

/// Returns true if the given standard effect is a dissolve effect that could
/// target any of the characters in `target_characters`.
fn has_dissolve_effect_targeting_standard(
    battle: &BattleState,
    source: EffectSource,
    effect: &StandardEffect,
    target_characters: &CardSet<CharacterId>,
) -> bool {
    if !effect_queries::is_dissolve_effect(effect) {
        return false;
    }

    if let Some(predicate) = target_predicates::get_character_target_predicate(effect) {
        let mut dissolvable = effect_predicates::matching_characters(
            battle,
            source,
            predicate,
            None,
            CharacterTargetingFlags { for_dissolve: true },
        );
        dissolvable.intersect_with(target_characters);
        !dissolvable.is_empty()
    } else {
        false
    }
}
