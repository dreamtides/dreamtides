use ability_data::effect::Effect;
use ability_data::quantity_expression_data::QuantityExpression;
use ability_data::standard_effect::StandardEffect;
use battle_queries::battle_card_queries::stack_card_queries;
use battle_queries::battle_player_queries::quantity_expression;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle_cards::stack_card_state::StackCardTargets;
use battle_state::core::effect_source::EffectSource;
use battle_queries::battle_trace;

use crate::card_mutations::{deck, negate};
use crate::character_mutations::dissolve;
use crate::effects::{negate_unless_pays_cost, pay_cost, targeting};

pub fn execute(
    battle: &mut BattleState,
    source: EffectSource,
    effect: &Effect,
    requested_targets: Option<&StackCardTargets>,
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
    targets: Option<&StackCardTargets>,
) {
    battle_trace!("Applying effect", battle, effect, targets);
    match effect {
        StandardEffect::DrawCardsForEach { count, for_each } => {
            draw_cards_for_each(battle, source, *count, for_each);
        }
        StandardEffect::DissolveCharacter { .. } => {
            dissolve(battle, source, targets);
        }
        StandardEffect::Negate { .. } => {
            negate(battle, source, targets);
        }
        StandardEffect::NegateUnlessPaysCost { cost, .. } => {
            negate_unless_pays_cost::execute(battle, source, targets, cost);
        }
        StandardEffect::OpponentPaysCost { cost } => {
            pay_cost::execute(battle, source, source.controller().opponent(), cost);
        }
        StandardEffect::PayCost { cost } => {
            pay_cost::execute(battle, source, source.controller(), cost);
        }
        _ => todo!("Implement {:?}", effect),
    }
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
    targets: Option<&StackCardTargets>,
) -> Option<()> {
    let id = targeting::character_id(targets)?;
    dissolve::execute(battle, source, id);
    Some(())
}

fn negate(
    battle: &mut BattleState,
    source: EffectSource,
    targets: Option<&StackCardTargets>,
) -> Option<()> {
    let id = targeting::stack_card_id(targets)?;
    negate::execute(battle, source, id);
    Some(())
}
