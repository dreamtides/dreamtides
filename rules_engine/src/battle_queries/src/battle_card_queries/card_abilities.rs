use std::sync::OnceLock;

use ability_data::ability::{Ability, EventAbility};
use ability_data::cost::Cost;
use ability_data::effect::Effect;
use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::quantity_expression_data::QuantityExpression;
use ability_data::standard_effect::StandardEffect;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardIdType;
use battle_state::battle_cards::ability_list::{AbilityData, AbilityList, CanPlayRestriction};
use core_data::card_types::CardType;
use core_data::identifiers::{AbilityNumber, CardName};
use core_data::numerics::Energy;

use crate::card_ability_queries::effect_predicates;

static MINSTREL_ABILITIES: OnceLock<AbilityList> = OnceLock::new();
static IMMOLATE_ABILITIES: OnceLock<AbilityList> = OnceLock::new();
static RIPPLE_ABILITIES: OnceLock<AbilityList> = OnceLock::new();
static ABOLISH_ABILITIES: OnceLock<AbilityList> = OnceLock::new();
static DREAMSCATTER_ABILITIES: OnceLock<AbilityList> = OnceLock::new();

pub fn query(battle: &BattleState, card_id: impl CardIdType) -> &'static AbilityList {
    query_by_name(battle.cards.card(card_id).name)
}

pub fn query_by_name(name: CardName) -> &'static AbilityList {
    match name {
        CardName::MinstrelOfFallingLight => {
            MINSTREL_ABILITIES.get_or_init(|| build_ability_list(vec![]))
        }
        CardName::Immolate => IMMOLATE_ABILITIES.get_or_init(|| {
            build_ability_list(vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: None,
                    effect: Effect::Effect(StandardEffect::DissolveCharacter {
                        target: Predicate::Enemy(CardPredicate::Character),
                    }),
                }),
            )])
        }),
        CardName::RippleOfDefiance => RIPPLE_ABILITIES.get_or_init(|| {
            build_ability_list(vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: None,
                    effect: Effect::Effect(StandardEffect::NegateUnlessPaysCost {
                        target: Predicate::Enemy(CardPredicate::Event),
                        cost: Cost::Energy(Energy(2)),
                    }),
                }),
            )])
        }),
        CardName::Abolish => ABOLISH_ABILITIES.get_or_init(|| {
            build_ability_list(vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: None,
                    effect: Effect::Effect(StandardEffect::Negate {
                        target: Predicate::Enemy(CardPredicate::Dream),
                    }),
                }),
            )])
        }),
        CardName::Dreamscatter => DREAMSCATTER_ABILITIES.get_or_init(|| {
            build_ability_list(vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: Some(Cost::SpendOneOrMoreEnergy),
                    effect: Effect::Effect(StandardEffect::DrawCardsForEach {
                        count: 1,
                        for_each: QuantityExpression::ForEachEnergySpentOnThisCard,
                    }),
                }),
            )])
        }),
    }
}

fn build_ability_list(abilities: Vec<(AbilityNumber, Ability)>) -> AbilityList {
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
        }
    }

    ability_list.can_play_restriction = merge_can_play_restrictions(vec![
        compute_event_target_restriction(&ability_list),
        compute_event_additional_cost_restriction(&ability_list),
    ]);

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
            ],
            Effect::WithOptions(options) => vec![
                effect_predicates::get_character_target_predicate(&options.effect),
                effect_predicates::get_stack_target_predicate(&options.effect),
            ],
            Effect::List(effects) => effects
                .iter()
                .flat_map(|effect| {
                    vec![
                        effect_predicates::get_character_target_predicate(&effect.effect),
                        effect_predicates::get_stack_target_predicate(&effect.effect),
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
        Predicate::Enemy(CardPredicate::Dream) => Some(CanPlayRestriction::EnemyStackCard),
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
