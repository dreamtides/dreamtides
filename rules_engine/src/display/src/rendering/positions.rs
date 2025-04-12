use battle_data::cards::card_data::CardData;
use display_data::object_position::{ObjectPosition, Position, StackType};

pub fn calculate(card: &CardData) -> ObjectPosition {
    let position = match card.zone() {
        battle_data::cards::zone::Zone::Hand => Position::InHand(card.owner),
        battle_data::cards::zone::Zone::Deck => Position::InDeck(card.owner),
        battle_data::cards::zone::Zone::Battlefield => Position::OnBattlefield(card.owner),
        battle_data::cards::zone::Zone::Stack => Position::OnStack(StackType::Default),
        battle_data::cards::zone::Zone::Void => Position::InVoid(card.owner),
        battle_data::cards::zone::Zone::Banished => Position::InBanished(card.owner),
    };

    for_card(card, position)
}

pub fn for_card(card: &CardData, position: Position) -> ObjectPosition {
    let sorting_key = card.id.object_id().0;
    ObjectPosition { position, sorting_key, sorting_sub_key: 0 }
}
