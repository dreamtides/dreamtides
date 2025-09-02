use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DebugDreamwellCardState {
    pub index: String,
    pub name: String,
    pub phase: String,
    pub produced_energy: String,
    pub abilities: Vec<String>,
    pub is_active: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DebugDreamwellState {
    pub next_index: String,
    pub first_iteration_complete: String,
    pub active_card: String,
    pub cards: Vec<DebugDreamwellCardState>,
}
