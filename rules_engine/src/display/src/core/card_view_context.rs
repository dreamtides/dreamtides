use battle_data_old::battle::old_battle_data::BattleData;
use battle_data_old::battle_cards::card_data::CardData;

/// Provides the context in which a card view is being displayed, i.e. either
/// during an active battle or in a deck or draft context.
pub enum CardViewContext<'a> {
    Battle(&'a BattleData, &'a CardData),
}

impl<'a> CardViewContext<'a> {
    pub fn battle(&self) -> &'a BattleData {
        match self {
            CardViewContext::Battle(battle, _) => battle,
        }
    }

    pub fn card(&self) -> &'a CardData {
        match self {
            CardViewContext::Battle(_, card) => card,
        }
    }
}
