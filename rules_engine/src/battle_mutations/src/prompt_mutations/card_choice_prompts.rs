use ability_data::effect::ModelEffectChoiceIndex;
use battle_queries::battle_card_queries::card;
use battle_queries::battle_trace;
use battle_queries::card_ability_queries::effect_prompts;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{ActivatedAbilityId, StackCardId};
use battle_state::core::effect_source::EffectSource;
use battle_state::prompt_types::prompt_data::OnSelected;
use core_data::types::PlayerName;

/// Adds a prompt to the `battle` for choices & targets required to play the
/// `card_id` card.
pub fn add(
    battle: &mut BattleState,
    player: PlayerName,
    card_id: StackCardId,
    modal_choice: Option<ModelEffectChoiceIndex>,
) {
    for data in &card::ability_list(battle, card_id).event_abilities {
        let source = EffectSource::Event {
            controller: player,
            stack_card_id: card_id,
            ability_number: data.ability_number,
        };

        let mut prompts = effect_prompts::query(
            battle,
            player,
            source,
            &data.ability.effect,
            None,
            OnSelected::AddStackTargets(card_id.into()),
            modal_choice,
        );

        if !prompts.is_empty() {
            battle_trace!("Adding target prompt", battle);
            battle.prompts.append(&mut prompts);
        }
    }
}

/// Adds a prompt to the `battle` for targets required for an activated ability.
pub fn add_for_activated_ability(
    battle: &mut BattleState,
    player: PlayerName,
    activated_ability_id: ActivatedAbilityId,
    modal_choice: Option<ModelEffectChoiceIndex>,
) {
    let abilities = card::ability_list(battle, activated_ability_id.character_id);
    if let Some(ability_data) = abilities
        .activated_abilities
        .iter()
        .find(|data| data.ability_number == activated_ability_id.ability_number)
    {
        let source = EffectSource::Activated { controller: player, activated_ability_id };
        let mut prompts = effect_prompts::query(
            battle,
            player,
            source,
            &ability_data.ability.effect,
            None,
            OnSelected::AddStackTargets(activated_ability_id.into()),
            modal_choice,
        );

        if !prompts.is_empty() {
            battle_trace!("Adding target prompt for activated ability", battle);
            battle.prompts.append(&mut prompts);
        }
    }
}
