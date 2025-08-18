use std::sync::{Arc, OnceLock};

use ability_data::ability::{Ability, EventAbility};
use ability_data::activated_ability::{ActivatedAbility, ActivatedAbilityOptions};
use ability_data::cost::Cost;
use ability_data::effect::{Effect, EffectWithOptions, ModalEffectChoice};
use ability_data::named_ability::NamedAbility;
use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::quantity_expression_data::QuantityExpression;
use ability_data::standard_effect::StandardEffect;
use ability_data::static_ability::{PlayFromVoid, StandardStaticAbility};
use ability_data::trigger_event::{PlayerTurn, TriggerEvent};
use ability_data::triggered_ability::TriggeredAbility;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardIdType;
use battle_state::battle_cards::ability_list::{
    AbilityConfiguration, AbilityData, AbilityList, CanPlayRestriction,
};
use battle_state::triggers::trigger::TriggerName;
use core_data::identifiers::{AbilityNumber, CardIdentity, CardName};
use core_data::numerics::{Energy, Spark};
use enumset::EnumSet;
use parking_lot::RwLock;

use crate::battle_card_queries::{build_named_abilities, card};
use crate::card_ability_queries::{effect_queries, target_predicates};

type AbilitySlot = Arc<OnceLock<Arc<AbilityList>>>;
type AbilitySlots = Vec<AbilitySlot>;
type AbilityTable = RwLock<AbilitySlots>;

static ABILITY_TABLE: OnceLock<AbilityTable> = OnceLock::new();

pub fn query(battle: &BattleState, card_id: impl CardIdType) -> Arc<AbilityList> {
    query_by_identity(card::get(battle, card_id).identity)
}

fn ability_table() -> &'static AbilityTable {
    ABILITY_TABLE.get_or_init(|| RwLock::new(Vec::new()))
}

fn slot_for_index(index: usize) -> AbilitySlot {
    let table = ability_table();
    {
        let guard = table.read();
        if guard.len() > index {
            return guard[index].clone();
        }
    }
    let mut guard = table.write();
    if guard.len() <= index {
        guard.resize_with(index + 1, || Arc::new(OnceLock::new()));
    }
    guard[index].clone()
}

pub fn query_by_identity(identity: CardIdentity) -> Arc<AbilityList> {
    let slot = slot_for_index(identity.0);
    slot.get_or_init(|| Arc::new(build_for(identity.tmp_to_card_name()))).clone()
}

fn build_ability_list(
    card_identity: CardIdentity,
    abilities: Vec<(AbilityNumber, Ability, AbilityConfiguration)>,
) -> AbilityList {
    let mut ability_list = AbilityList::default();

    for (ability_number, ability, configuration) in abilities {
        match ability {
            Ability::Event(event_ability) => {
                ability_list.event_abilities.push(AbilityData {
                    ability_number,
                    ability: event_ability.clone(),
                    configuration,
                });
            }
            Ability::Static(static_ability) => {
                ability_list.static_abilities.push(AbilityData {
                    ability_number,
                    ability: static_ability.clone(),
                    configuration,
                });
            }
            Ability::Activated(activated_ability) => {
                ability_list.activated_abilities.push(AbilityData {
                    ability_number,
                    ability: activated_ability.clone(),
                    configuration,
                });
            }
            Ability::Triggered(triggered_ability) => {
                ability_list.triggered_abilities.push(AbilityData {
                    ability_number,
                    ability: triggered_ability.clone(),
                    configuration,
                });
            }
            Ability::Named(named_ability) => {
                build_named_abilities::build(
                    card_identity,
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
    if restrictions.iter().any(|r| r.is_none()) {
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

fn event_effect_predicates(list: &AbilityList) -> Vec<EventEffectPredicate> {
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
            Effect::Modal(_) => vec![],
        })
        .collect()
}

fn compute_event_additional_cost_restriction(list: &AbilityList) -> Option<CanPlayRestriction> {
    let costs: Vec<&Cost> =
        list.event_abilities.iter().filter_map(|a| a.ability.additional_cost.as_ref()).collect();

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
        triggers.insert(watch_for_battlefield_trigger(&ability.ability.trigger));
    }

    triggers
}

fn watch_for_battlefield_trigger(event: &TriggerEvent) -> TriggerName {
    match event {
        TriggerEvent::Materialize(..) => TriggerName::Materialized,
        TriggerEvent::Play(..) => TriggerName::PlayedCard,
        TriggerEvent::PlayDuringTurn(..) => TriggerName::PlayedCard,
        TriggerEvent::PlayFromHand(..) => TriggerName::PlayedCardFromHand,
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

fn build_for(name: CardName) -> AbilityList {
    match name {
        CardName::TestVanillaCharacter => {
            build_ability_list(CardName::TestVanillaCharacter.tmp_to_card_identity(), vec![])
        }
        CardName::TestDissolve => {
            build_ability_list(CardName::TestDissolve.tmp_to_card_identity(), vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: None,
                    effect: Effect::Effect(StandardEffect::DissolveCharacter {
                        target: Predicate::Enemy(CardPredicate::Character),
                    }),
                }),
                AbilityConfiguration {
                    targeting_prompt: Some("Select an enemy character.".to_string()),
                    ..Default::default()
                },
            )])
        }
        CardName::TestNamedDissolve => {
            build_ability_list(CardName::TestNamedDissolve.tmp_to_card_identity(), vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: None,
                    effect: Effect::Effect(StandardEffect::DissolveCharacter {
                        target: Predicate::Enemy(CardPredicate::Character),
                    }),
                }),
                AbilityConfiguration {
                    targeting_prompt: Some("Select an enemy character.".to_string()),
                    ..Default::default()
                },
            )])
        }
        CardName::TestCounterspellUnlessPays => {
            build_ability_list(CardName::TestCounterspellUnlessPays.tmp_to_card_identity(), vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: None,
                    effect: Effect::Effect(StandardEffect::CounterspellUnlessPaysCost {
                        target: Predicate::Enemy(CardPredicate::Event),
                        cost: Cost::Energy(Energy(2)),
                    }),
                }),
                AbilityConfiguration {
                    targeting_prompt: Some("Select an enemy event.".to_string()),
                    choice_prompt: Some("Pay 2\u{f7e4} to resolve this card?".to_string()),
                    ..Default::default()
                },
            )])
        }
        CardName::TestCounterspell => {
            build_ability_list(CardName::TestCounterspell.tmp_to_card_identity(), vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: None,
                    effect: Effect::Effect(StandardEffect::Counterspell {
                        target: Predicate::Enemy(CardPredicate::Card),
                    }),
                }),
                AbilityConfiguration {
                    targeting_prompt: Some("Select an enemy card.".to_string()),
                    ..Default::default()
                },
            )])
        }
        CardName::TestVariableEnergyDraw => {
            build_ability_list(CardName::TestVariableEnergyDraw.tmp_to_card_identity(), vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: Some(Cost::SpendOneOrMoreEnergy),
                    effect: Effect::Effect(StandardEffect::DrawCardsForEach {
                        count: 1,
                        for_each: QuantityExpression::ForEachEnergySpentOnThisCard,
                    }),
                }),
                AbilityConfiguration {
                    additional_cost_prompt: Some("Pay one or more \u{f7e4}.".to_string()),
                    ..Default::default()
                },
            )])
        }
        CardName::TestDrawOne => {
            build_ability_list(CardName::TestDrawOne.tmp_to_card_identity(), vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: None,
                    effect: Effect::Effect(StandardEffect::DrawCards { count: 1 }),
                }),
                AbilityConfiguration { ..Default::default() },
            )])
        }
        CardName::TestTriggerGainSparkWhenMaterializeAnotherCharacter => build_ability_list(
            CardName::TestTriggerGainSparkWhenMaterializeAnotherCharacter.tmp_to_card_identity(),
            vec![(
                AbilityNumber(0),
                Ability::Triggered(TriggeredAbility {
                    trigger: TriggerEvent::Materialize(Predicate::Another(
                        CardPredicate::Character,
                    )),
                    effect: Effect::Effect(StandardEffect::GainsSpark {
                        target: Predicate::This,
                        gains: Spark(1),
                    }),
                    options: None,
                }),
                AbilityConfiguration::default(),
            )],
        ),
        CardName::TestTriggerGainSparkOnPlayCardEnemyTurn => build_ability_list(
            CardName::TestTriggerGainSparkOnPlayCardEnemyTurn.tmp_to_card_identity(),
            vec![(
                AbilityNumber(0),
                Ability::Triggered(TriggeredAbility {
                    trigger: TriggerEvent::PlayDuringTurn(
                        Predicate::Your(CardPredicate::Card),
                        PlayerTurn::EnemyTurn,
                    ),
                    effect: Effect::Effect(StandardEffect::GainsSpark {
                        target: Predicate::This,
                        gains: Spark(1),
                    }),
                    options: None,
                }),
                AbilityConfiguration::default(),
            )],
        ),
        CardName::TestTriggerGainTwoSparkOnPlayCardEnemyTurn => build_ability_list(
            CardName::TestTriggerGainTwoSparkOnPlayCardEnemyTurn.tmp_to_card_identity(),
            vec![(
                AbilityNumber(0),
                Ability::Triggered(TriggeredAbility {
                    trigger: TriggerEvent::PlayDuringTurn(
                        Predicate::Your(CardPredicate::Card),
                        PlayerTurn::EnemyTurn,
                    ),
                    effect: Effect::Effect(StandardEffect::GainsSpark {
                        target: Predicate::This,
                        gains: Spark(2),
                    }),
                    options: None,
                }),
                AbilityConfiguration::default(),
            )],
        ),
        CardName::TestActivatedAbilityDrawCard => {
            build_ability_list(CardName::TestActivatedAbilityDrawCard.tmp_to_card_identity(), vec![
                (
                    AbilityNumber(0),
                    Ability::Activated(ActivatedAbility {
                        costs: vec![Cost::Energy(Energy(1))],
                        effect: Effect::Effect(StandardEffect::DrawCards { count: 1 }),
                        options: None,
                    }),
                    AbilityConfiguration::default(),
                ),
            ])
        }
        CardName::TestMultiActivatedAbilityDrawCardCharacter => build_ability_list(
            CardName::TestMultiActivatedAbilityDrawCardCharacter.tmp_to_card_identity(),
            vec![(
                AbilityNumber(0),
                Ability::Activated(ActivatedAbility {
                    costs: vec![Cost::Energy(Energy(1))],
                    effect: Effect::Effect(StandardEffect::DrawCards { count: 1 }),
                    options: Some(ActivatedAbilityOptions { is_multi: true, is_fast: false }),
                }),
                AbilityConfiguration::default(),
            )],
        ),
        CardName::TestFastActivatedAbilityDrawCardCharacter => build_ability_list(
            CardName::TestFastActivatedAbilityDrawCardCharacter.tmp_to_card_identity(),
            vec![(
                AbilityNumber(0),
                Ability::Activated(ActivatedAbility {
                    costs: vec![Cost::Energy(Energy(1))],
                    effect: Effect::Effect(StandardEffect::DrawCards { count: 1 }),
                    options: Some(ActivatedAbilityOptions { is_multi: false, is_fast: true }),
                }),
                AbilityConfiguration::default(),
            )],
        ),
        CardName::TestFastMultiActivatedAbilityDrawCardCharacter => build_ability_list(
            CardName::TestFastMultiActivatedAbilityDrawCardCharacter.tmp_to_card_identity(),
            vec![(
                AbilityNumber(0),
                Ability::Activated(ActivatedAbility {
                    costs: vec![Cost::Energy(Energy(3))],
                    effect: Effect::Effect(StandardEffect::DrawCards { count: 1 }),
                    options: Some(ActivatedAbilityOptions { is_multi: true, is_fast: true }),
                }),
                AbilityConfiguration::default(),
            )],
        ),
        CardName::TestActivatedAbilityDissolveCharacter => build_ability_list(
            CardName::TestActivatedAbilityDissolveCharacter.tmp_to_card_identity(),
            vec![(
                AbilityNumber(0),
                Ability::Activated(ActivatedAbility {
                    costs: vec![Cost::Energy(Energy(2))],
                    effect: Effect::Effect(StandardEffect::DissolveCharacter {
                        target: Predicate::Enemy(CardPredicate::Character),
                    }),
                    options: None,
                }),
                AbilityConfiguration {
                    targeting_prompt: Some("Select an enemy character.".to_string()),
                    ..Default::default()
                },
            )],
        ),
        CardName::TestDualActivatedAbilityCharacter => build_ability_list(
            CardName::TestDualActivatedAbilityCharacter.tmp_to_card_identity(),
            vec![
                (
                    AbilityNumber(0),
                    Ability::Activated(ActivatedAbility {
                        costs: vec![Cost::Energy(Energy(1))],
                        effect: Effect::Effect(StandardEffect::DrawCards { count: 1 }),
                        options: None,
                    }),
                    AbilityConfiguration::default(),
                ),
                (
                    AbilityNumber(1),
                    Ability::Activated(ActivatedAbility {
                        costs: vec![Cost::Energy(Energy(2))],
                        effect: Effect::Effect(StandardEffect::DrawCards { count: 2 }),
                        options: None,
                    }),
                    AbilityConfiguration::default(),
                ),
            ],
        ),
        CardName::TestForeseeOne => {
            build_ability_list(CardName::TestForeseeOne.tmp_to_card_identity(), vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: None,
                    effect: Effect::Effect(StandardEffect::Foresee { count: 1 }),
                }),
                AbilityConfiguration::default(),
            )])
        }
        CardName::TestForeseeTwo => {
            build_ability_list(CardName::TestForeseeTwo.tmp_to_card_identity(), vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: None,
                    effect: Effect::Effect(StandardEffect::Foresee { count: 2 }),
                }),
                AbilityConfiguration::default(),
            )])
        }
        CardName::TestForeseeOneDrawACard => {
            build_ability_list(CardName::TestForeseeOneDrawACard.tmp_to_card_identity(), vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: None,
                    effect: Effect::List(vec![
                        EffectWithOptions::new(StandardEffect::Foresee { count: 1 }),
                        EffectWithOptions::new(StandardEffect::DrawCards { count: 1 }),
                    ]),
                }),
                AbilityConfiguration::default(),
            )])
        }
        CardName::TestDrawOneReclaim => {
            build_ability_list(CardName::TestDrawOneReclaim.tmp_to_card_identity(), vec![
                (
                    AbilityNumber(0),
                    Ability::Event(EventAbility {
                        additional_cost: None,
                        effect: Effect::Effect(StandardEffect::DrawCards { count: 1 }),
                    }),
                    AbilityConfiguration::default(),
                ),
                (
                    AbilityNumber(1),
                    Ability::Named(NamedAbility::Reclaim(Some(Energy(1)))),
                    AbilityConfiguration::default(),
                ),
            ])
        }
        CardName::TestReturnVoidCardToHand => {
            build_ability_list(CardName::TestReturnVoidCardToHand.tmp_to_card_identity(), vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: None,
                    effect: Effect::Effect(StandardEffect::ReturnFromYourVoidToHand {
                        target: Predicate::YourVoid(CardPredicate::Card),
                    }),
                }),
                AbilityConfiguration {
                    targeting_prompt: Some("Select a card from your void.".to_string()),
                    ..Default::default()
                },
            )])
        }
        CardName::TestReturnOneOrTwoVoidEventCardsToHand => build_ability_list(
            CardName::TestReturnOneOrTwoVoidEventCardsToHand.tmp_to_card_identity(),
            vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: None,
                    effect: Effect::Effect(StandardEffect::ReturnUpToCountFromYourVoidToHand {
                        target: Predicate::YourVoid(CardPredicate::Event),
                        count: 2,
                    }),
                }),
                AbilityConfiguration {
                    targeting_prompt: Some("Select one or two events from your void.".to_string()),
                    ..Default::default()
                },
            )],
        ),
        CardName::TestModalDrawOneOrDrawTwo => {
            build_ability_list(CardName::TestModalDrawOneOrDrawTwo.tmp_to_card_identity(), vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: None,
                    effect: Effect::Modal(vec![
                        ModalEffectChoice {
                            energy_cost: Energy(1),
                            effect: Effect::Effect(StandardEffect::DrawCards { count: 1 }),
                        },
                        ModalEffectChoice {
                            energy_cost: Energy(3),
                            effect: Effect::Effect(StandardEffect::DrawCards { count: 2 }),
                        },
                    ]),
                }),
                AbilityConfiguration { ..Default::default() },
            )])
        }
        CardName::TestModalDrawOneOrDissolveEnemy => build_ability_list(
            CardName::TestModalDrawOneOrDissolveEnemy.tmp_to_card_identity(),
            vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: None,
                    effect: Effect::Modal(vec![
                        ModalEffectChoice {
                            energy_cost: Energy(1),
                            effect: Effect::Effect(StandardEffect::DrawCards { count: 1 }),
                        },
                        ModalEffectChoice {
                            energy_cost: Energy(2),
                            effect: Effect::Effect(StandardEffect::DissolveCharacter {
                                target: Predicate::Enemy(CardPredicate::Character),
                            }),
                        },
                    ]),
                }),
                AbilityConfiguration { ..Default::default() },
            )],
        ),
        CardName::TestReturnToHand => {
            build_ability_list(CardName::TestReturnToHand.tmp_to_card_identity(), vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: None,
                    effect: Effect::Effect(StandardEffect::ReturnToHand {
                        target: Predicate::Enemy(CardPredicate::Character),
                    }),
                }),
                AbilityConfiguration {
                    targeting_prompt: Some("Select an enemy character.".to_string()),
                    ..Default::default()
                },
            )])
        }
        CardName::TestPreventDissolveThisTurn => {
            build_ability_list(CardName::TestPreventDissolveThisTurn.tmp_to_card_identity(), vec![
                (
                    AbilityNumber(0),
                    Ability::Event(EventAbility {
                        additional_cost: None,
                        effect: Effect::Effect(StandardEffect::PreventDissolveThisTurn {
                            target: Predicate::Your(CardPredicate::Character),
                        }),
                    }),
                    AbilityConfiguration::default(),
                ),
            ])
        }
        CardName::TestCounterspellCharacter => {
            build_ability_list(CardName::TestCounterspellCharacter.tmp_to_card_identity(), vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: None,
                    effect: Effect::Effect(StandardEffect::Counterspell {
                        target: Predicate::Enemy(CardPredicate::Character),
                    }),
                }),
                AbilityConfiguration::default(),
            )])
        }
        CardName::TestForeseeOneReclaim => {
            build_ability_list(CardName::TestForeseeOneReclaim.tmp_to_card_identity(), vec![
                (
                    AbilityNumber(0),
                    Ability::Event(EventAbility {
                        additional_cost: None,
                        effect: Effect::Effect(StandardEffect::Foresee { count: 1 }),
                    }),
                    AbilityConfiguration::default(),
                ),
                (
                    AbilityNumber(1),
                    Ability::Named(NamedAbility::Reclaim(Some(Energy(3)))),
                    AbilityConfiguration::default(),
                ),
            ])
        }
        CardName::TestForeseeOneDrawReclaim => {
            build_ability_list(CardName::TestForeseeOneDrawReclaim.tmp_to_card_identity(), vec![
                (
                    AbilityNumber(0),
                    Ability::Event(EventAbility {
                        additional_cost: None,
                        effect: Effect::List(vec![
                            EffectWithOptions::new(StandardEffect::Foresee { count: 1 }),
                            EffectWithOptions::new(StandardEffect::DrawCards { count: 1 }),
                        ]),
                    }),
                    AbilityConfiguration::default(),
                ),
                (
                    AbilityNumber(1),
                    Ability::Named(NamedAbility::Reclaim(Some(Energy(4)))),
                    AbilityConfiguration::default(),
                ),
            ])
        }
        CardName::TestModalReturnToHandOrDrawTwo => build_ability_list(
            CardName::TestModalReturnToHandOrDrawTwo.tmp_to_card_identity(),
            vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: None,
                    effect: Effect::Modal(vec![
                        ModalEffectChoice {
                            energy_cost: Energy(2),
                            effect: Effect::Effect(StandardEffect::ReturnToHand {
                                target: Predicate::Enemy(CardPredicate::Character),
                            }),
                        },
                        ModalEffectChoice {
                            energy_cost: Energy(3),
                            effect: Effect::Effect(StandardEffect::DrawCards { count: 2 }),
                        },
                    ]),
                }),
                AbilityConfiguration::default(),
            )],
        ),
    }
}
