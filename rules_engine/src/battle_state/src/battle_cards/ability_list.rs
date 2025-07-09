use ability_data::ability::EventAbility;
use ability_data::activated_ability::ActivatedAbility;
use ability_data::static_ability::StaticAbility;
use ability_data::triggered_ability::TriggeredAbility;
use core_data::card_types::CardType;
use core_data::identifiers::AbilityNumber;
use core_data::numerics::Energy;
use enumset::EnumSet;

use crate::triggers::trigger::TriggerName;

/// Abilities of a card which can be applied during a battle.
///
/// This is a wrapper around `Ability` which stores some precomputed state
/// information to improve search performance.
#[derive(Debug, Clone, Default)]
pub struct AbilityList {
    pub event_abilities: Vec<AbilityData<EventAbility>>,
    pub static_abilities: Vec<AbilityData<StaticAbility>>,
    pub activated_abilities: Vec<AbilityData<ActivatedAbility>>,
    pub triggered_abilities: Vec<AbilityData<TriggeredAbility>>,

    /// A field indicating restrictions on playing this card.
    pub can_play_restriction: Option<CanPlayRestriction>,

    /// Triggers which can fire when this card is on the battlefield.
    pub battlefield_triggers: EnumSet<TriggerName>,

    /// Indicates whether this card has activated abilities that can be used on
    /// the battlefield.
    pub has_battlefield_activated_abilities: bool,
}

/// Wrapper around an ability which stores additional metadata.
#[derive(Debug, Clone)]
pub struct AbilityData<T> {
    pub ability_number: AbilityNumber,
    pub ability: T,
    pub configuration: AbilityConfiguration,
}

/// Configuration options for an ability.
#[derive(Debug, Clone, Default)]
pub struct AbilityConfiguration {
    /// Label to display when selecting a target for this ability.
    pub targeting_prompt: Option<String>,

    /// Label to display when selecting a choice for this ability.
    pub choice_prompt: Option<String>,

    /// Label to display when selecting an additional cost for this ability.
    pub additional_cost_prompt: Option<String>,
}

/// A restriction on playing a card.
///
/// This is a performance optimization because determining card play
/// legality is very expensive. If a value is present here, it means that
/// that we do not need to examine the the card's abilities to determine
/// whether it can be played -- it is sufficient to look at this restriction
/// in addition to standard things like whether its energy cost can be paid.
#[derive(Debug, Clone, Copy)]
pub enum CanPlayRestriction {
    Unrestricted,
    EnemyCharacter,
    EnemyStackCard,
    EnemyStackCardOfType(CardType),
    AdditionalEnergyAvailable(Energy),
}
