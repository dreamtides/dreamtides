use core_data::display_types::{
    AudioClipAddress, EffectAddress, MaterialAddress, Milliseconds, ProjectileAddress,
};
use core_data::identifiers::BaseCardId;
use serde::{Deserialize, Serialize};

use crate::tabula_primitives::TabulaValue;

/// Represents a custom effect applied by a card.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardEffectRow {
    pub card_id: BaseCardId,

    pub effect_type: CardEffectRowType,

    pub effect_trigger: CardEffectRowTrigger,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub projectile_source: Option<CardEffectRowObjectPredicate>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub projectile_target: Option<CardEffectRowObjectPredicate>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub projectile_address: Option<TabulaValue<ProjectileAddress>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub projectile_fire_sound: Option<TabulaValue<AudioClipAddress>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub projectile_impact_sound: Option<TabulaValue<AudioClipAddress>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dissolve_material: Option<TabulaValue<MaterialAddress>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dissolve_color: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dissolve_sound: Option<TabulaValue<AudioClipAddress>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect_target: Option<CardEffectRowObjectPredicate>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect_address: Option<TabulaValue<EffectAddress>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect_duration_milliseconds: Option<TabulaValue<Milliseconds>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect_scale: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect_sound: Option<TabulaValue<AudioClipAddress>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub card_trail_targets: Option<CardEffectRowObjectPredicate>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub card_trail_address: Option<TabulaValue<ProjectileAddress>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_duration_milliseconds: Option<TabulaValue<Milliseconds>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CardEffectRowType {
    FireProjectile,
    DissolveTargets,
    ReverseDissolveTargets,
    DisplayEffect,
    SetCardTrail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CardEffectRowTrigger {
    ApplyTargetedEffect,
    DrawCards,
    ActivatedAbility,
    SelectedTargetsForCard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CardEffectRowObjectPredicate {
    ThisCard,
    ForEachTarget,
    ControllerDeck,
}
