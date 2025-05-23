use core_data::identifiers::CardName;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::battle_cards::ability_list::CanPlayRestriction;

/// Identifies a card within a zone.
///
/// A new ObjectId is assigned each time a card changes zones, meaning that it
/// can be used for targeting effects that end when the card changes zones.
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Eq,
    PartialEq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    JsonSchema,
)]
pub struct ObjectId(pub usize);

/// Core state data for a card
#[derive(Clone, Debug)]
pub struct BattleCardState {
    pub name: CardName,
    pub object_id: ObjectId,
    pub can_play_restriction: Option<CanPlayRestriction>,
}
