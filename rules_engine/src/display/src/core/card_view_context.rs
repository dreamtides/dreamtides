use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardId;
use core_data::identifiers::CardName;

/// Provides the context in which a card view is being displayed, i.e. either
/// during an active battle or in a deck or draft context.
pub enum CardViewContext<'a> {
    Battle(&'a BattleState, CardName, CardId),
}

impl<'a> CardViewContext<'a> {
    pub fn battle(&self) -> &'a BattleState {
        match self {
            CardViewContext::Battle(battle, _, _) => battle,
        }
    }

    pub fn card_name(&self) -> CardName {
        match self {
            CardViewContext::Battle(_, card_name, _) => *card_name,
        }
    }

    pub fn card_id(&self) -> CardId {
        match self {
            CardViewContext::Battle(_, _, card_id) => *card_id,
        }
    }
}
