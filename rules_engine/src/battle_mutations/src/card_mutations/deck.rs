use battle_queries::battle_card_queries::card_abilities;
use battle_state::battle::all_cards::CreatedCard;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{DeckCardId, HandCardId};
use battle_state::battle_cards::card_set::CardSet;
use battle_state::core::effect_source::EffectSource;
use core_data::identifiers::CardName;
use core_data::numerics::Energy;
use core_data::types::PlayerName;
use rand::seq::IteratorRandom;
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::card_mutations::{create_test_deck, move_card};
use crate::player_mutations::energy;

const HAND_SIZE_LIMIT: usize = 10;

/// Draw a card from `player`'s deck and put it into their hand. If their deck
/// is empty, it will be replaced with a new shuffled copy of the deck.
///
/// Returns the new [HandCardId] for the card if a card was drawn, or None if no
/// card was drawn (e.g. if the player's hand size limit was exceeded or the
/// draw was prevented by a game effect).
pub fn draw_card(
    battle: &mut BattleState,
    source: EffectSource,
    player: PlayerName,
) -> Option<HandCardId> {
    if battle.cards.hand(player).len() >= HAND_SIZE_LIMIT {
        // If a player exceeds the hand size limit, they instead gain 1
        // energy for each card they would have drawn.
        energy::gain(battle, player, source, Energy(1));
        let p = battle.turn_history.current_action_history.player_mut(player);
        p.hand_size_limit_exceeded = true;
        return None;
    }

    let Some(id) = random_element(battle.cards.deck(player), &mut battle.rng) else {
        create_test_deck::add(battle, player);
        return draw_card(battle, source, player);
    };
    Some(move_card::from_deck_to_hand(battle, source, player, id))
}

/// Draw a number of cards from `player`'s deck and put them into their hand.
pub fn draw_cards(battle: &mut BattleState, source: EffectSource, player: PlayerName, count: u32) {
    for _ in 0..count {
        draw_card(battle, source, player);
    }
}

/// Adds a list of cards by name to a player's deck
pub fn add_cards(battle: &mut BattleState, player: PlayerName, cards: Vec<CardName>) {
    battle.cards.create_cards_in_deck(
        player,
        cards
            .into_iter()
            .map(|name| CreatedCard {
                name,
                can_play_restriction: card_abilities::query_by_name(name).can_play_restriction,
            })
            .collect(),
    );
}

/// Returns a random element from the given set.
fn random_element(set: &CardSet<DeckCardId>, rng: &mut Xoshiro256PlusPlus) -> Option<DeckCardId> {
    set.iter().choose(rng)
}
