use ability_data::ability::Ability;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle::effect_source::EffectSource;
use battle_data::battle_cards::card_id::StackCardId;
use battle_data::prompt_types::prompt_data::PromptResumeAction;
use core_data::card_types::CardType;
use core_data::identifiers::AbilityNumber;
use logging::battle_trace;

use crate::effects::apply_effect;
use crate::zone_mutations::move_card;

/// Resolves all cards currently on the stack, applying their effects.
///
/// Cards resolve in Last In, First Out order, meaning the top card of the stack
/// is resolved first.
pub fn resolve_stack(battle: &mut BattleData) {
    while let Some(card_id) = battle.cards.stack().last() {
        resolve_card(battle, *card_id);

        // Stop resolving the stack if a prompt is pending.
        if battle.prompt.is_some() {
            battle.prompt_resume_action = Some(PromptResumeAction::ResolveStack);
            break;
        }
    }
}

/// Resolves a card currently on the stack, applying its effects and moving it
/// to the appropriate zone.
fn resolve_card(battle: &mut BattleData, card_id: StackCardId) -> Option<()> {
    battle_trace!("Resolving card", battle, card_id);
    if battle.cards.card(card_id)?.properties.card_type == CardType::Event {
        apply_event_effects(battle, card_id);
    }

    battle.cards.card_mut(card_id)?.targets.clear();
    let source = EffectSource::Game { controller: battle.turn.active_player };
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

fn apply_event_effects(battle: &mut BattleData, card_id: StackCardId) -> Option<()> {
    let card = battle.cards.card(card_id)?;
    let controller = card.controller();
    let effects = card
        .abilities
        .iter()
        .filter_map(|ability| match ability {
            Ability::Event(effect) => Some(effect),
            _ => None,
        })
        .cloned()
        .collect::<Vec<_>>();

    for (i, effect) in effects.into_iter().enumerate() {
        let event_source =
            EffectSource::Event { controller, card: card_id, ability_number: AbilityNumber(i) };
        apply_effect::apply(
            battle,
            event_source,
            effect,
            battle.cards.card(card_id)?.targets.clone(),
        );
    }
    Some(())
}
