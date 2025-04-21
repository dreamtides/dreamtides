use assert_with::{expect, panic_with};
use battle_data::battle::battle_data::BattleData;
use battle_data::battle_cards::card_data::TargetId;
use battle_data::battle_cards::card_id::{CharacterId, StackCardId};
use battle_data::prompt_types::prompt_data::Prompt;
use tracing::info;

/// Applies whatever game effect is required for a card being selected in the
/// UI, e.g. setting it as a chosen target of a card on the stack.
pub fn select_character_for_prompt(battle: &mut BattleData, character_id: CharacterId) {
    let prompt_data = expect!(battle.prompt.as_ref(), battle, || format!(
        "No active prompt for selecting {:?}",
        character_id
    ));
    let stack_card = expect!(battle.cards.top_of_stack_mut(), battle, || format!(
        "No active stack for selecting {:?}",
        character_id
    ));
    match prompt_data.prompt {
        Prompt::ChooseCharacter { .. } => {
            stack_card.targets.push(TargetId::Character(character_id));
            info!("Targets for {:?} updated to {:?}", stack_card.id, stack_card.targets);
            battle.prompt = None;
        }
        _ => {
            panic_with!(battle, "Expected a character prompt");
        }
    }
}

/// Applies whatever game effect is required for a card being selected in the
/// UI, e.g. setting it as a chosen target of a card on the stack.
pub fn select_stack_card_for_prompt(battle: &mut BattleData, stack_card_id: StackCardId) {
    let prompt_data = expect!(battle.prompt.as_ref(), battle, || format!(
        "No active prompt for selecting {:?}",
        stack_card_id
    ));
    let stack_card = expect!(battle.cards.top_of_stack_mut(), battle, || format!(
        "No active stack for selecting {:?}",
        stack_card_id
    ));
    match prompt_data.prompt {
        Prompt::ChooseStackCard { .. } => {
            stack_card.targets.push(TargetId::StackCard(stack_card_id));
            info!("Targets for {:?} updated to {:?}", stack_card.id, stack_card.targets);
            battle.prompt = None;
        }
        _ => {
            panic_with!(battle, "Expected a stack card prompt");
        }
    }
}
