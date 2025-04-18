use ability_data::ability::Ability;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle::effect_source::EffectSource;
use battle_data::battle_cards::card_id::StackCardId;
use core_data::card_types::CardType;
use core_data::identifiers::AbilityNumber;

use crate::effects::apply_effect;
use crate::zone_mutations::move_card;

/// Resolves all cards currently on the stack, applying their effects.
pub fn resolve_stack(battle: &mut BattleData, source: EffectSource) {
    let stack_cards = battle.cards.stack().to_vec();
    for card_id in stack_cards {
        resolve_card(battle, source, card_id);
    }
}

/// Resolves a card currently on the stack, applying its effects and moving it
/// to the appropriate zone.
fn resolve_card(battle: &mut BattleData, source: EffectSource, card_id: StackCardId) -> Option<()> {
    if battle.cards.card(card_id)?.properties.card_type == CardType::Event {
        apply_event_effects(battle, source, card_id);
    }

    match battle.cards.card(card_id)?.properties.card_type {
        CardType::Character(_) => {
            move_card::to_battlefield(battle, source, card_id);
        }
        _ => {
            move_card::to_void(battle, source, card_id);
        }
    }

    Some(())
}

fn apply_event_effects(
    battle: &mut BattleData,
    source: EffectSource,
    card_id: StackCardId,
) -> Option<()> {
    let effects = battle
        .cards
        .card(card_id)?
        .abilities
        .iter()
        .filter_map(|ability| match ability {
            Ability::Event(effect) => Some(effect),
            _ => None,
        })
        .cloned()
        .collect::<Vec<_>>();

    for (i, effect) in effects.into_iter().enumerate() {
        let event_source = EffectSource::Event {
            controller: source.controller(),
            card: card_id,
            ability_number: AbilityNumber(i),
        };
        apply_effect::apply(
            battle,
            event_source,
            effect,
            battle.cards.card(card_id)?.targets.clone(),
        );
    }
    Some(())
}
