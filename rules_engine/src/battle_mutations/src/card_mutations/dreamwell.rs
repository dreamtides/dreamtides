use std::sync::Arc;

use battle_queries::panic_with;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle_cards::dreamwell_data::DreamwellCard;
use rand::seq::SliceRandom;

/// Draws the next card from the dreamwell.
///
/// Panics if the dreamwell is empty.
pub fn draw(battle: &mut BattleState) -> Arc<DreamwellCard> {
    if battle.dreamwell.next_index == 0 {
        randomize(battle);
    }

    let card = if let Some(card) = battle.dreamwell.cards.get(battle.dreamwell.next_index) {
        card.clone()
    } else {
        panic_invalid_index(battle, battle.dreamwell.next_index);
    };

    battle.dreamwell.next_index += 1;
    if battle.dreamwell.next_index == battle.dreamwell.cards.len() {
        // Special case: when we reach the end of the dreamwell, remove all
        // 'phase 0' cards, since these are the "starter" cards.
        let mut new_cards = battle.dreamwell.cards.as_ref().clone();
        new_cards.retain(|c| c.definition.phase != 0);

        if !new_cards.is_empty() {
            battle.dreamwell.cards = Arc::new(new_cards);
        }
        battle.dreamwell.next_index = 0;
    }

    card
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
