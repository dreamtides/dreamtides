use ability_data::trigger_event::TriggerEvent;
use battle_state::battle::battle_state::BattleState;
use battle_state::triggers::trigger::Trigger;

/// Returns true if the predicates in a [TriggerEvent] match for the given
/// [Trigger].
pub fn matches(_battle: &BattleState, _trigger: Trigger, _event: TriggerEvent) -> bool {
    true
}
