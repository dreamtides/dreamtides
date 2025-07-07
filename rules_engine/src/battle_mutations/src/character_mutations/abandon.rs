use battle_queries::battle_trace;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CharacterId, VoidCardId};
use battle_state::core::effect_source::EffectSource;
use battle_state::triggers::trigger::Trigger;

use crate::character_mutations::dissolve;

/// Dissolves a character, moving it to the void.
///
/// Panics if the character is not on the battlefield.
pub fn apply(battle: &mut BattleState, source: EffectSource, id: CharacterId) -> VoidCardId {
    battle_trace!("Abandoning character", battle, id);
    let id = dissolve::execute(battle, source, id);
    battle.triggers.push(source, Trigger::Dissolved(id));
    id
}
