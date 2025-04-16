use battle_data::battle::battle_data::BattleData;
use battle_data::battle_cards::card_data::TargetId;
use battle_data::battle_cards::card_instance_id::CardInstanceId;
use battle_data::prompts::prompt_data::Prompt;
use core_data::effect_source::EffectSource;
use core_data::identifiers::CardId;
use tracing::info;

/// Applies whatever game effect is required for a card being selected in the
/// UI, e.g. setting it as a chosen target of a card on the stack.
pub fn select_for_prompt(battle: &mut BattleData, _source: EffectSource, card_id: CardId) {
    assert!(battle.prompt.is_some(), "No active prompt when selecting a card");
    let prompt = battle.prompt.as_ref().expect("Prompt should exist");

    match &prompt.prompt {
        Prompt::ChooseCharacter { valid } => {
            let card = battle.cards.card(card_id);
            assert!(card.is_some(), "Selected card does not exist");
            let character_id = match card.expect("Card should exist").id {
                CardInstanceId::Battlefield(char_id) if valid.contains(&char_id) => char_id,
                _ => panic!("Selected card is not a valid target for the current prompt"),
            };

            let top_stack_card_id = battle
                .cards
                .stack()
                .last()
                .expect("Stack should be non-empty when selecting a card");
            let top_stack_card =
                battle.cards.card_mut(*top_stack_card_id).expect("Top stack card should exist");

            top_stack_card.targets.push(TargetId::Character(character_id));
            info!("Targets for card updated to {:?}", top_stack_card.targets);
            battle.prompt = None;
        }
    }
}
