use core_data::numerics::Energy;
use core_data::types::PlayerName;

use crate::battle::card_id::{ActivatedAbilityId, CharacterId, StackCardId};
use crate::battle_cards::battle_card_state::ObjectId;

/// A vector of items on the stack
///
/// No significant performance differences between SmallVec and Vec here.
pub type StackItems = Vec<StackItemState>;

#[derive(Clone, Debug, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum StackItemId {
    Card(StackCardId),
    ActivatedAbility(ActivatedAbilityId),
}

impl From<StackCardId> for StackItemId {
    fn from(id: StackCardId) -> Self {
        StackItemId::Card(id)
    }
}

impl From<ActivatedAbilityId> for StackItemId {
    fn from(id: ActivatedAbilityId) -> Self {
        StackItemId::ActivatedAbility(id)
    }
}

#[derive(Clone, Debug)]
pub struct StackItemState {
    pub id: StackItemId,
    pub controller: PlayerName,
    pub targets: Option<EffectTargets>,
    pub additional_costs_paid: StackCardAdditionalCostsPaid,
}

#[derive(Clone, Debug)]
pub enum EffectTargets {
    Character(CharacterId, ObjectId),
    StackCard(StackCardId, ObjectId),
}

#[derive(Clone, Debug)]
pub enum StackCardAdditionalCostsPaid {
    None,
    Energy(Energy),
}
