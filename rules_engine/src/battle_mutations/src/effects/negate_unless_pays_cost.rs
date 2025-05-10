use ability_data::cost::Cost;
use ability_data::effect::Effect;
use ability_data::predicate::Predicate;
use ability_data::standard_effect::StandardEffect;
use battle_queries::battle_player_queries::costs;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle_cards::stack_card_state::StackCardTargets;
use battle_state::core::effect_source::EffectSource;
use battle_state::prompt_types::prompt_data::{
    PromptChoice, PromptChoiceLabel, PromptConfiguration, PromptContext, PromptData, PromptType,
};
use core_data::numerics::Energy;

use crate::card_mutations::negate;
use crate::effects::targeting;
use crate::prompt_mutations::prompts;

pub fn execute(
    battle: &mut BattleState,
    source: EffectSource,
    targets: &StackCardTargets,
    cost: &Cost,
) {
    if costs::can_pay(battle, source.controller().opponent(), cost) {
        prompts::set(battle, PromptData {
            source,
            player: source.controller().opponent(),
            prompt_type: PromptType::Choose {
                choices: vec![
                    // TODO: Build cost label
                    PromptChoice {
                        label: PromptChoiceLabel::PayEnergy(Energy(2)),
                        effect: Effect::Effect(StandardEffect::OpponentPaysCost {
                            cost: cost.clone(),
                        }),
                        targets: StackCardTargets::None,
                    },
                    PromptChoice {
                        label: PromptChoiceLabel::Decline,
                        effect: Effect::Effect(StandardEffect::Negate { target: Predicate::It }),
                        targets: StackCardTargets::None,
                    },
                ],
            },
            context: PromptContext::TargetNegativeEffect,
            configuration: PromptConfiguration { ..Default::default() },
        });
    } else {
        // TODO: Get controller for target
        negate::execute(
            battle,
            source,
            source.controller().opponent(),
            targeting::stack_card_id(battle, targets),
        );
    }
}
