use std::sync::OnceLock;

use ability_data::ability::{Ability, EventAbility};
use ability_data::activated_ability::{ActivatedAbility, ActivatedAbilityOptions};
use ability_data::cost::Cost;
use ability_data::effect::{Effect, EffectWithOptions};
use ability_data::named_ability::NamedAbility;
use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::quantity_expression_data::QuantityExpression;
use ability_data::standard_effect::StandardEffect;
use ability_data::static_ability::{PlayFromVoid, StandardStaticAbility};
use ability_data::trigger_event::{PlayerTurn, TriggerEvent};
use ability_data::triggered_ability::TriggeredAbility;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{AbilityId, CardIdType};
use battle_state::battle_cards::ability_list::{
    AbilityConfiguration, AbilityData, AbilityList, AbilityReference, CanPlayRestriction,
};
use battle_state::triggers::trigger::TriggerName;
use core_data::card_types::CardType;
use core_data::identifiers::{AbilityNumber, CardName};
use core_data::numerics::{Energy, Spark};
use enumset::EnumSet;

use crate::battle_card_queries::{build_named_abilities, card};
use crate::card_ability_queries::effect_predicates;
use crate::panic_with;

static CHARACTER_ABILITIES: OnceLock<AbilityList> = OnceLock::new();
static TEST_DISSOLVE_ABILITIES: OnceLock<AbilityList> = OnceLock::new();
static TEST_COUNTERSPELL_UNLESS_PAY: OnceLock<AbilityList> = OnceLock::new();
static COUNTERSPELL_ABILITIES: OnceLock<AbilityList> = OnceLock::new();
static ENERGY_PROMPT_ABILITIES: OnceLock<AbilityList> = OnceLock::new();
static TEST_DRAW_ONE_ABILITIES: OnceLock<AbilityList> = OnceLock::new();
static TEST_TRIGGER_GAIN_SPARK_WHEN_MATERIALIZE_ANOTHER_CHARACTER: OnceLock<AbilityList> =
    OnceLock::new();
static TEST_TRIGGER_GAIN_SPARK_PLAY_OPPONENT_TURN: OnceLock<AbilityList> = OnceLock::new();
static TEST_ACTIVATED_ABILITY_CHARACTER: OnceLock<AbilityList> = OnceLock::new();
static TEST_MULTI_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER: OnceLock<AbilityList> = OnceLock::new();
static TEST_FAST_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER: OnceLock<AbilityList> = OnceLock::new();
static TEST_FAST_MULTI_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER: OnceLock<AbilityList> =
    OnceLock::new();
static TEST_ACTIVATED_ABILITY_DISSOLVE_CHARACTER: OnceLock<AbilityList> = OnceLock::new();
static TEST_DUAL_ACTIVATED_ABILITY_CHARACTER: OnceLock<AbilityList> = OnceLock::new();
static TEST_FORESEE_1: OnceLock<AbilityList> = OnceLock::new();
static TEST_FORESEE_2: OnceLock<AbilityList> = OnceLock::new();
static TEST_FORESEE_1_DRAW_A_CARD: OnceLock<AbilityList> = OnceLock::new();
static TEST_DRAW_ONE_RECLAIM: OnceLock<AbilityList> = OnceLock::new();
static TEST_RETURN_VOID_CARD_TO_HAND: OnceLock<AbilityList> = OnceLock::new();
static TEST_RETURN_ONE_OR_TWO_VOID_EVENT_CARDS_TO_HAND: OnceLock<AbilityList> = OnceLock::new();

pub fn query(battle: &BattleState, card_id: impl CardIdType) -> &'static AbilityList {
    query_by_name(card::get(battle, card_id).name)
}

pub fn query_by_name(name: CardName) -> &'static AbilityList {
    match name {
        CardName::TestVanillaCharacter => CHARACTER_ABILITIES
            .get_or_init(|| build_ability_list(CardName::TestVanillaCharacter, vec![])),
        CardName::TestDissolve => TEST_DISSOLVE_ABILITIES.get_or_init(|| {
            build_ability_list(CardName::TestDissolve, vec![(
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
        }),
        CardName::TestCounterspellUnlessPays => TEST_COUNTERSPELL_UNLESS_PAY.get_or_init(|| {
            build_ability_list(CardName::TestCounterspellUnlessPays, vec![(
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
        }),
        CardName::TestCounterspell => COUNTERSPELL_ABILITIES.get_or_init(|| {
            build_ability_list(CardName::TestCounterspell, vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: None,
                    effect: Effect::Effect(StandardEffect::Counterspell {
                        target: Predicate::Enemy(CardPredicate::CardOnStack),
                    }),
                }),
                AbilityConfiguration {
                    targeting_prompt: Some("Select an enemy card.".to_string()),
                    ..Default::default()
                },
            )])
        }),
        CardName::TestVariableEnergyDraw => ENERGY_PROMPT_ABILITIES.get_or_init(|| {
            build_ability_list(CardName::TestVariableEnergyDraw, vec![(
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
        }),
        CardName::TestDrawOne => TEST_DRAW_ONE_ABILITIES.get_or_init(|| {
            build_ability_list(CardName::TestDrawOne, vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: None,
                    effect: Effect::Effect(StandardEffect::DrawCards { count: 1 }),
                }),
                AbilityConfiguration { ..Default::default() },
            )])
        }),
        CardName::TestTriggerGainSparkWhenMaterializeAnotherCharacter => {
            TEST_TRIGGER_GAIN_SPARK_WHEN_MATERIALIZE_ANOTHER_CHARACTER.get_or_init(|| {
                build_ability_list(
                    CardName::TestTriggerGainSparkWhenMaterializeAnotherCharacter,
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
                )
            })
        }
        CardName::TestTriggerGainSparkOnPlayCardEnemyTurn => {
            TEST_TRIGGER_GAIN_SPARK_PLAY_OPPONENT_TURN.get_or_init(|| {
                build_ability_list(CardName::TestTriggerGainSparkOnPlayCardEnemyTurn, vec![(
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
                )])
            })
        }
        CardName::TestActivatedAbilityDrawCardCharacter => TEST_ACTIVATED_ABILITY_CHARACTER
            .get_or_init(|| {
                build_ability_list(CardName::TestActivatedAbilityDrawCardCharacter, vec![(
                    AbilityNumber(0),
                    Ability::Activated(ActivatedAbility {
                        costs: vec![Cost::Energy(Energy(1))],
                        effect: Effect::Effect(StandardEffect::DrawCards { count: 1 }),
                        options: None,
                    }),
                    AbilityConfiguration::default(),
                )])
            }),
        CardName::TestMultiActivatedAbilityDrawCardCharacter => {
            TEST_MULTI_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER.get_or_init(|| {
                build_ability_list(CardName::TestMultiActivatedAbilityDrawCardCharacter, vec![(
                    AbilityNumber(0),
                    Ability::Activated(ActivatedAbility {
                        costs: vec![Cost::Energy(Energy(1))],
                        effect: Effect::Effect(StandardEffect::DrawCards { count: 1 }),
                        options: Some(ActivatedAbilityOptions { is_multi: true, is_fast: false }),
                    }),
                    AbilityConfiguration::default(),
                )])
            })
        }
        CardName::TestFastActivatedAbilityDrawCardCharacter => {
            TEST_FAST_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER.get_or_init(|| {
                build_ability_list(CardName::TestFastActivatedAbilityDrawCardCharacter, vec![(
                    AbilityNumber(0),
                    Ability::Activated(ActivatedAbility {
                        costs: vec![Cost::Energy(Energy(1))],
                        effect: Effect::Effect(StandardEffect::DrawCards { count: 1 }),
                        options: Some(ActivatedAbilityOptions { is_multi: false, is_fast: true }),
                    }),
                    AbilityConfiguration::default(),
                )])
            })
        }
        CardName::TestFastMultiActivatedAbilityDrawCardCharacter => {
            TEST_FAST_MULTI_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER.get_or_init(|| {
                build_ability_list(CardName::TestFastMultiActivatedAbilityDrawCardCharacter, vec![
                    (
                        AbilityNumber(0),
                        Ability::Activated(ActivatedAbility {
                            costs: vec![Cost::Energy(Energy(1))],
                            effect: Effect::Effect(StandardEffect::DrawCards { count: 1 }),
                            options: Some(ActivatedAbilityOptions {
                                is_multi: true,
                                is_fast: true,
                            }),
                        }),
                        AbilityConfiguration::default(),
                    ),
                ])
            })
        }
        CardName::TestActivatedAbilityDissolveCharacter => {
            TEST_ACTIVATED_ABILITY_DISSOLVE_CHARACTER.get_or_init(|| {
                build_ability_list(CardName::TestActivatedAbilityDissolveCharacter, vec![(
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
                )])
            })
        }
        CardName::TestDualActivatedAbilityCharacter => TEST_DUAL_ACTIVATED_ABILITY_CHARACTER
            .get_or_init(|| {
                build_ability_list(CardName::TestDualActivatedAbilityCharacter, vec![
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
                ])
            }),
        CardName::TestForeseeOne => TEST_FORESEE_1.get_or_init(|| {
            build_ability_list(CardName::TestForeseeOne, vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: None,
                    effect: Effect::Effect(StandardEffect::Foresee { count: 1 }),
                }),
                AbilityConfiguration::default(),
            )])
        }),
        CardName::TestForeseeTwo => TEST_FORESEE_2.get_or_init(|| {
            build_ability_list(CardName::TestForeseeTwo, vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: None,
                    effect: Effect::Effect(StandardEffect::Foresee { count: 2 }),
                }),
                AbilityConfiguration::default(),
            )])
        }),
        CardName::TestForeseeOneDrawACard => TEST_FORESEE_1_DRAW_A_CARD.get_or_init(|| {
            build_ability_list(CardName::TestForeseeOneDrawACard, vec![(
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
        }),
        CardName::TestDrawOneReclaim => TEST_DRAW_ONE_RECLAIM.get_or_init(|| {
            build_ability_list(CardName::TestDrawOneReclaim, vec![
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
        }),
        CardName::TestReturnVoidCardToHand => TEST_RETURN_VOID_CARD_TO_HAND.get_or_init(|| {
            build_ability_list(CardName::TestReturnVoidCardToHand, vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: None,
                    effect: Effect::Effect(StandardEffect::ReturnFromYourVoidToHand {
                        target: Predicate::YourVoid(CardPredicate::Card),
                    }),
                }),
                AbilityConfiguration::default(),
            )])
        }),
        CardName::TestReturnOneOrTwoVoidEventCardsToHand => {
            TEST_RETURN_ONE_OR_TWO_VOID_EVENT_CARDS_TO_HAND.get_or_init(|| {
                build_ability_list(CardName::TestReturnOneOrTwoVoidEventCardsToHand, vec![(
                    AbilityNumber(0),
                    Ability::Event(EventAbility {
                        additional_cost: None,
                        effect: Effect::Effect(StandardEffect::ReturnUpToCountForYourVoidToHand {
                            target: Predicate::YourVoid(CardPredicate::Event),
                            count: 2,
                        }),
                    }),
                    AbilityConfiguration::default(),
                )])
            })
        }
    }
}

/// Returns a reference to an ability based on its ID.
///
/// Panics if the ability is not found.
pub fn ability(battle: &BattleState, ability_id: AbilityId) -> AbilityReference<'static> {
    let ability_list = query(battle, ability_id.card_id);
    ability_list
        .event_abilities
        .iter()
        .find(|a| a.ability_number == ability_id.ability_number)
        .map(|a| AbilityReference::Event(&a.ability))
        .or_else(|| {
            ability_list
                .static_abilities
                .iter()
                .find(|a| a.ability_number == ability_id.ability_number)
                .map(|a| AbilityReference::Static(&a.ability))
        })
        .or_else(|| {
            ability_list
                .activated_abilities
                .iter()
                .find(|a| a.ability_number == ability_id.ability_number)
                .map(|a| AbilityReference::Activated(&a.ability))
        })
        .or_else(|| {
            ability_list
                .triggered_abilities
                .iter()
                .find(|a| a.ability_number == ability_id.ability_number)
                .map(|a| AbilityReference::Triggered(&a.ability))
        })
        .unwrap_or_else(|| panic_with!("Ability not found", battle, ability_id))
}

fn build_ability_list(
    card_name: CardName,
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
                    card_name,
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
    let predicates: Vec<&Predicate> = list
        .event_abilities
        .iter()
        .flat_map(|data| match &data.ability.effect {
            Effect::Effect(effect) => vec![
                effect_predicates::get_character_target_predicate(effect),
                effect_predicates::get_stack_target_predicate(effect),
                effect_predicates::get_void_target_predicate(effect),
            ],
            Effect::WithOptions(options) => vec![
                effect_predicates::get_character_target_predicate(&options.effect),
                effect_predicates::get_stack_target_predicate(&options.effect),
                effect_predicates::get_void_target_predicate(&options.effect),
            ],
            Effect::List(effects) => effects
                .iter()
                .flat_map(|effect| {
                    vec![
                        effect_predicates::get_character_target_predicate(&effect.effect),
                        effect_predicates::get_stack_target_predicate(&effect.effect),
                        effect_predicates::get_void_target_predicate(&effect.effect),
                    ]
                })
                .collect(),
        })
        .flatten()
        .collect();

    let predicate = match predicates[..] {
        [] => {
            return Some(CanPlayRestriction::Unrestricted);
        }
        [predicate] => predicate,
        [..] => {
            return None;
        }
    };

    match predicate {
        Predicate::Enemy(CardPredicate::Character) => Some(CanPlayRestriction::EnemyCharacter),
        Predicate::Enemy(CardPredicate::CardOnStack) => Some(CanPlayRestriction::EnemyStackCard),
        Predicate::Enemy(CardPredicate::Event) => {
            Some(CanPlayRestriction::EnemyStackCardOfType(CardType::Event))
        }
        _ => None,
    }
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
        TriggerEvent::Play(..) => TriggerName::PlayedCardFromHand,
        TriggerEvent::PlayDuringTurn(..) => TriggerName::PlayedCardFromHand,
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
