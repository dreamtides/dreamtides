use std::sync::OnceLock;

use ability_data::ability::{Ability, EventAbility};
use ability_data::cost::Cost;
use ability_data::effect::Effect;
use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::quantity_expression_data::QuantityExpression;
use ability_data::standard_effect::StandardEffect;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardIdType;
use battle_state::battle_cards::ability_list::AbilityList;
use core_data::identifiers::{AbilityNumber, CardName};
use core_data::numerics::Energy;

static MINSTREL_ABILITIES: OnceLock<AbilityList> = OnceLock::new();
static IMMOLATE_ABILITIES: OnceLock<AbilityList> = OnceLock::new();
static RIPPLE_ABILITIES: OnceLock<AbilityList> = OnceLock::new();
static ABOLISH_ABILITIES: OnceLock<AbilityList> = OnceLock::new();
static DREAMSCATTER_ABILITIES: OnceLock<AbilityList> = OnceLock::new();

pub fn query(battle: &BattleState, card_id: impl CardIdType) -> &'static AbilityList {
    match battle.cards.name(card_id) {
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
                ability_list.event_abilities.push((ability_number, event_ability.clone()));
            }
            Ability::Static(static_ability) => {
                ability_list.static_abilities.push((ability_number, static_ability.clone()));
            }
            Ability::Activated(activated_ability) => {
                ability_list.activated_abilities.push((ability_number, activated_ability.clone()));
            }
            Ability::Triggered(triggered_ability) => {
                ability_list.triggered_abilities.push((ability_number, triggered_ability.clone()));
            }
        }
    }

    ability_list
}
