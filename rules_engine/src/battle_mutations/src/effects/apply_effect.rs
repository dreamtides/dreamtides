use ability_data::effect::{Effect, EffectWithOptions};
use ability_data::standard_effect::StandardEffect;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle_cards::card_data::TargetId;
use battle_data::battle_cards::card_id::CharacterId;
use core_data::source::Source;

use crate::character_mutations::dissolve;

/// Applies an effect to the battle state.
pub fn apply(
    battle: &mut BattleData,
    source: Source,
    effect: Effect,
    targets: Vec<TargetId>,
) -> Option<()> {
    match effect {
        Effect::Effect(standard_effect) => {
            apply_standard_effect(battle, source, standard_effect, &targets)
        }
        Effect::WithOptions(effect_with_options) => {
            apply_effect_with_options(battle, source, effect_with_options, &targets)
        }
        Effect::List(effects) => apply_list_effect(battle, source, effects, &targets),
    }
}

fn apply_effect_with_options(
    _battle: &mut BattleData,
    _source: Source,
    _effect: EffectWithOptions,
    _targets: &[TargetId],
) -> Option<()> {
    todo!("Implement effect with options")
}

fn apply_list_effect(
    battle: &mut BattleData,
    source: Source,
    effects: Vec<EffectWithOptions>,
    targets: &[TargetId],
) -> Option<()> {
    for effect in effects {
        apply_effect_with_options(battle, source, effect, targets);
    }
    Some(())
}

fn apply_standard_effect(
    battle: &mut BattleData,
    source: Source,
    effect: StandardEffect,
    targets: &[TargetId],
) -> Option<()> {
    match effect {
        StandardEffect::DissolveCharacter { .. } => {
            for character_id in character_ids(targets) {
                dissolve::apply(battle, source, character_id);
            }
        }
        _ => todo!("Implement {:?}", effect),
    }
    Some(())
}

fn character_ids(targets: &[TargetId]) -> impl Iterator<Item = CharacterId> + '_ {
    targets.iter().filter_map(|target| match target {
        TargetId::Character(character_id) => Some(*character_id),
        _ => None,
    })
}
