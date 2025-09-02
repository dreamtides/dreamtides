use core_data::display_types::{
    AudioClipAddress, EffectAddress, MaterialAddress, Milliseconds, ProjectileAddress,
};
use core_data::identifiers::BaseCardId;

/// Represents a custom effect applied by a card.
pub struct CardEffectRow {
    pub card_id: BaseCardId,
    pub effect_type: CardEffectRowType,
    pub effect_trigger: CardEffectRowTrigger,
    pub projectile_source: Option<CardEffectRowObjectPredicate>,
    pub projectile_target: Option<CardEffectRowObjectPredicate>,
    pub projectile_address: Option<ProjectileAddress>,
    pub projectile_fire_sound: Option<AudioClipAddress>,
    pub projectile_impact_sound: Option<AudioClipAddress>,
    pub dissolve_material: Option<MaterialAddress>,
    pub dissolve_color: Option<String>,
    pub dissolve_sound: Option<AudioClipAddress>,
    pub effect_target: Option<CardEffectRowObjectPredicate>,
    pub effect_address: Option<EffectAddress>,
    pub effect_duration_milliseconds: Option<Milliseconds>,
    pub effect_scale: Option<f64>,
    pub effect_sound: Option<AudioClipAddress>,
    pub card_trail_targets: Option<CardEffectRowObjectPredicate>,
    pub card_trail_address: Option<ProjectileAddress>,
}

pub enum CardEffectRowType {
    FireProjectile,
    DissolveTargets,
    ReverseDissolveTargets,
    DisplayEffect,
    SetCardTrail,
}

pub enum CardEffectRowTrigger {
    ApplyTargetedEffect,
    DrawCards,
    ActivatedAbility,
    SelectedTargetsForCard,
}

pub enum CardEffectRowObjectPredicate {
    ThisCard,
    ForEachTarget,
    ControllerDeck,
}
