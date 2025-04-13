use battle_data::battle::battle_data::BattleData;
use battle_data::battle_cards::card_id::ObjectId;
use battle_data::battle_cards::card_types::CardType;
use battle_data::battle_cards::zone::Zone;
use core_data::identifiers::CardId;
use core_data::source::Source;
use core_data::types::PlayerName;

use crate::player_mutations::energy;
use crate::zone_mutations::move_card;

/// Attempts to play a card as `player`.
///
/// Returns the [ObjectId] of the card it its new zone if the card was played
/// successfully, otherwise returns `None`, e.g. if this card is prevented from
/// being played or no longer exists.
pub fn execute(
    battle: &mut BattleData,
    player: PlayerName,
    source: Source,
    card_id: CardId,
) -> Option<ObjectId> {
    if let Some(energy_cost) = battle.cards.card(card_id)?.properties.cost {
        energy::spend(battle, player, source, energy_cost);
    }
    battle.cards.card_mut(card_id)?.revealed_to_opponent = true;
    move_card::run(
        battle,
        source,
        card_id,
        destination_zone(battle.cards.card(card_id)?.properties.card_type),
    )
}

fn destination_zone(card_type: CardType) -> Zone {
    match card_type {
        CardType::Character(_) => Zone::Battlefield,
        CardType::Event => Zone::Void,
        _ => panic!("Invalid card type: {:?}", card_type),
    }
}
