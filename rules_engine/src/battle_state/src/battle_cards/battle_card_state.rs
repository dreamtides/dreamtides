use core_data::identifiers::CardName;
use core_data::types::PlayerName;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::battle_cards::ability_list::CanPlayRestriction;
use crate::battle_player::player_map::PlayerMap;

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

/// Core state data for a card within a battle.
#[derive(Clone, Debug)]
pub struct BattleCardState {
    /// The name of this card.
    pub name: CardName,

    /// The player who owns this card.
    pub owner: PlayerName,

    /// The object ID of this card.
    pub object_id: ObjectId,

    /// If true, the card is revealed to the indicated player while in its
    /// current zone. This is not required for the card to be revealed in public
    /// zones like the battlefield, only to override normal visibility rules.
    ///
    /// This flag is automatically cleared when the card is moved to a new zone.
    pub revealed_to_player_override: PlayerMap<bool>,

    /// Restriction on playing this card, as a performance optimization.
    pub can_play_restriction: Option<CanPlayRestriction>,
}
