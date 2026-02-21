use ability_data::cost::Cost;
use ability_data::effect::Effect;
use ability_data::predicate::Predicate;
use ability_data::standard_effect::StandardEffect;
use battle_queries::battle_player_queries::costs;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle_cards::stack_card_state::EffectTargets;
use battle_state::core::effect_source::EffectSource;
use battle_state::prompt_types::prompt_data::{
    PromptChoice, PromptChoiceLabel, PromptConfiguration, PromptData, PromptType,
};
use strings::strings;

use crate::card_mutations::counterspell;
use crate::effects::apply_effect::EffectWasApplied;
use crate::effects::targeting;

pub fn execute(
    battle: &mut BattleState,
    source: EffectSource,
    targets: &mut Option<EffectTargets>,
    cost: &Cost,
) -> Option<EffectWasApplied> {
    if costs::can_pay(battle, source.controller().opponent(), cost) {
        let prompt_targets = targets.clone();
        battle.prompts.push_back(PromptData {
            source,
            player: source.controller().opponent(),
            prompt_type: PromptType::Choose {
                choices: vec![
                    PromptChoice {
                        label: costs::pay_cost_label(cost),
                        effect: Effect::Effect(StandardEffect::OpponentPaysCost {
                            cost: cost.clone(),
                        }),
                        targets: prompt_targets.clone(),
                    },
                    PromptChoice {
                        label: PromptChoiceLabel::String(
                            strings::phrase_ids::DECLINE_PROMPT_BUTTON,
                        ),
                        effect: Effect::Effect(StandardEffect::Counterspell {
                            target: Predicate::It,
                        }),
                        targets: prompt_targets,
                    },
                ],
            },
            configuration: PromptConfiguration { ..Default::default() },
            prompt_description: strings::prompt_pay_or_decline_description().to_string(),
        });
    } else {
        counterspell::execute(battle, source, targeting::stack_card_id(targets)?);
    }
    Some(EffectWasApplied)
}
