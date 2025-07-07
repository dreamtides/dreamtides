use ability_data::effect::Effect;
use ability_data::quantity_expression_data::QuantityExpression;
use ability_data::standard_effect::StandardEffect;
use battle_queries::battle_card_queries::stack_card_queries;
use battle_queries::battle_player_queries::quantity_expression;
use battle_queries::battle_trace;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle_cards::stack_card_state::EffectTargets;
use battle_state::core::effect_source::EffectSource;
use core_data::numerics::Spark;

use crate::card_mutations::{counterspell, deck, spark};
use crate::character_mutations::dissolve;
use crate::effects::{counterspell_unless_pays_cost, pay_cost, targeting};

/// Applies an effect to the given [BattleState]. If the effect requires a
/// target, it can be provided via `requested_targets`. Targeted effects with no
/// targets or invalid targets will be ignored.
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
) {
    let targets = stack_card_queries::validate_targets(battle, requested_targets);
    match effect {
        Effect::Effect(standard) => apply_standard_effect(battle, source, standard, targets),
        _ => todo!("Implement this"),
    }

    if !battle.cards.has_stack() {
        // If this effect removed the last card from the stack, stack priority
        // is ended.
        battle.stack_priority = None;
    }
}

fn apply_standard_effect(
    battle: &mut BattleState,
    source: EffectSource,
    effect: &StandardEffect,
    targets: Option<&EffectTargets>,
) {
    battle_trace!("Applying effect", battle, effect, targets);
    match effect {
        StandardEffect::Counterspell { .. } => {
            counterspell(battle, source, targets);
        }
        StandardEffect::CounterspellUnlessPaysCost { cost, .. } => {
            counterspell_unless_pays_cost::execute(battle, source, targets, cost);
        }
        StandardEffect::DrawCards { count } => {
            deck::draw_cards(battle, source, source.controller(), *count);
        }
        StandardEffect::DrawCardsForEach { count, for_each } => {
            draw_cards_for_each(battle, source, *count, for_each);
        }
        StandardEffect::DissolveCharacter { .. } => {
            dissolve(battle, source, targets);
        }
        StandardEffect::GainsSpark { gains, .. } => {
            gains_spark(battle, source, targets, *gains);
        }
        StandardEffect::OpponentPaysCost { cost } => {
            pay_cost::execute(battle, source, source.controller().opponent(), cost);
        }
        _ => todo!("Implement {:?}", effect),
    }
}

fn counterspell(
    battle: &mut BattleState,
    source: EffectSource,
    targets: Option<&EffectTargets>,
) -> Option<()> {
    let id = targeting::stack_card_id(targets)?;
    counterspell::execute(battle, source, id);
    Some(())
}

fn draw_cards_for_each(
    battle: &mut BattleState,
    source: EffectSource,
    count: u32,
    for_each: &QuantityExpression,
) {
    let matching = quantity_expression::count(battle, source, for_each);
    deck::draw_cards(battle, source, source.controller(), count * matching);
}

fn dissolve(
    battle: &mut BattleState,
    source: EffectSource,
    targets: Option<&EffectTargets>,
) -> Option<()> {
    let id = targeting::character_id(targets)?;
    dissolve::execute(battle, source, id);
    Some(())
}

fn gains_spark(
    battle: &mut BattleState,
    source: EffectSource,
    targets: Option<&EffectTargets>,
    gains: Spark,
) -> Option<()> {
    let id = targeting::character_id(targets)?;
    spark::gain(battle, source, id, gains);
    Some(())
}
