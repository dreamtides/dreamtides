use std::collections::BTreeMap;

use core_data::numerics::TurnId;

use crate::debug_snapshots::debug_battle_data::DebugBattleData;

/// Data structure for storing debug traces of events during a turn of a battle.
#[derive(Debug, Clone, Default)]
pub struct BattleTracing {
    /// The turn for which we are storing tracing data
    pub turn: TurnId,

    /// Events from the turn in question
    pub current: Vec<BattleTraceEvent>,
}

#[derive(Debug, Clone)]
pub struct BattleTraceEvent {
    /// Description of event
    pub message: String,

    /// Map from symbol names to symbol values for relevant symbols at the time
    /// this trace event was captured
    pub values: BTreeMap<String, String>,

    /// String representation of the values
    pub values_string: String,

    /// Snapshot of the battle state at the time this trace event was captured
    pub snapshot: DebugBattleData,
}
