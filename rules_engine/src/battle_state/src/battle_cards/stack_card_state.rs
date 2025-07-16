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

impl StackItemState {
    pub fn append_character_target(&mut self, character_id: CharacterId, object_id: ObjectId) {
        if let Some(targets) = &mut self.targets {
            targets.add(SingleEffectTarget::Character(character_id, object_id));
        } else {
            self.targets =
                Some(EffectTargets::Single(SingleEffectTarget::Character(character_id, object_id)));
        }
    }

    pub fn append_stack_card_target(&mut self, stack_card_id: StackCardId, object_id: ObjectId) {
        if let Some(targets) = &mut self.targets {
            targets.add(SingleEffectTarget::StackCard(stack_card_id, object_id));
        } else {
            self.targets = Some(EffectTargets::Single(SingleEffectTarget::StackCard(
                stack_card_id,
                object_id,
            )));
        }
    }
}

#[derive(Clone, Debug)]
pub enum EffectTargets {
    Single(SingleEffectTarget),
    List(Vec<SingleEffectTarget>),
}

#[derive(Clone, Debug)]
pub enum SingleEffectTarget {
    Character(CharacterId, ObjectId),
    StackCard(StackCardId, ObjectId),
}

impl EffectTargets {
    pub fn add(&mut self, target: SingleEffectTarget) {
        match self {
            EffectTargets::Single(existing) => {
                *self = EffectTargets::List(vec![existing.clone(), target]);
            }
            EffectTargets::List(targets) => {
                targets.push(target);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum StackCardAdditionalCostsPaid {
    None,
    Energy(Energy),
}
