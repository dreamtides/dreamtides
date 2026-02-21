use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::standard_effect::StandardEffect;
use rlf::Phrase;
use strings::strings;

use crate::serializer::predicate_serializer;

/// Returns auto-generated prompt text for a standard effect that requires
/// target selection, or None if the effect does not require targeting.
pub fn serialize_prompt(effect: &StandardEffect) -> Option<String> {
    if let Some(text) = serialize_character_prompt(effect) {
        return Some(text);
    }
    if let Some(text) = serialize_stack_prompt(effect) {
        return Some(text);
    }
    serialize_void_prompt(effect)
}

fn serialize_character_prompt(effect: &StandardEffect) -> Option<String> {
    let (target, action) = match effect {
        StandardEffect::DissolveCharacter { target }
        | StandardEffect::DissolveCharactersCount { target, .. }
        | StandardEffect::DissolveCharactersQuantity { target, .. } => {
            (target, strings::dissolve())
        }
        StandardEffect::BanishCharacter { target }
        | StandardEffect::BanishCharacterUntilLeavesPlay { target, .. }
        | StandardEffect::BanishUntilNextMain { target }
        | StandardEffect::BanishCollection { target, .. } => (target, strings::banish()),
        StandardEffect::BanishThenMaterialize { target, .. } => {
            (target, strings::prompt_action_banish_then_materialize())
        }
        StandardEffect::Copy { target } | StandardEffect::MaterializeSilentCopy { target, .. } => {
            (target, strings::prompt_action_copy())
        }
        StandardEffect::DisableActivatedAbilitiesWhileInPlay { target } => {
            (target, strings::prompt_action_disable_abilities())
        }
        StandardEffect::GainControl { target } => (target, strings::prompt_action_gain_control()),
        StandardEffect::GainEnergyEqualToCost { target } => {
            (target, strings::prompt_action_gain_energy())
        }
        StandardEffect::GainsReclaim { target, .. } => {
            (target, strings::prompt_action_give_reclaim())
        }
        StandardEffect::GainsSpark { target, .. }
        | StandardEffect::GainsSparkForQuantity { target, .. }
        | StandardEffect::GainsSparkUntilYourNextMainForEach { target, .. } => {
            (target, strings::prompt_action_give_spark())
        }
        StandardEffect::MaterializeCharacterAtEndOfTurn { target }
        | StandardEffect::MaterializeCollection { target, .. } => (target, strings::materialize()),
        StandardEffect::PreventDissolveThisTurn { target } => {
            (target, strings::prompt_action_protect())
        }
        StandardEffect::PutOnTopOfEnemyDeck { target } => {
            (target, strings::prompt_action_put_on_enemy_deck())
        }
        StandardEffect::ReturnToHand { target } => {
            (target, strings::prompt_action_return_to_hand())
        }
        StandardEffect::AbandonAndGainEnergyForSpark { target, .. } => {
            (target, strings::prompt_action_abandon())
        }
        _ => return None,
    };

    let target_phrase = serialize_prompt_predicate(target);
    Some(strings::prompt_choose_to_action(target_phrase, action).to_string())
}

fn serialize_stack_prompt(effect: &StandardEffect) -> Option<String> {
    let (target, action) = match effect {
        StandardEffect::Counterspell { target }
        | StandardEffect::CounterspellUnlessPaysCost { target, .. } => (target, strings::prevent()),
        StandardEffect::AbandonAtEndOfTurn { target } => {
            (target, strings::prompt_action_abandon_at_end_of_turn())
        }
        StandardEffect::BanishWhenLeavesPlay { target } => {
            (target, strings::prompt_action_banish_when_leaves())
        }
        _ => return None,
    };

    let target_phrase = serialize_prompt_predicate(target);
    Some(strings::prompt_choose_to_action(target_phrase, action).to_string())
}

fn serialize_void_prompt(effect: &StandardEffect) -> Option<String> {
    match effect {
        StandardEffect::ReturnFromYourVoidToHand { target } => {
            let target_phrase = serialize_prompt_predicate(target);
            Some(strings::prompt_choose_target(target_phrase).to_string())
        }
        StandardEffect::ReturnUpToCountFromYourVoidToHand { target, count } => {
            let target_phrase = serialize_prompt_predicate(target);
            if *count <= 1 {
                Some(strings::prompt_choose_target(target_phrase).to_string())
            } else {
                Some(
                    strings::prompt_choose_up_to(
                        *count,
                        target_phrase,
                        strings::prompt_action_return(),
                    )
                    .to_string(),
                )
            }
        }
        _ => None,
    }
}

/// Serializes a predicate for prompt context.
///
/// Unlike the rules-text serializer, this always includes the full card type
/// name (e.g., "an enemy character" instead of just "an enemy").
fn serialize_prompt_predicate(predicate: &Predicate) -> Phrase {
    match predicate {
        Predicate::Enemy(CardPredicate::Character) => {
            strings::predicate_with_indefinite_article(strings::enemy_character())
        }
        _ => predicate_serializer::serialize_predicate(predicate),
    }
}
