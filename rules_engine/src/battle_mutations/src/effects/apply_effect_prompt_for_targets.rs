use ability_data::effect::Effect;
use battle_queries::battle_card_queries::automatic_effect_targets::{self, AutomaticEffectTargets};
use battle_state::battle::battle_state::{BattleState, PendingEffect, PendingEffectIndex};
use battle_state::battle::card_id::CardId;
use battle_state::core::effect_source::EffectSource;
use battle_state::prompt_types::prompt_data::OnSelected;

use crate::effects::apply_effect;
use crate::prompt_mutations::targeting_prompt;

/// Applies an effect to the [BattleState], prompting for effect targets if
/// required.
///
/// This is used when resolving triggered effects, 'if you do' effects, and
/// similar actions which happen on resolution. It is NOT used for playing cards
/// or activating abilities, which have their targets selected before adding
/// them to the stack.
///
/// The predicate "This" will be interpreted by this function as being the card
/// associated with the provided [EffectSource]. The predicate "That" will be
/// applied based on the `that_card` parameter to this function.
pub fn execute(
    battle: &mut BattleState,
    source: EffectSource,
    effect: &Effect,
    that_card: Option<CardId>,
) {
    let automatic_targets = automatic_effect_targets::query(battle, source, effect, that_card);
    match automatic_targets {
        AutomaticEffectTargets::RequiresPrompt => {
            let pending_effect_index = PendingEffectIndex(battle.pending_effects.len());
            let mut prompts = targeting_prompt::targeting_prompts(
                battle,
                source.controller(),
                source,
                effect,
                that_card,
                OnSelected::AddPendingEffectTarget(pending_effect_index),
            );
            if prompts.is_empty() {
                apply_effect::execute(battle, source, effect, None, None);
            } else {
                battle.prompts.append(&mut prompts);
                battle.pending_effects.push_back(PendingEffect {
                    source,
                    effect: effect.clone(),
                    requested_targets: None,
                    modal_choice: None,
                });
            }
        }
        AutomaticEffectTargets::Targets(targets) => {
            apply_effect::execute(battle, source, effect, targets.as_ref(), None);
        }
    }
}
