use core_data::types::PlayerName;

use crate::battle::card_id::{CharacterId, StackCardId};

#[derive(Clone, Copy, Debug)]
pub struct StackCardState {
    pub id: StackCardId,
    pub controller: PlayerName,
    pub targets: StackCardTargets,
}

#[derive(Clone, Copy, Debug)]
pub enum StackCardTargets {
    None,
    Character(CharacterId),
    StackCard(StackCardId),
}
