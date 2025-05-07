use assert_with::{expect, panic_with};
use battle_data::battle::battle_data::BattleData;
use battle_data::battle_cards::card_data::TargetId;
use battle_data::battle_cards::card_id::{CharacterId, StackCardId};
use battle_data::prompt_types::prompt_data::PromptType;
use logging::battle_trace;

/// Selects a character as the target of a card on the stack.
pub fn character(battle: &mut BattleData, character_id: CharacterId) {
    let object_id = expect!(
        battle.cards.card(character_id).map(|c| c.object_id),
        battle,
        || format!("Character does not exist: {:?}", character_id)
    );
    let prompt_data = expect!(battle.prompt.as_ref(), battle, || format!(
        "No active prompt for selecting {:?}",
        character_id
    ));
    let stack_card = expect!(battle.cards.top_of_stack_mut(), battle, || format!(
        "No active stack for selecting {:?}",
        character_id
    ));

    match prompt_data.prompt_type {
        PromptType::ChooseCharacter { .. } => {
            stack_card.targets.push(TargetId::Character(character_id, object_id));
            battle_trace!(
                "Selected character target",
                battle,
                for_card = stack_card.id,
                target = character_id
            );
            battle.prompt = None;
        }
        _ => {
            panic_with!(battle, "Expected a character prompt");
        }
    }
}

/// Selects a card on the stack as a target of a card on the stack.
pub fn stack_card(battle: &mut BattleData, stack_card_id: StackCardId) {
    let object_id = expect!(
        battle.cards.card(stack_card_id).map(|c| c.object_id),
        battle,
        || format!("Stack card does not exist: {:?}", stack_card_id)
    );
    let prompt_data = expect!(battle.prompt.as_ref(), battle, || format!(
        "No active prompt for selecting {:?}",
        stack_card_id
    ));
    let stack_card = expect!(battle.cards.top_of_stack_mut(), battle, || format!(
        "No active stack for selecting {:?}",
        stack_card_id
    ));
    match prompt_data.prompt_type {
        PromptType::ChooseStackCard { .. } => {
            stack_card.targets.push(TargetId::StackCard(stack_card_id, object_id));
            battle_trace!(
                "Selected stack target",
                battle,
                for_card = stack_card.id,
                target = stack_card_id
            );
            battle.prompt = None;
        }
        _ => {
            panic_with!(battle, "Expected a stack card prompt");
        }
    }
}
