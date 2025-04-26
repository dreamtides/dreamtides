use battle_data::battle::battle_data::BattleData;
use battle_data::battle::effect_source::EffectSource;
use battle_data::battle_cards::card_id::HandCardId;
use core_data::types::PlayerName;

use crate::zone_mutations::move_card;

/// Draw a card from `player`'s deck and put it into their hand. If their deck
/// is empty, it will be replaced with a new shuffled copy of the deck.
///
/// Returns the new [HandCardId] for the card if a card was drawn, or None if no
/// card was drawn (e.g. if the draw was prevented by a game effect).
pub fn draw_card(
    battle: &mut BattleData,
    source: EffectSource,
    player: PlayerName,
) -> Option<HandCardId> {
    let Some(&id) = battle.cards.deck(player).back() else {
        todo!("Todo: implement this");
    };
    let id = move_card::to_hand(battle, source, id)?;
    battle.cards.card_mut(id)?.revealed_to_owner = true;
    Some(id)
}

/// Draw a number of cards from `player`'s deck and put them into their hand.
///
/// Returns the new [HandCardId]s for the cards that were drawn, if any.
pub fn draw_cards(
    battle: &mut BattleData,
    source: EffectSource,
    player: PlayerName,
    count: u32,
) -> Vec<HandCardId> {
    let mut result = Vec::new();
    for _ in 0..count {
        if let Some(id) = draw_card(battle, source, player) {
            result.push(id);
        }
    }
    result
}

/// Shuffles the deck for the [PlayerName] player.
pub fn shuffle(battle: &mut BattleData, player: PlayerName) {
    battle.cards.shuffle_deck(player, &mut battle.rng);
}
