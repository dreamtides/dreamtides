use ability_data::ability::Ability;
use core_data::identifiers::{CardId, CardIdentity};
use core_data::types::PlayerName;

use crate::battle::turn_data::TurnData;
use crate::battle_cards::additional_cost_choice_data::AdditionalCostData;
use crate::battle_cards::card_id::{CharacterId, ObjectId, StackCardId};
use crate::battle_cards::card_properties::CardProperties;
use crate::battle_cards::zone::Zone;

#[derive(Clone, Debug)]
pub struct CardData {
    /// Unique identifier for this card
    pub id: CardId,

    /// Identifies the art & rules text for this card
    pub identity: CardIdentity,

    /// The owner of this card.
    pub owner: PlayerName,

    /// The zone this card is currently in.
    pub zone: Zone,

    /// Unique identifier for this card within a zone.
    pub object_id: ObjectId,

    /// Properties of this card.
    pub properties: CardProperties,

    /// Abilities of this card.
    pub abilities: Vec<Ability>,

    /// Whether this card is revealed to its owner.
    pub revealed_to_owner: bool,

    /// Whether this card is revealed to its opponent.
    pub revealed_to_opponent: bool,

    /// The targets of this card while it is on the stack.
    ///
    /// This value is cleared during card resolution before it is removed from
    /// the stack.
    pub targets: Vec<TargetId>,

    /// Choices about additional costs to play this card.
    ///
    /// Additional costs are paid immediately when a card is put on the stack.
    /// They're persisted here so that card effects can reference what cost was
    /// paid, if any.
    ///
    /// This value is cleared during card resolution before it is removed from
    /// the stack.
    pub additional_cost_choices: Vec<AdditionalCostData>,

    /// Turn on which this card last entered its current zone.
    ///
    /// Do not mutate this field directly, use the `move_card` module instead.
    pub turn_entered_current_zone: TurnData,
}

impl CardData {
    /// Whether this card is revealed to the given player.
    ///
    /// Note that cards in Dreamtides currently have no concept of "face down"
    /// or "face up", they're simply revealed or not revealed to a given player.
    pub fn is_revealed_to(&self, player_name: PlayerName) -> bool {
        match (self.owner, player_name) {
            (PlayerName::One, PlayerName::One) => self.revealed_to_owner,
            (PlayerName::One, PlayerName::Two) => self.revealed_to_opponent,
            (PlayerName::Two, PlayerName::One) => self.revealed_to_opponent,
            (PlayerName::Two, PlayerName::Two) => self.revealed_to_owner,
        }
    }

    /// The controller of this card.
    pub fn controller(&self) -> PlayerName {
        self.owner
    }
}

/// Represents a target for a card or ability.
///
/// Targets are scoped to a specific [ObjectId] -- if a target changes zones, it
/// ceases to be a valid target.
#[derive(Clone, Debug)]
pub enum TargetId {
    StackCard(StackCardId, ObjectId),
    Character(CharacterId, ObjectId),
}

impl TargetId {
    pub fn object_id(&self) -> ObjectId {
        match self {
            TargetId::StackCard(_, object_id) => *object_id,
            TargetId::Character(_, object_id) => *object_id,
        }
    }
}
