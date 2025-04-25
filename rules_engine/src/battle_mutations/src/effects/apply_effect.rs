use ability_data::effect::{Effect, EffectWithOptions};
use ability_data::predicate::Predicate;
use ability_data::quantity_expression_data::QuantityExpression;
use ability_data::standard_effect::StandardEffect;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle::effect_source::EffectSource;
use battle_data::battle_cards::card_data::TargetId;
use battle_data::battle_cards::card_id::{CharacterId, StackCardId};
use battle_data::battle_cards::zone::Zone;
use battle_data::prompt_types::prompt_data::{
    Prompt, PromptChoice, PromptConfiguration, PromptContext, PromptData,
};
use battle_queries::cost_queries::costs;
use battle_queries::player_queries;

use crate::character_mutations::dissolve;
use crate::core::prompts;
use crate::effects::{negate, pay_cost};
use crate::zone_mutations::deck;

/// Applies an effect to a set of [TargetId]s.
///
/// Any targets in the provided list which are no longer valid (e.g. because
/// they have changed zones) are removed before applying effects. This may cause
/// the effect to do nothing.
pub fn apply(
    battle: &mut BattleData,
    source: EffectSource,
    effect: Effect,
    mut targets: Vec<TargetId>,
) -> Option<()> {
    remove_invalid_targets(battle, &mut targets);

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

fn remove_invalid_targets(battle: &BattleData, targets: &mut Vec<TargetId>) {
    targets.retain(|target| {
        let current_card = match target {
            TargetId::StackCard(stack_id, _) => battle.cards.card(*stack_id),
            TargetId::Character(character_id, _) => battle.cards.card(*character_id),
        };

        match current_card {
            Some(card) => card.object_id == target.object_id(),
            None => false,
        }
    });
}

fn apply_effect_with_options(
    _battle: &mut BattleData,
    _source: EffectSource,
    _effect: EffectWithOptions,
    _targets: &[TargetId],
) -> Option<()> {
    todo!("Implement effect with options")
}

fn apply_list_effect(
    battle: &mut BattleData,
    source: EffectSource,
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
    source: EffectSource,
    effect: StandardEffect,
    targets: &[TargetId],
) -> Option<()> {
    match effect {
        StandardEffect::DrawCardsForEach { count, for_each } => {
            draw_cards_for_each(battle, source, count, for_each)
        }
        StandardEffect::DissolveCharacter { .. } => {
            for character_id in character_ids(targets) {
                dissolve::apply(battle, source, character_id);
            }
        }
        StandardEffect::Negate { .. } => negate(battle, source, targets),
        StandardEffect::NegateUnlessPaysCost { cost, .. } => {
            negate_unless_pays_cost(battle, source, targets, cost)
        }
        StandardEffect::OpponentPaysCost { cost } => {
            pay_cost::apply(battle, source, source.controller().opponent(), cost)
        }
        StandardEffect::PayCost { cost } => {
            pay_cost::apply(battle, source, source.controller(), cost)
        }
        _ => todo!("Implement {:?}", effect),
    }
    Some(())
}

fn draw_cards_for_each(
    battle: &mut BattleData,
    source: EffectSource,
    count: u32,
    for_each: QuantityExpression,
) {
    let matching = player_queries::quantity_expression::count(battle, source, for_each);
    deck::draw_cards(battle, source, source.controller(), count * matching);
}

fn negate(battle: &mut BattleData, source: EffectSource, targets: &[TargetId]) {
    for stack_card_id in stack_card_ids(targets) {
        negate::apply(battle, source, stack_card_id);
    }
}

fn negate_unless_pays_cost(
    battle: &mut BattleData,
    source: EffectSource,
    targets: &[TargetId],
    cost: ability_data::cost::Cost,
) {
    if costs::can_pay(battle, source.controller().opponent(), &cost) {
        prompts::set(battle, PromptData {
            source,
            player: source.controller().opponent(),
            prompt: Prompt::Choose {
                choices: vec![
                    PromptChoice {
                        label: "Pay $2".to_string(),
                        effect: Effect::Effect(StandardEffect::OpponentPaysCost { cost }),
                        targets: vec![],
                    },
                    PromptChoice {
                        label: "Decline".to_string(),
                        effect: Effect::Effect(StandardEffect::Negate { target: Predicate::It }),
                        targets: targets.to_vec(),
                    },
                ],
            },
            context: PromptContext::TargetNegativeEffect,
            configuration: PromptConfiguration {
                move_source_to: Some(Zone::Void),
                ..Default::default()
            },
        });
    } else {
        negate(battle, source, targets);
    }
}

fn character_ids(targets: &[TargetId]) -> impl Iterator<Item = CharacterId> + '_ {
    targets.iter().filter_map(|target| match target {
        TargetId::Character(character_id, _) => Some(*character_id),
        _ => None,
    })
}

fn stack_card_ids(targets: &[TargetId]) -> impl Iterator<Item = StackCardId> + '_ {
    targets.iter().filter_map(|target| match target {
        TargetId::StackCard(stack_card_id, _) => Some(*stack_card_id),
        _ => None,
    })
}
