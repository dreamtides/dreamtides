use battle_data::battle_cards::card_data::CardData;
use battle_data::battle_cards::zone::Zone;
use display_data::object_position::{ObjectPosition, Position, StackType};

pub fn calculate(card: &CardData) -> ObjectPosition {
    let position = match card.zone {
        Zone::Hand => Position::InHand(card.owner),
        Zone::Deck => Position::InDeck(card.owner),
        Zone::Battlefield => Position::OnBattlefield(card.owner),
        Zone::Stack => Position::OnStack(StackType::Default),
        Zone::Void => Position::InVoid(card.owner),
        Zone::Banished => Position::InBanished(card.owner),
    };

    for_card(card, position)
}

pub fn for_card(card: &CardData, position: Position) -> ObjectPosition {
    let sorting_key = card.object_id.0;
    ObjectPosition { position, sorting_key, sorting_sub_key: 0 }
}
