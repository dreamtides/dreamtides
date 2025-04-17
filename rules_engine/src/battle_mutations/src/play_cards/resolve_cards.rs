use ability_data::ability::Ability;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle_cards::card_id::StackCardId;
use core_data::card_types::CardType;
use core_data::effect_source::EffectSource;

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
///
/// Returns the [ObjectId] of the card in its new zone, or None if the card
/// failed to resolve, e.g. because it no longer exists.
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

    for effect in effects {
        apply_effect::apply(battle, source, effect, battle.cards.card(card_id)?.targets.clone());
    }
    Some(())
}
