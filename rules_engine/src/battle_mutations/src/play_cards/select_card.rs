use battle_data::battle::battle_data::BattleData;
use battle_data::battle_cards::card_data::TargetId;
use battle_data::battle_cards::card_id::CharacterId;
use battle_data::prompts::prompt_data::Prompt;
use core_data::effect_source::EffectSource;
use tracing::info;

/// Applies whatever game effect is required for a card being selected in the
/// UI, e.g. setting it as a chosen target of a card on the stack.
pub fn select_for_prompt(
    battle: &mut BattleData,
    _source: EffectSource,
    character_id: CharacterId,
) {
    match &battle.expect_prompt().prompt {
        Prompt::ChooseCharacter { .. } => {
            let stack_card = battle.cards.top_of_stack_mut().expect("No active stack");
            stack_card.targets.push(TargetId::Character(character_id));
            info!("Targets for {:?} updated to {:?}", stack_card.id, stack_card.targets);
            battle.prompt = None;
        }
    }
}
