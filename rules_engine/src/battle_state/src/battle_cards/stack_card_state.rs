use bit_set::BitSet;
use core_data::types::PlayerName;

use crate::battle::card_id::StackCardId;

#[derive(Clone, Debug)]
pub struct StackCardState {
    pub id: StackCardId,
    pub controller: PlayerName,
    pub targets: BitSet,
}
