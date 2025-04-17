use crate::battle_cards::card_data::CardData;

pub struct DebugCardData {
    pub id: String,
    pub owner: String,
    pub zone: String,
    pub object_id: String,
    pub properties: DebugCardProperties,
    pub abilities: Vec<String>,
    pub revealed_to_owner: String,
    pub revealed_to_opponent: String,
    pub targets: Vec<String>,
    pub turn_entered_current_zone: String,
}

impl DebugCardData {
    pub fn new(card_data: CardData) -> Self {
        Self {
            id: format!("{:?}", card_data.id),
            owner: format!("{:?}", card_data.owner),
            zone: format!("{:?}", card_data.zone),
            object_id: format!("ObjectId({})", card_data.object_id.0),
            properties: DebugCardProperties {
                card_type: format!("{:?}", card_data.properties.card_type),
                spark: format!("{:?}", card_data.properties.spark),
                cost: format!("{:?}", card_data.properties.cost),
                is_fast: format!("{}", card_data.properties.is_fast),
            },
            abilities: card_data.abilities.iter().map(|ability| format!("{:?}", ability)).collect(),
            revealed_to_owner: format!("{}", card_data.revealed_to_owner),
            revealed_to_opponent: format!("{}", card_data.revealed_to_opponent),
            targets: card_data.targets.iter().map(|target| format!("{:?}", target)).collect(),
            turn_entered_current_zone: format!(
                "TurnData {{ active_player: {:?}, turn_id: TurnId({}) }}",
                card_data.turn_entered_current_zone.active_player,
                card_data.turn_entered_current_zone.turn_id.0
            ),
        }
    }
}

pub struct DebugCardProperties {
    pub card_type: String,
    pub spark: String,
    pub cost: String,
    pub is_fast: String,
}
