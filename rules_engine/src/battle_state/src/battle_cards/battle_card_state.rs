use core_data::card_types::CardType;
use core_data::numerics::{Energy, Spark};
use core_data::types::PlayerName;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::battle::battle_card_definitions::BattleCardIdentity;
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

/// A card ID paired with an object ID.
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
pub struct CardObjectId<T> {
    pub card_id: T,
    pub object_id: ObjectId,
}

/// Core state data for a card within a battle.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BattleCardState {
    /// The identity of this card.
    ///
    /// Uniquely identifies this card's rules text.
    pub identity: BattleCardIdentity,

    /// The player who owns this card.
    pub owner: PlayerName,

    /// The object ID of this card.
    pub object_id: ObjectId,

    /// Base energy cost for this card.
    pub base_energy_cost: Option<Energy>,

    /// Base spark for this card, if it is a character.
    pub base_spark: Option<Spark>,

    /// Card type for this card.
    pub card_type: CardType,

    /// Fast flag for this card.
    pub is_fast: bool,

    /// If true, the card is revealed to the indicated player while in its
    /// current zone. This is not required for the card to be revealed in public
    /// zones like the battlefield, only to override normal visibility rules.
    ///
    /// This flag is automatically cleared when the card is moved to a new zone.
    pub revealed_to_player_override: PlayerMap<bool>,

    /// Restriction on playing this card, as a performance optimization.
    pub can_play_restriction: Option<CanPlayRestriction>,
}
