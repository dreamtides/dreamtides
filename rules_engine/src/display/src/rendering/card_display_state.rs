use battle_queries::battle_card_queries::card;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardId;
use battle_state::battle_cards::zone::Zone;
use core_data::types::PlayerName;

use crate::rendering::positions::{self, ControllerAndZone};

/// Returns true if the card is revealed to the player.
pub fn is_revealed_to(battle: &BattleState, card_id: CardId, player: PlayerName) -> bool {
    if *card::get(battle, card_id).revealed_to_player_override.player(player) {
        return true;
    }

    let ControllerAndZone { controller, zone } = positions::controller_and_zone(battle, card_id);

    match zone {
        Zone::Banished | Zone::Void | Zone::Stack | Zone::Battlefield => true,
        Zone::Hand => controller == player,
        Zone::Deck => false,
    }
}
