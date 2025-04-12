use battle_data::battle::battle_data::BattleData;
use battle_data::cards::card_id::{CardIdType, HandCardId};
use battle_data::cards::zone::Zone;
use core_data::source::Source;
use core_data::types::PlayerName;

use crate::zones::move_card;

/// Draw a card from `player`'s deck and put it into their hand. If their deck
/// is empty, it will be replaced with a new shuffled copy of the deck.
///
/// Returns the new [HandCardId] for the card if a card was drawn, or None if no
/// card was drawn (e.g. if the draw was prevented by a game effect).
pub fn draw_card(
    battle: &mut BattleData,
    source: Source,
    player: PlayerName,
) -> Option<HandCardId> {
    let Some(&id) = battle.cards.deck(player).back() else {
        todo!("Todo: implement this");
    };
    let identifier = id.card_identifier(&battle.cards)?;
    let object_id = move_card::run(battle, source, id, Zone::Hand)?;
    battle.cards.card_mut(identifier)?.set_revealed_to(player, true);
    Some(HandCardId::new(object_id, identifier))
}

/// Draw a number of cards from `player`'s deck and put them into their hand.
///
/// Returns the new [HandCardId]s for the cards that were drawn, if any.
pub fn draw_cards(
    battle: &mut BattleData,
    source: Source,
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
