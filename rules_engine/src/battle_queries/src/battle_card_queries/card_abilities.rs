use ability_data::ability::Ability;
use ability_data::cost::Cost;
use ability_data::effect::Effect;
use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::standard_effect::StandardEffect;
use ability_data::static_ability::{PlayFromVoid, StandardStaticAbility};
use ability_data::trigger_event::{TriggerEvent, TriggerKeyword};
use battle_state::battle_cards::ability_list::{AbilityData, AbilityList, CanPlayRestriction};
use battle_state::triggers::trigger::TriggerName;
use core_data::identifiers::AbilityNumber;
use core_data::numerics::Energy;
use enumset::EnumSet;
use tabula_data::card_definition::CardDefinition;

use crate::battle_card_queries::build_named_abilities;
use crate::card_ability_queries::{effect_queries, target_predicates};

#[derive(Debug, Copy, Clone)]
pub enum PredicateType {
    Character,
    Stack,
    Void,
}

pub struct EventEffectPredicate<'a> {
    pub effect: &'a StandardEffect,
    pub predicate: &'a Predicate,
    pub predicate_type: PredicateType,
}

/// Builds an ability list from a card definition for a specific identity.
pub fn build_from_definition(definition: &CardDefinition) -> AbilityList {
    let abilities = definition
        .abilities
        .iter()
        .enumerate()
        .map(|(i, ability)| (AbilityNumber(i), ability.clone()))
        .collect();
    build_ability_list(definition, abilities)
}

fn build_ability_list(
    definition: &CardDefinition,
    abilities: Vec<(AbilityNumber, Ability)>,
) -> AbilityList {
    let mut ability_list = AbilityList::default();

    for (ability_number, ability) in abilities {
        match ability {
            Ability::Event(event_ability) => {
                ability_list
                    .event_abilities
                    .push(AbilityData { ability_number, ability: event_ability.clone() });
            }
            Ability::Static(static_ability) => {
                ability_list
                    .static_abilities
                    .push(AbilityData { ability_number, ability: static_ability.clone() });
            }
            Ability::Activated(activated_ability) => {
                ability_list
                    .activated_abilities
                    .push(AbilityData { ability_number, ability: activated_ability.clone() });
            }
            Ability::Triggered(triggered_ability) => {
                ability_list
                    .triggered_abilities
                    .push(AbilityData { ability_number, ability: triggered_ability.clone() });
            }
            Ability::Named(named_ability) => {
                build_named_abilities::build(
                    definition,
                    &mut ability_list,
                    named_ability,
                    ability_number,
                );
            }
        }
    }

    ability_list.can_play_restriction = merge_can_play_restrictions(vec![
        compute_event_target_restriction(&ability_list),
        compute_event_additional_cost_restriction(&ability_list),
    ]);
    ability_list.battlefield_triggers = battlefield_triggers(&ability_list);
    ability_list.stack_triggers = stack_triggers(&ability_list);
    ability_list.has_battlefield_activated_abilities = !ability_list.activated_abilities.is_empty();
    ability_list.has_play_from_void_ability = has_play_from_void_ability(&ability_list);

    ability_list
}

fn merge_can_play_restrictions(
    restrictions: Vec<Option<CanPlayRestriction>>,
) -> Option<CanPlayRestriction> {
    if restrictions.iter().any(Option::is_none) {
        return None;
    }

    let specific_restrictions: Vec<CanPlayRestriction> = restrictions
        .into_iter()
        .flatten()
        .filter(|r| !matches!(r, CanPlayRestriction::Unrestricted))
        .collect();

    match &specific_restrictions[..] {
        [] => Some(CanPlayRestriction::Unrestricted),
        [restriction] => Some(*restriction),
        [..] => None,
    }
}

fn compute_event_target_restriction(list: &AbilityList) -> Option<CanPlayRestriction> {
    if list.event_abilities.iter().any(|data| matches!(data.ability.effect, Effect::Modal(_))) {
        return None;
    }

    let predicates = event_effect_predicates(list);
    let effect_predicate = match &predicates[..] {
        [] => {
            return Some(CanPlayRestriction::Unrestricted);
        }
        [predicate] => predicate,
        [..] => {
            return None;
        }
    };

    match effect_predicate.predicate_type {
        PredicateType::Character => {
            if effect_queries::is_dissolve_effect(effect_predicate.effect) {
                // Dissolve-specific restrictions since we need to check for the
                // 'prevent dissolve' status.
                match effect_predicate.predicate {
                    Predicate::Enemy(CardPredicate::Character) => {
                        Some(CanPlayRestriction::DissolveEnemyCharacter)
                    }
                    _ => None,
                }
            } else {
                match effect_predicate.predicate {
                    Predicate::Enemy(CardPredicate::Character) => {
                        Some(CanPlayRestriction::EnemyCharacterOnBattlefield)
                    }
                    _ => None,
                }
            }
        }
        PredicateType::Stack => match effect_predicate.predicate {
            Predicate::Enemy(CardPredicate::Card) => Some(CanPlayRestriction::EnemyCardOnStack),
            Predicate::Enemy(CardPredicate::Event) => {
                Some(CanPlayRestriction::EnemyEventCardOnStack)
            }
            Predicate::Enemy(CardPredicate::Character) => {
                Some(CanPlayRestriction::EnemyCharacterCardOnStack)
            }
            _ => None,
        },
        PredicateType::Void => None,
    }
}

fn event_effect_predicates(list: &AbilityList) -> Vec<EventEffectPredicate<'_>> {
    list.event_abilities
        .iter()
        .flat_map(|data| match &data.ability.effect {
            Effect::Effect(effect) => vec![
                target_predicates::get_character_target_predicate(effect).map(|predicate| {
                    EventEffectPredicate {
                        effect,
                        predicate,
                        predicate_type: PredicateType::Character,
                    }
                }),
                target_predicates::get_stack_target_predicate(effect).map(|predicate| {
                    EventEffectPredicate { effect, predicate, predicate_type: PredicateType::Stack }
                }),
                target_predicates::get_void_target_predicate(effect).map(|predicate| {
                    EventEffectPredicate { effect, predicate, predicate_type: PredicateType::Void }
                }),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>(),
            Effect::WithOptions(options) => vec![
                target_predicates::get_character_target_predicate(&options.effect).map(
                    |predicate| EventEffectPredicate {
                        effect: &options.effect,
                        predicate,
                        predicate_type: PredicateType::Character,
                    },
                ),
                target_predicates::get_stack_target_predicate(&options.effect).map(|predicate| {
                    EventEffectPredicate {
                        effect: &options.effect,
                        predicate,
                        predicate_type: PredicateType::Stack,
                    }
                }),
                target_predicates::get_void_target_predicate(&options.effect).map(|predicate| {
                    EventEffectPredicate {
                        effect: &options.effect,
                        predicate,
                        predicate_type: PredicateType::Void,
                    }
                }),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>(),
            Effect::List(effects) => effects
                .iter()
                .flat_map(|effect| {
                    vec![
                        target_predicates::get_character_target_predicate(&effect.effect).map(
                            |predicate| EventEffectPredicate {
                                effect: &effect.effect,
                                predicate,
                                predicate_type: PredicateType::Character,
                            },
                        ),
                        target_predicates::get_stack_target_predicate(&effect.effect).map(
                            |predicate| EventEffectPredicate {
                                effect: &effect.effect,
                                predicate,
                                predicate_type: PredicateType::Stack,
                            },
                        ),
                        target_predicates::get_void_target_predicate(&effect.effect).map(
                            |predicate| EventEffectPredicate {
                                effect: &effect.effect,
                                predicate,
                                predicate_type: PredicateType::Void,
                            },
                        ),
                    ]
                    .into_iter()
                    .flatten()
                })
                .collect(),
            Effect::ListWithOptions(list_with_options) => list_with_options
                .effects
                .iter()
                .flat_map(|effect| {
                    vec![
                        target_predicates::get_character_target_predicate(&effect.effect).map(
                            |predicate| EventEffectPredicate {
                                effect: &effect.effect,
                                predicate,
                                predicate_type: PredicateType::Character,
                            },
                        ),
                        target_predicates::get_stack_target_predicate(&effect.effect).map(
                            |predicate| EventEffectPredicate {
                                effect: &effect.effect,
                                predicate,
                                predicate_type: PredicateType::Stack,
                            },
                        ),
                        target_predicates::get_void_target_predicate(&effect.effect).map(
                            |predicate| EventEffectPredicate {
                                effect: &effect.effect,
                                predicate,
                                predicate_type: PredicateType::Void,
                            },
                        ),
                    ]
                    .into_iter()
                    .flatten()
                })
                .collect(),
            Effect::Modal(_) => vec![],
        })
        .collect()
}

fn compute_event_additional_cost_restriction(list: &AbilityList) -> Option<CanPlayRestriction> {
    // Check both event abilities' additional_cost and activated abilities' costs
    // (activated abilities on event cards represent additional costs)
    let costs: Vec<&Cost> = list
        .event_abilities
        .iter()
        .filter_map(|a| a.ability.additional_cost.as_ref())
        .chain(list.activated_abilities.iter().flat_map(|a| a.ability.costs.iter()))
        .collect();

    let cost = match costs[..] {
        [] => return Some(CanPlayRestriction::Unrestricted),
        [cost] => cost,
        _ => return None,
    };

    match cost {
        Cost::SpendOneOrMoreEnergy => {
            Some(CanPlayRestriction::AdditionalEnergyAvailable(Energy(1)))
        }
        _ => None,
    }
}

fn battlefield_triggers(list: &AbilityList) -> EnumSet<TriggerName> {
    let mut triggers = EnumSet::new();

    for ability in list.triggered_abilities.iter() {
        for trigger in watch_for_battlefield_triggers(&ability.ability.trigger) {
            triggers.insert(trigger);
        }
    }

    triggers
}

fn watch_for_battlefield_triggers(event: &TriggerEvent) -> EnumSet<TriggerName> {
    match event {
        TriggerEvent::Materialize(..) => {
            let mut triggers = EnumSet::new();
            triggers.insert(TriggerName::Materialized);
            triggers
        }
        TriggerEvent::Keywords(keywords) => {
            let mut triggers = EnumSet::new();
            for keyword in keywords {
                match keyword {
                    TriggerKeyword::Materialized => {
                        triggers.insert(TriggerName::Materialized);
                    }
                    TriggerKeyword::Judgment => {
                        triggers.insert(TriggerName::Judgment);
                    }
                    TriggerKeyword::Dissolved => {
                        triggers.insert(TriggerName::Dissolved);
                    }
                }
            }
            triggers
        }
        TriggerEvent::Play(..) => {
            let mut triggers = EnumSet::new();
            triggers.insert(TriggerName::PlayedCard);
            triggers
        }
        TriggerEvent::PlayCardsInTurn(..) => {
            let mut triggers = EnumSet::new();
            triggers.insert(TriggerName::PlayedCard);
            triggers
        }
        TriggerEvent::PlayDuringTurn(..) => {
            let mut triggers = EnumSet::new();
            triggers.insert(TriggerName::PlayedCard);
            triggers
        }
        TriggerEvent::PlayFromHand(..) => {
            let mut triggers = EnumSet::new();
            triggers.insert(TriggerName::PlayedCardFromHand);
            triggers
        }
        TriggerEvent::PutIntoVoid(..) => {
            let mut triggers = EnumSet::new();
            triggers.insert(TriggerName::PutIntoVoid);
            triggers
        }
        TriggerEvent::LeavesPlay(..) => {
            let mut triggers = EnumSet::new();
            triggers.insert(TriggerName::Banished);
            triggers.insert(TriggerName::Dissolved);
            triggers
        }
        _ => todo!("Implement watch_for_trigger() for {:?}", event),
    }
}

fn stack_triggers(list: &AbilityList) -> EnumSet<TriggerName> {
    let mut triggers = EnumSet::new();

    for ability_data in &list.static_abilities {
        if let StandardStaticAbility::PlayFromVoid { .. } =
            ability_data.ability.standard_static_ability()
        {
            triggers.insert(TriggerName::PlayedCardFromVoid);
        }
    }
    triggers
}

fn has_play_from_void_ability(list: &AbilityList) -> bool {
    for ability in list.static_abilities.iter() {
        match ability.ability.standard_static_ability() {
            StandardStaticAbility::PlayFromVoid(PlayFromVoid { .. }) => {
                return true;
            }
            StandardStaticAbility::PlayOnlyFromVoid => {
                return true;
            }
            _ => {}
        }
    }

    false
}
