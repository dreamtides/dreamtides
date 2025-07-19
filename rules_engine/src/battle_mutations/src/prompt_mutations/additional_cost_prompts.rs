use ability_data::cost::Cost;
use battle_queries::battle_card_queries::card_abilities;
use battle_queries::battle_trace;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::StackCardId;
use battle_state::core::effect_source::EffectSource;
use battle_state::prompt_types::prompt_data::{PromptConfiguration, PromptData, PromptType};
use core_data::numerics::Energy;
use core_data::types::PlayerName;

/// Adds a prompt for the controller of the `card_id` card to pay additional
/// costs for this card, if any.
pub fn add(battle: &mut BattleState, controller: PlayerName, card_id: StackCardId) {
    for data in &card_abilities::query(battle, card_id).event_abilities {
        if let Some(additional_cost) = &data.ability.additional_cost {
            let source = EffectSource::Event {
                controller,
                stack_card_id: card_id,
                ability_number: data.ability_number,
            };
            let prompt_data = create_prompt_for_cost(battle, controller, source, additional_cost);
            battle_trace!("Adding additional cost prompt", battle);
            battle.prompts.push_back(prompt_data);
            return;
        }
    }
}

fn create_prompt_for_cost(
    battle: &BattleState,
    player: PlayerName,
    source: EffectSource,
    cost: &Cost,
) -> PromptData {
    let prompt = match cost {
        Cost::SpendOneOrMoreEnergy => {
            let energy = battle.players.player(player).current_energy;
            PromptType::ChooseEnergyValue { minimum: Energy(1), maximum: energy }
        }
        _ => todo!("Implement additional cost prompt for {:?}", cost),
    };

    PromptData {
        player,
        prompt_type: prompt,
        source,
        configuration: PromptConfiguration { optional: false },
    }
}
