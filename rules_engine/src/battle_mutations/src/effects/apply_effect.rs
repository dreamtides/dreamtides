use ability_data::effect::Effect;
use battle_queries::battle_card_queries::stack_card_queries;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle_cards::stack_card_state::EffectTargets;
use battle_state::core::effect_source::EffectSource;

use crate::effects::apply_standard_effect;

/// Marker struct indicating that an effect was applied to the battle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EffectWasApplied;

/// Applies an effect to the given [BattleState]. If the effect requires a
/// target, it can be provided via `requested_targets`. Targeted effects with no
/// targets or invalid targets will be ignored. Returns `Some(EffectWasApplied)`
/// if any visible changes to the battle state were made as a result of this
/// effect.
///
/// # Arguments
///
/// * `battle` - The current battle state.
/// * `source` - The source of the effect.
/// * `effect` - The effect to apply.
/// * `requested_targets` - The targets for the effect.
pub fn execute(
    battle: &mut BattleState,
    source: EffectSource,
    effect: &Effect,
    requested_targets: Option<&EffectTargets>,
) -> Option<EffectWasApplied> {
    let targets = stack_card_queries::validate_targets(battle, requested_targets);
    let result = match effect {
        Effect::Effect(standard) => apply_standard_effect::apply(battle, source, standard, targets),
        _ => todo!("Implement this"),
    };

    if !battle.cards.has_stack() {
        // If this effect removed the last card from the stack, stack priority
        // is ended.
        battle.stack_priority = None;
    }

    result
}
