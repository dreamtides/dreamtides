use ability_data::cost::Cost;
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
use core_data::types::PlayerName;

use crate::card_mutations::{counterspell, deck, spark};
use crate::character_mutations::dissolve;
use crate::effects::{counterspell_unless_pays_cost, pay_cost, targeting};

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
        Effect::Effect(standard) => apply_standard_effect(battle, source, standard, targets),
        _ => todo!("Implement this"),
    };

    if !battle.cards.has_stack() {
        // If this effect removed the last card from the stack, stack priority
        // is ended.
        battle.stack_priority = None;
    }

    result
}

fn apply_standard_effect(
    battle: &mut BattleState,
    source: EffectSource,
    effect: &StandardEffect,
    targets: Option<&EffectTargets>,
) -> Option<EffectWasApplied> {
    battle_trace!("Applying effect", battle, effect, targets);
    match effect {
        StandardEffect::Counterspell { .. } => counterspell(battle, source, targets),
        StandardEffect::CounterspellUnlessPaysCost { cost, .. } => {
            counterspell_unless_pays_cost::execute(battle, source, targets, cost)
        }
        StandardEffect::DrawCards { count } => {
            draw_cards(battle, source, source.controller(), *count)
        }
        StandardEffect::DrawCardsForEach { count, for_each } => {
            draw_cards_for_each(battle, source, *count, for_each)
        }
        StandardEffect::DissolveCharacter { .. } => dissolve(battle, source, targets),
        StandardEffect::GainsSpark { gains, .. } => gains_spark(battle, source, targets, *gains),
        StandardEffect::OpponentPaysCost { cost } => opponent_pays_cost(battle, source, cost),
        _ => todo!("Implement {:?}", effect),
    }
}

fn counterspell(
    battle: &mut BattleState,
    source: EffectSource,
    targets: Option<&EffectTargets>,
) -> Option<EffectWasApplied> {
    let id = targeting::stack_card_id(targets)?;
    counterspell::execute(battle, source, id);
    Some(EffectWasApplied)
}

fn draw_cards(
    battle: &mut BattleState,
    source: EffectSource,
    player: PlayerName,
    count: u32,
) -> Option<EffectWasApplied> {
    if count > 0 {
        deck::draw_cards(battle, source, player, count);
        Some(EffectWasApplied)
    } else {
        None
    }
}

fn draw_cards_for_each(
    battle: &mut BattleState,
    source: EffectSource,
    count: u32,
    for_each: &QuantityExpression,
) -> Option<EffectWasApplied> {
    let matching = quantity_expression::count(battle, source, for_each);
    let quantity = count * matching;
    if quantity > 0 {
        deck::draw_cards(battle, source, source.controller(), quantity);
        Some(EffectWasApplied)
    } else {
        None
    }
}

fn dissolve(
    battle: &mut BattleState,
    source: EffectSource,
    targets: Option<&EffectTargets>,
) -> Option<EffectWasApplied> {
    let id = targeting::character_id(targets)?;
    dissolve::execute(battle, source, id);
    Some(EffectWasApplied)
}

fn gains_spark(
    battle: &mut BattleState,
    source: EffectSource,
    targets: Option<&EffectTargets>,
    gains: Spark,
) -> Option<EffectWasApplied> {
    let id = targeting::character_id(targets)?;
    spark::gain(battle, source, id, gains);
    Some(EffectWasApplied)
}

fn opponent_pays_cost(
    battle: &mut BattleState,
    source: EffectSource,
    cost: &Cost,
) -> Option<EffectWasApplied> {
    pay_cost::execute(battle, source, source.controller().opponent(), cost);
    Some(EffectWasApplied)
}
