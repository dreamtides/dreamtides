use battle_queries::battle_trace;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::HandCardId;
use battle_state::battle_cards::card_set::CardSet;
use battle_state::core::effect_source::EffectSource;
use battle_state::prompt_types::prompt_data::{
    ChooseHandCardsPrompt, HandCardEffect, PromptData, PromptType,
};
use battle_state::triggers::trigger::Trigger;
use core_data::types::PlayerName;
use strings::strings;

use crate::card_mutations::move_card;
use crate::effects::apply_effect::EffectWasApplied;

pub fn execute(
    battle: &mut BattleState,
    source: EffectSource,
    player: PlayerName,
    count: u32,
) -> Option<EffectWasApplied> {
    if count == 0 {
        return None;
    }

    let hand_cards: Vec<HandCardId> = battle.cards.hand(player).iter().collect();
    let cards_to_discard = count.min(hand_cards.len() as u32);

    if cards_to_discard == 0 {
        battle_trace!("No cards in hand to discard", battle, player);
        return None;
    }

    if cards_to_discard == hand_cards.len() as u32 {
        for hand_card_id in hand_cards {
            let void_card_id = move_card::from_hand_to_void(battle, source, player, hand_card_id);
            battle.triggers.push(source, Trigger::Discarded(void_card_id));
        }
        battle_trace!("Discarded all cards from hand", battle, player, cards_to_discard);
    } else {
        let mut valid = CardSet::new();
        for hand_card_id in hand_cards.iter() {
            valid.insert(*hand_card_id);
        }
        let prompt = PromptData {
            source,
            player,
            prompt_type: PromptType::ChooseHandCards(ChooseHandCardsPrompt {
                effect: HandCardEffect::Discard,
                valid,
                selected: CardSet::default(),
                maximum_selection: cards_to_discard,
            }),
            configuration: Default::default(),
            prompt_description: strings::prompt_choose_cards_to_discard_description().to_string(),
        };

        battle.prompts.push_back(prompt);
        battle_trace!("Added prompt to discard cards from hand", battle, player, cards_to_discard);
    }

    Some(EffectWasApplied)
}
