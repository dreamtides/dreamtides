use core_data::types::PlayerName;
use serde::Serialize;

use crate::battle_cards::all_cards::AllCards;
use crate::battle_cards::card_data::CardData;
use crate::battle_cards::card_id::CardIdType;
use crate::debug_snapshots::debug_card_data::DebugCardData;

#[derive(Debug, Clone, Serialize)]
pub struct DebugAllCards {
    pub cards: Vec<DebugCardData>,
    pub p1_battlefield: Vec<DebugCardData>,
    pub p2_battlefield: Vec<DebugCardData>,
    pub p1_void: Vec<DebugCardData>,
    pub p2_void: Vec<DebugCardData>,
    pub p1_hand: Vec<DebugCardData>,
    pub p2_hand: Vec<DebugCardData>,
    pub p1_deck: Vec<DebugCardData>,
    pub p2_deck: Vec<DebugCardData>,
    pub stack: Vec<DebugCardData>,
    pub p1_banished: Vec<DebugCardData>,
    pub p2_banished: Vec<DebugCardData>,
    pub next_object_id: String,
}

impl DebugAllCards {
    pub fn new(all_cards: AllCards) -> Self {
        Self {
            cards: Self::cards_to_debug(all_cards.all_cards()),
            p1_battlefield: Self::cards_to_debug(all_cards.battlefield_cards(PlayerName::One)),
            p2_battlefield: Self::cards_to_debug(all_cards.battlefield_cards(PlayerName::Two)),
            p1_void: Self::card_ids_to_debug(all_cards.void(PlayerName::One).iter(), &all_cards),
            p2_void: Self::card_ids_to_debug(all_cards.void(PlayerName::Two).iter(), &all_cards),
            p1_hand: Self::cards_to_debug(all_cards.hand_cards(PlayerName::One)),
            p2_hand: Self::cards_to_debug(all_cards.hand_cards(PlayerName::Two)),
            p1_deck: Self::card_ids_to_debug(all_cards.deck(PlayerName::One).iter(), &all_cards),
            p2_deck: Self::card_ids_to_debug(all_cards.deck(PlayerName::Two).iter(), &all_cards),
            stack: Self::card_ids_to_debug(all_cards.stack().iter(), &all_cards),
            p1_banished: Self::card_ids_to_debug(
                all_cards.banished(PlayerName::One).iter(),
                &all_cards,
            ),
            p2_banished: Self::card_ids_to_debug(
                all_cards.banished(PlayerName::Two).iter(),
                &all_cards,
            ),
            next_object_id: format!(
                "ObjectId({})",
                all_cards.all_cards().next().map_or(0, |card| card.object_id.0)
            ),
        }
    }

    fn cards_to_debug<'a>(cards: impl Iterator<Item = &'a CardData>) -> Vec<DebugCardData> {
        cards.map(|card| DebugCardData::new(card.clone())).collect()
    }

    fn card_ids_to_debug<'a, T>(
        ids: impl Iterator<Item = &'a T>,
        all_cards: &AllCards,
    ) -> Vec<DebugCardData>
    where
        T: 'a + Copy + CardIdType,
    {
        ids.filter_map(|id| all_cards.card(*id).map(|card| DebugCardData::new(card.clone())))
            .collect()
    }
}
