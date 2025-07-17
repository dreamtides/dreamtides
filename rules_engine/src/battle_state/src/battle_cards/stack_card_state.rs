use std::collections::VecDeque;

use core_data::numerics::Energy;
use core_data::types::PlayerName;

use crate::battle::card_id::{ActivatedAbilityId, CharacterId, StackCardId, VoidCardId};
use crate::battle_cards::battle_card_state::ObjectId;
use crate::battle_cards::card_set::CardSet;

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
            targets.add(StandardEffectTarget::Character(character_id, object_id));
        } else {
            self.targets = Some(EffectTargets::Standard(StandardEffectTarget::Character(
                character_id,
                object_id,
            )));
        }
    }

    pub fn append_stack_card_target(&mut self, stack_card_id: StackCardId, object_id: ObjectId) {
        if let Some(targets) = &mut self.targets {
            targets.add(StandardEffectTarget::StackCard(stack_card_id, object_id));
        } else {
            self.targets = Some(EffectTargets::Standard(StandardEffectTarget::StackCard(
                stack_card_id,
                object_id,
            )));
        }
    }
}

#[derive(Clone, Debug)]
pub enum EffectTargets {
    /// A target for a standard effect.
    Standard(StandardEffectTarget),

    /// Target queue for an effect list. An entry of `None` indicates that the
    /// specified target was provided but is no longer valid on resolution, e.g.
    /// because a target character has been destroyed.
    ///
    /// During effect resolution, we pop targets from the list when required,
    /// i.e. it is assumed that this order will match the order in which targets
    /// are required for effects.
    EffectList(VecDeque<Option<StandardEffectTarget>>),
}

#[derive(Clone, Debug, Copy)]
pub struct VoidCardTarget {
    pub id: VoidCardId,
    pub object_id: ObjectId,
}

#[derive(Clone, Debug)]
pub enum StandardEffectTarget {
    Character(CharacterId, ObjectId),
    StackCard(StackCardId, ObjectId),
    VoidCards(CardSet<VoidCardId>),
}

impl EffectTargets {
    pub fn add(&mut self, target: StandardEffectTarget) {
        match self {
            EffectTargets::Standard(existing) => {
                *self = EffectTargets::EffectList(VecDeque::from([
                    Some(existing.clone()),
                    Some(target),
                ]));
            }
            EffectTargets::EffectList(targets) => {
                targets.push_back(Some(target));
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum StackCardAdditionalCostsPaid {
    None,
    Energy(Energy),
}
