use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DebugCardState {
    pub id: String,
    pub object_id: String,
    pub controller: String,
    pub current_zone: String,
    pub properties: DebugCardProperties,
    pub abilities: Vec<String>,
    pub stack_state: DebugStackCardState,
}

#[derive(Debug, Clone, Serialize)]
pub struct DebugCardProperties {
    pub card_type: String,
    pub spark: String,
    pub cost: String,
    pub is_fast: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DebugStackCardState {
    pub has_stack_card_state: bool,
    pub id: String,
    pub controller: String,
    pub targets: String,
    pub additional_costs_paid: String,
}
