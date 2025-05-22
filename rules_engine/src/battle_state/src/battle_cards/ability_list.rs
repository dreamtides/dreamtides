use ability_data::ability::EventAbility;
use ability_data::activated_ability::ActivatedAbility;
use ability_data::static_ability::StaticAbility;
use ability_data::triggered_ability::TriggeredAbility;
use core_data::card_types::CardType;
use core_data::identifiers::AbilityNumber;
use core_data::numerics::Energy;

/// Abilities of a card which can be applied during a battle.
///
/// This is a wrapper around `Ability` which stores some precomputed state
/// information to improve search performance.
#[derive(Debug, Clone, Default)]
pub struct AbilityList {
    pub event_abilities: Vec<(AbilityNumber, EventAbility)>,
    pub static_abilities: Vec<(AbilityNumber, StaticAbility)>,
    pub activated_abilities: Vec<(AbilityNumber, ActivatedAbility)>,
    pub triggered_abilities: Vec<(AbilityNumber, TriggeredAbility)>,

    /// A field indicating restrictions on playing this card.
    ///
    /// This is a performance optimization because determining card play
    /// legality is very expensive. If a value is present here, it means that
    /// that we do not need to examine the the card's abilities to determine
    /// whether it can be played -- it is sufficient to look at this restriction
    /// in addition to standard things like whether its energy cost can be paid.
    pub can_play_restriction: Option<CanPlayRestriction>,
}

#[derive(Debug, Clone, Copy)]
pub enum CanPlayRestriction {
    Unrestricted,
    EnemyCharacter,
    EnemyStackCard,
    EnemyStackCardOfType(CardType),
    AdditionalEnergyAvailable(Energy),
}
