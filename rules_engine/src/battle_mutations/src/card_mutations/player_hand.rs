use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardId;
use battle_state::battle_cards::zone::Zone;
use battle_state::core::effect_source::EffectSource;
use core_data::types::PlayerName;

use crate::card_mutations::deck;

/// Randomizes the hand of the provided `player`.
///
/// Returns all hards in this player's hand to their deck, then draws that many
/// cards.
pub fn randomize_player_hand(battle: &mut BattleState, player: PlayerName) {
    let hand = battle.cards.hand(player).clone();
    let count = hand.len();

    for card_id in &hand {
        battle.cards.move_card(player, CardId(card_id), Zone::Hand, Zone::Deck);
    }

    deck::draw_cards(battle, EffectSource::Game { controller: player }, player, count as u32);
}
