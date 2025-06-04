use core_data::numerics::Energy;
use core_data::types::PlayerName;

use crate::battle::card_id::{CharacterId, StackCardId};
use crate::battle_cards::battle_card_state::ObjectId;

#[derive(Clone, Debug)]
pub struct StackCardState {
    pub id: StackCardId,
    pub controller: PlayerName,
    pub targets: Option<StackCardTargets>,
    pub additional_costs_paid: StackCardAdditionalCostsPaid,
}

#[derive(Clone, Debug)]
pub enum StackCardTargets {
    Character(CharacterId, ObjectId),
    StackCard(StackCardId, ObjectId),
}

#[derive(Clone, Debug)]
pub enum StackCardAdditionalCostsPaid {
    None,
    Energy(Energy),
}
