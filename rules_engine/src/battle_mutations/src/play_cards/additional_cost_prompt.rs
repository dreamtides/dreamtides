use std::cmp;

use ability_data::ability::Ability;
use ability_data::cost::Cost;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle::effect_source::EffectSource;
use battle_data::battle_cards::card_id::StackCardId;
use battle_data::prompt_types::prompt_data::{
    Prompt, PromptConfiguration, PromptContext, PromptData,
};
use core_data::numerics::Energy;
use core_data::types::PlayerName;

/// Adds a prompt for the controller of the `card_id` card to pay additional
/// costs for this card, if any.
pub fn add_additional_cost_prompt(
    battle: &mut BattleData,
    source: EffectSource,
    card_id: StackCardId,
) {
    let Some(card) = battle.cards.card(card_id) else {
        return;
    };
    let player = card.controller();

    for ability in &card.abilities {
        if let Ability::Event(event) = ability {
            if let Some(prompt_data) =
                create_prompt_for_cost(battle, player, source, event.additional_cost.as_ref())
            {
                battle.prompt = Some(prompt_data);
                return;
            }
        }
    }
}

fn create_prompt_for_cost(
    battle: &BattleData,
    player: PlayerName,
    source: EffectSource,
    cost: Option<&Cost>,
) -> Option<PromptData> {
    let (prompt, context) = match cost? {
        Cost::SpendAnyAmountOfEnergy => {
            let energy = battle.player(player).current_energy;
            (
                Prompt::ChooseEnergyValue {
                    minimum: Energy(0),
                    current: cmp::min(Energy(1), energy),
                    maximum: energy,
                },
                PromptContext::PickAmountOfEnergyToSpend,
            )
        }
        _ => todo!("Implement additional cost prompt for {:?}", cost),
    };

    Some(PromptData {
        player,
        prompt,
        source,
        configuration: PromptConfiguration { optional: false, ..Default::default() },
        context,
    })
}
