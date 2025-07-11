use serde::Serialize;

use crate::debug::debug_card_state::{DebugCardState, DebugStackItemState};

#[derive(Debug, Clone, Serialize)]
pub struct DebugAllCards {
    pub p1_battlefield: Vec<DebugCardState>,
    pub p2_battlefield: Vec<DebugCardState>,
    pub p1_void: Vec<DebugCardState>,
    pub p2_void: Vec<DebugCardState>,
    pub p1_hand: Vec<DebugCardState>,
    pub p2_hand: Vec<DebugCardState>,
    pub p1_shuffled_into_deck: Vec<DebugCardState>,
    pub p2_shuffled_into_deck: Vec<DebugCardState>,
    pub p1_top_of_deck: Vec<DebugCardState>,
    pub p2_top_of_deck: Vec<DebugCardState>,
    pub stack: Vec<DebugStackItemState>,
    pub p1_banished: Vec<DebugCardState>,
    pub p2_banished: Vec<DebugCardState>,
}
