use ability_data::effect::Effect;
use ability_data::named_ability::NamedAbility;
use ability_data::predicate::Predicate;
use ability_data::standard_effect::StandardEffect;
use ability_data::static_ability::{PlayFromVoid, StandardStaticAbility, StaticAbility};
use battle_state::battle_cards::ability_list::{AbilityConfiguration, AbilityData, AbilityList};
use core_data::identifiers::AbilityNumber;
use tabula_data::card_definition::CardDefinition;

/// Expands a named ability into its constituent abilities.
pub fn build(
    definition: &CardDefinition,
    ability_list: &mut AbilityList,
    named: NamedAbility,
    ability_number: AbilityNumber,
) {
    match named {
        NamedAbility::Reclaim(energy_cost) => {
            ability_list.static_abilities.push(AbilityData {
                ability_number,
                ability: StaticAbility::StaticAbility(StandardStaticAbility::PlayFromVoid(
                    PlayFromVoid {
                        energy_cost: energy_cost.or_else(|| definition.energy_cost),
                        additional_cost: None,
                        if_you_do: Some(Effect::Effect(StandardEffect::BanishWhenLeavesPlay {
                            target: Predicate::This,
                        })),
                    },
                )),
                configuration: AbilityConfiguration::default(),
            });
        }
    }
}
