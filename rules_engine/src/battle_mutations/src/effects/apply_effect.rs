use ability_data::cost::Cost;
use ability_data::effect::Effect;
use ability_data::quantity_expression_data::QuantityExpression;
use ability_data::standard_effect::StandardEffect;
use battle_queries::battle_player_queries::quantity_expression;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle_cards::stack_card_state::StackCardTargets;
use battle_state::core::effect_source::EffectSource;

use crate::card_mutations::{deck, negate};
use crate::character_mutations::dissolve;
use crate::effects::{negate_unless_pays_cost, targeting};

pub fn execute(
    battle: &mut BattleState,
    source: EffectSource,
    effect: &Effect,
    targets: &StackCardTargets,
) {
    match effect {
        Effect::Effect(standard) => apply_standard_effect(battle, source, standard, targets),
        _ => todo!("Implement this"),
    }
}

fn apply_standard_effect(
    battle: &mut BattleState,
    source: EffectSource,
    effect: &StandardEffect,
    targets: &StackCardTargets,
) {
    match effect {
        StandardEffect::DrawCardsForEach { count, for_each } => {
            draw_cards_for_each(battle, source, *count, for_each)
        }
        StandardEffect::DissolveCharacter { .. } => dissolve(battle, source, targets),
        StandardEffect::Negate { .. } => negate(battle, source, targets),
        StandardEffect::NegateUnlessPaysCost { cost, .. } => {
            negate_unless_pays_cost::execute(battle, source, targets, cost)
        }
        StandardEffect::OpponentPaysCost { cost } => opponent_pays_cost(battle, source, cost),
        StandardEffect::PayCost { cost } => pay_cost(battle, source, cost),
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

fn dissolve(battle: &mut BattleState, source: EffectSource, targets: &StackCardTargets) {
    let id = targeting::character_id(battle, targets);
    // TODO: Get controller for dissolve target
    dissolve::execute(battle, source, source.controller().opponent(), id);
}

fn negate(battle: &mut BattleState, source: EffectSource, targets: &StackCardTargets) {
    let id = targeting::stack_card_id(battle, targets);
    // TODO: Get controller for negate target
    negate::execute(battle, source, source.controller().opponent(), id);
}

fn opponent_pays_cost(battle: &mut BattleState, source: EffectSource, cost: &Cost) {}

fn pay_cost(battle: &mut BattleState, source: EffectSource, cost: &Cost) {}
