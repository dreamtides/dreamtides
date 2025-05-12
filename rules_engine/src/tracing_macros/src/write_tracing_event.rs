use battle_queries::debug_snapshot::debug_battle_snapshot;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle_trace::battle_tracing::BattleTraceEvent;
use serde_json;

pub fn write_battle_event(battle: &mut BattleState, event: BattleTraceEvent) {
    todo!("")
}

pub fn write_panic_snapshot(battle: &BattleState) {
    let snapshot = debug_battle_snapshot::capture(battle);
    eprintln!(
        "\n\n\n>>>>>>>>>>>>>>>>>>>>\n{}\n<<<<<<<<<<<<<<<<<<<<\n\n\n",
        serde_json::to_string_pretty(&snapshot).unwrap_or_else(|_| format!("{:?}", snapshot))
    );
}
