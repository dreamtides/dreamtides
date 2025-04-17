use core_data::types::PlayerName;

use crate::battle_cards::all_cards::AllCards;
use crate::battle_cards::card_data::CardData;
use crate::battle_cards::card_id::CardIdType;
use crate::debug_snapshots::debug_card_data::DebugCardData;

pub struct DebugAllCards {
    pub cards: Vec<DebugCardData>,
    pub user_battlefield: Vec<DebugCardData>,
    pub enemy_battlefield: Vec<DebugCardData>,
    pub user_void: Vec<DebugCardData>,
    pub enemy_void: Vec<DebugCardData>,
    pub user_hand: Vec<DebugCardData>,
    pub enemy_hand: Vec<DebugCardData>,
    pub user_deck: Vec<DebugCardData>,
    pub enemy_deck: Vec<DebugCardData>,
    pub stack: Vec<DebugCardData>,
    pub user_banished: Vec<DebugCardData>,
    pub enemy_banished: Vec<DebugCardData>,
    pub next_object_id: String,
}

impl DebugAllCards {
    pub fn new(all_cards: AllCards) -> Self {
        Self {
            cards: Self::cards_to_debug(all_cards.all_cards()),
            user_battlefield: Self::cards_to_debug(all_cards.battlefield_cards(PlayerName::User)),
            enemy_battlefield: Self::cards_to_debug(all_cards.battlefield_cards(PlayerName::Enemy)),
            user_void: Self::card_ids_to_debug(all_cards.void(PlayerName::User).iter(), &all_cards),
            enemy_void: Self::card_ids_to_debug(
                all_cards.void(PlayerName::Enemy).iter(),
                &all_cards,
            ),
            user_hand: Self::cards_to_debug(all_cards.hand_cards(PlayerName::User)),
            enemy_hand: Self::cards_to_debug(all_cards.hand_cards(PlayerName::Enemy)),
            user_deck: Self::card_ids_to_debug(all_cards.deck(PlayerName::User).iter(), &all_cards),
            enemy_deck: Self::card_ids_to_debug(
                all_cards.deck(PlayerName::Enemy).iter(),
                &all_cards,
            ),
            stack: Self::card_ids_to_debug(all_cards.stack().iter(), &all_cards),
            user_banished: Self::card_ids_to_debug(
                all_cards.banished(PlayerName::User).iter(),
                &all_cards,
            ),
            enemy_banished: Self::card_ids_to_debug(
                all_cards.banished(PlayerName::Enemy).iter(),
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
