use std::collections::{BTreeSet, VecDeque};

use ability_data::effect::ModelEffectChoiceIndex;
use core_data::numerics::Energy;
use core_data::types::PlayerName;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::battle::card_id::{
    ActivatedAbilityId, CardId, CardIdType, CharacterId, StackCardId, VoidCardId,
};
use crate::battle_cards::battle_card_state::{CardObjectId, ObjectId};

/// A vector of items on the stack
///
/// No significant performance differences between SmallVec and Vec here.
pub type StackItems = Vec<StackItemState>;

#[derive(
    Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, JsonSchema,
)]
pub enum StackItemId {
    Card(StackCardId),
    ActivatedAbility(ActivatedAbilityId),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StackItemState {
    pub id: StackItemId,
    pub controller: PlayerName,
    pub targets: Option<EffectTargets>,
    pub additional_costs_paid: StackCardAdditionalCostsPaid,
    pub modal_choice: Option<ModelEffectChoiceIndex>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EffectTargets {
    /// A target for a standard effect.
    Standard(StandardEffectTarget),

    /// Target queue for an effect list. An entry of `None` indicates that the
    /// specified target was provided but is no longer valid on resolution, e.g.
    /// because a target character has been dissolved.
    ///
    /// During effect resolution, we pop targets from the list when required,
    /// i.e. it is assumed that this order will match the order in which targets
    /// are required for effects.
    EffectList(VecDeque<Option<StandardEffectTarget>>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StandardEffectTarget {
    Character(CardObjectId<CharacterId>),
    StackCard(CardObjectId<StackCardId>),
    VoidCardSet(BTreeSet<CardObjectId<VoidCardId>>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StackCardAdditionalCostsPaid {
    None,
    Energy(Energy),
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

impl StackItemId {
    pub fn underlying_card_id(&self) -> CardId {
        match self {
            StackItemId::Card(id) => id.card_id(),
            StackItemId::ActivatedAbility(activated) => activated.character_id.card_id(),
        }
    }
}

impl StackItemState {
    pub fn append_character_target(&mut self, character_id: CharacterId, object_id: ObjectId) {
        if let Some(targets) = &mut self.targets {
            targets.add(StandardEffectTarget::Character(CardObjectId {
                card_id: character_id,
                object_id,
            }));
        } else {
            self.targets =
                Some(EffectTargets::Standard(StandardEffectTarget::Character(CardObjectId {
                    card_id: character_id,
                    object_id,
                })));
        }
    }

    pub fn append_stack_card_target(&mut self, stack_card_id: StackCardId, object_id: ObjectId) {
        if let Some(targets) = &mut self.targets {
            targets.add(StandardEffectTarget::StackCard(CardObjectId {
                card_id: stack_card_id,
                object_id,
            }));
        } else {
            self.targets =
                Some(EffectTargets::Standard(StandardEffectTarget::StackCard(CardObjectId {
                    card_id: stack_card_id,
                    object_id,
                })));
        }
    }
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

    /// Returns the set of card IDs targeted by this effect.
    pub fn card_ids(&self) -> Vec<CardId> {
        match self {
            EffectTargets::Standard(target) => target.card_ids(),
            EffectTargets::EffectList(targets) => targets
                .iter()
                .filter_map(|target| target.as_ref().map(|target| target.card_ids()))
                .flatten()
                .collect(),
        }
    }
}

impl StandardEffectTarget {
    pub fn card_ids(&self) -> Vec<CardId> {
        match self {
            StandardEffectTarget::Character(id) => vec![id.card_id.card_id()],
            StandardEffectTarget::StackCard(id) => vec![id.card_id.card_id()],
            StandardEffectTarget::VoidCardSet(ids) => {
                ids.iter().map(|id| id.card_id.card_id()).collect()
            }
        }
    }
}
