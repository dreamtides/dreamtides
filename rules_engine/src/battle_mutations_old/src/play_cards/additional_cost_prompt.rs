use ability_data::ability::Ability;
use ability_data::cost::Cost;
use battle_data_old::battle::old_battle_data::BattleData;
use battle_data_old::battle::effect_source::EffectSource;
use battle_data_old::battle_cards::card_id::StackCardId;
use battle_data_old::prompt_types::prompt_data::{
    PromptConfiguration, PromptContext, PromptData, PromptType,
};
use core_data::numerics::Energy;
use core_data::types::PlayerName;
use logging::battle_trace;

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
                battle_trace!("Adding additional cost prompt", battle, prompt_data);
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
        Cost::SpendOneOrMoreEnergy => {
            let energy = battle.player(player).current_energy;
            (
                PromptType::ChooseEnergyValue {
                    minimum: Energy(1),
                    current: Energy(1),
                    maximum: energy,
                },
                PromptContext::PickAmountOfEnergyToSpend,
            )
        }
        _ => todo!("Implement additional cost prompt for {:?}", cost),
    };

    Some(PromptData {
        player,
        prompt_type: prompt,
        source,
        configuration: PromptConfiguration { optional: false, ..Default::default() },
        context,
    })
}
