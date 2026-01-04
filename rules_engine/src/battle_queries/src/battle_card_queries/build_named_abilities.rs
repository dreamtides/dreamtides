use ability_data::effect::Effect;
use ability_data::named_ability::NamedAbility;
use ability_data::predicate::Predicate;
use ability_data::standard_effect::StandardEffect;
use ability_data::static_ability::{PlayFromVoid, StandardStaticAbility, StaticAbility};
use battle_state::battle_cards::ability_list::{AbilityData, AbilityList};
use core_data::identifiers::AbilityNumber;
use core_data::numerics::Energy;
use tabula_data::card_definitions::card_definition::CardDefinition;

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
                        energy_cost: energy_cost.or(definition.energy_cost),
                        additional_cost: None,
                        if_you_do: Some(Effect::Effect(StandardEffect::BanishWhenLeavesPlay {
                            target: Predicate::This,
                        })),
                    },
                )),
            });
        }
        NamedAbility::ReclaimForCost(cost) => {
            ability_list.static_abilities.push(AbilityData {
                ability_number,
                ability: StaticAbility::StaticAbility(StandardStaticAbility::PlayFromVoid(
                    PlayFromVoid {
                        energy_cost: Some(Energy(0)),
                        additional_cost: Some(cost),
                        if_you_do: Some(Effect::Effect(StandardEffect::BanishWhenLeavesPlay {
                            target: Predicate::This,
                        })),
                    },
                )),
            });
        }
    }
}
