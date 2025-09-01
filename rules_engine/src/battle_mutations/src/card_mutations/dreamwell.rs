use std::sync::Arc;

use battle_queries::panic_with;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle_cards::dreamwell_data::{BattleDreamwellCardId, DreamwellCard};
use rand::seq::SliceRandom;

/// Draws the next card from the dreamwell.
///
/// Panics if the dreamwell is empty.
pub fn draw(battle: &mut BattleState) -> (Arc<DreamwellCard>, BattleDreamwellCardId) {
    if battle.dreamwell.next_index == 0 {
        randomize(battle);
    }

    let mut index = battle.dreamwell.next_index;

    // Skip phase 0 cards if this is not the first iteration through the
    // dreamwell.
    if battle.dreamwell.first_iteration_complete {
        while index < battle.dreamwell.cards.len() {
            if let (Some(card), _) = battle.dreamwell.get(index) {
                if card.definition.phase != 0 {
                    break;
                }
            }
            index += 1;
        }
    }

    let (card, card_id) = if let (Some(card), card_id) = battle.dreamwell.get(index) {
        (card.clone(), card_id)
    } else {
        panic_invalid_index(battle, index);
    };

    battle.dreamwell.next_index = index + 1;
    if battle.dreamwell.next_index >= battle.dreamwell.cards.len() {
        battle.dreamwell.first_iteration_complete = true;
        battle.dreamwell.next_index = 0;
    }

    (card, card_id)
}

pub fn randomize(battle: &mut BattleState) {
    let mut new_cards = battle.dreamwell.cards.as_ref().clone();
    new_cards.shuffle(&mut battle.rng);
    new_cards.sort_by_key(|c| c.definition.phase);
    battle.dreamwell.cards = Arc::new(new_cards);
}

#[cold]
fn panic_invalid_index(battle: &BattleState, index: usize) -> ! {
    panic_with!("Invalid dreamwell index", battle, index);
}
