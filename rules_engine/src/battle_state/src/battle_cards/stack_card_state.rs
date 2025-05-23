use core_data::numerics::Energy;
use core_data::types::PlayerName;

use crate::battle::card_id::{CharacterId, StackCardId};

#[derive(Clone, Debug)]
pub struct StackCardState {
    pub id: StackCardId,
    pub controller: PlayerName,
    pub targets: Option<StackCardTargets>,
    pub additional_costs_paid: StackCardAdditionalCostsPaid,
}

#[derive(Clone, Debug)]
pub enum StackCardTargets {
    Character(CharacterId),
    StackCard(StackCardId),
}

#[derive(Clone, Debug)]
pub enum StackCardAdditionalCostsPaid {
    None,
    Energy(Energy),
}
