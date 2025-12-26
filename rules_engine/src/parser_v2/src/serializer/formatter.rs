use ability_data::standard_effect::StandardEffect;

pub fn serialize_standard_effect(effect: &StandardEffect) -> String {
    match effect {
        StandardEffect::DrawCards { .. } => "Draw {cards}.".to_string(),
        StandardEffect::DiscardCards { .. } => "Discard {cards}.".to_string(),
        StandardEffect::GainEnergy { .. } => "Gain {e}.".to_string(),
        _ => unimplemented!("Serialization not yet implemented for this effect type"),
    }
}
