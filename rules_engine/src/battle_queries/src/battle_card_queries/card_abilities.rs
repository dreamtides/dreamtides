use std::sync::OnceLock;

use ability_data::ability::{Ability, EventAbility};
use ability_data::cost::Cost;
use ability_data::effect::Effect;
use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::quantity_expression_data::QuantityExpression;
use ability_data::standard_effect::StandardEffect;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardIdType;
use core_data::identifiers::{AbilityNumber, CardName};
use core_data::numerics::Energy;

static MINSTREL_ABILITIES: OnceLock<Vec<(AbilityNumber, Ability)>> = OnceLock::new();
static IMMOLATE_ABILITIES: OnceLock<Vec<(AbilityNumber, Ability)>> = OnceLock::new();
static RIPPLE_ABILITIES: OnceLock<Vec<(AbilityNumber, Ability)>> = OnceLock::new();
static ABOLISH_ABILITIES: OnceLock<Vec<(AbilityNumber, Ability)>> = OnceLock::new();
static DREAMSCATTER_ABILITIES: OnceLock<Vec<(AbilityNumber, Ability)>> = OnceLock::new();

pub fn query(
    battle: &BattleState,
    card_id: impl CardIdType,
) -> &'static [(AbilityNumber, Ability)] {
    match battle.cards.name(card_id) {
        CardName::MinstrelOfFallingLight => MINSTREL_ABILITIES.get_or_init(|| vec![]),
        CardName::Immolate => IMMOLATE_ABILITIES.get_or_init(|| {
            vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: None,
                    effect: Effect::Effect(StandardEffect::DissolveCharacter {
                        target: Predicate::Enemy(CardPredicate::Character),
                    }),
                }),
            )]
        }),
        CardName::RippleOfDefiance => RIPPLE_ABILITIES.get_or_init(|| {
            vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: None,
                    effect: Effect::Effect(StandardEffect::NegateUnlessPaysCost {
                        target: Predicate::Enemy(CardPredicate::Event),
                        cost: Cost::Energy(Energy(2)),
                    }),
                }),
            )]
        }),
        CardName::Abolish => ABOLISH_ABILITIES.get_or_init(|| {
            vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: None,
                    effect: Effect::Effect(StandardEffect::Negate {
                        target: Predicate::Enemy(CardPredicate::Dream),
                    }),
                }),
            )]
        }),
        CardName::Dreamscatter => DREAMSCATTER_ABILITIES.get_or_init(|| {
            vec![(
                AbilityNumber(0),
                Ability::Event(EventAbility {
                    additional_cost: Some(Cost::SpendOneOrMoreEnergy),
                    effect: Effect::Effect(StandardEffect::DrawCardsForEach {
                        count: 1,
                        for_each: QuantityExpression::ForEachEnergySpentOnThisCard,
                    }),
                }),
            )]
        }),
    }
}
