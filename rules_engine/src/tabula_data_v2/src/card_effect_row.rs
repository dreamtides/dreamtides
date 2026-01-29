use std::path::Path;

use core_data::display_color::DisplayColor;
use core_data::display_types::{
    AudioClipAddress, EffectAddress, MaterialAddress, Milliseconds, ProjectileAddress,
    StringWrapper,
};
use core_data::identifiers::BaseCardId;
use serde::{Deserialize, Serialize};
use strum::EnumString;
use uuid::Uuid;

use crate::tabula_error::TabulaError;
use crate::toml_loader::CardEffectRowRaw;

/// The type of visual effect to apply for a card.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString)]
pub enum CardEffectRowType {
    /// Fire a projectile from source to target.
    FireProjectile,
    /// Apply a dissolve shader effect to targets.
    DissolveTargets,
    /// Apply a reverse dissolve shader effect to targets.
    ReverseDissolveTargets,
    /// Display a visual effect at a location.
    DisplayEffect,
    /// Set a trail effect on a card.
    SetCardTrail,
}

/// The game event that triggers a card effect.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString)]
pub enum CardEffectRowTrigger {
    /// Triggered when a targeted effect is applied.
    ApplyTargetedEffect,
    /// Triggered when cards are drawn.
    DrawCards,
    /// Triggered when an activated ability is used.
    ActivatedAbility,
    /// Triggered when targets are selected for a card.
    SelectedTargetsForCard,
}

/// Predicate for selecting game objects as effect sources or targets.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString)]
pub enum CardEffectRowObjectPredicate {
    /// The card that owns this effect.
    ThisCard,
    /// Each target of the card's effect.
    ForEachTarget,
    /// The controlling player's deck.
    ControllerDeck,
}

/// A row from the card effects table defining visual effects for cards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardEffectRow {
    /// The card this effect applies to.
    pub card_id: BaseCardId,
    /// The type of visual effect.
    pub effect_type: CardEffectRowType,
    /// The game event that triggers this effect.
    pub effect_trigger: CardEffectRowTrigger,
    /// Source location for projectile effects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projectile_source: Option<CardEffectRowObjectPredicate>,
    /// Target location for projectile effects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projectile_target: Option<CardEffectRowObjectPredicate>,
    /// Asset path for the projectile prefab.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projectile_address: Option<ProjectileAddress>,
    /// Sound to play when firing the projectile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projectile_fire_sound: Option<AudioClipAddress>,
    /// Sound to play on projectile impact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projectile_impact_sound: Option<AudioClipAddress>,
    /// Material for dissolve shader effects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dissolve_material: Option<MaterialAddress>,
    /// Color for dissolve shader effects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dissolve_color: Option<DisplayColor>,
    /// Sound for dissolve effects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dissolve_sound: Option<AudioClipAddress>,
    /// Target location for display effects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect_target: Option<CardEffectRowObjectPredicate>,
    /// Asset path for display effect prefab.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect_address: Option<EffectAddress>,
    /// Duration of the display effect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect_duration_milliseconds: Option<Milliseconds>,
    /// Scale multiplier for display effects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect_scale: Option<f64>,
    /// Sound for display effects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect_sound: Option<AudioClipAddress>,
    /// Target objects for card trail effects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card_trail_targets: Option<CardEffectRowObjectPredicate>,
    /// Asset path for card trail prefab.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card_trail_address: Option<ProjectileAddress>,
    /// Duration of the card trail effect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_duration_milliseconds: Option<Milliseconds>,
}

/// Builds a [CardEffectRow] from raw TOML data.
pub fn build_card_effect_row(
    raw: &CardEffectRowRaw,
    file: &Path,
) -> Result<CardEffectRow, TabulaError> {
    let card_id = parse_uuid(&raw.card_id, file, "card_id")?;
    let effect_type = parse_effect_type(&raw.effect_type, file, card_id)?;
    let effect_trigger = parse_effect_trigger(&raw.effect_trigger, file, card_id)?;

    Ok(CardEffectRow {
        card_id: BaseCardId(card_id),
        effect_type,
        effect_trigger,
        projectile_source: parse_object_predicate_opt(&raw.projectile_source, file, card_id)?,
        projectile_target: parse_object_predicate_opt(&raw.projectile_target, file, card_id)?,
        projectile_address: parse_string_wrapper_opt(&raw.projectile_address),
        projectile_fire_sound: parse_string_wrapper_opt(&raw.projectile_fire_sound),
        projectile_impact_sound: parse_string_wrapper_opt(&raw.projectile_impact_sound),
        dissolve_material: parse_string_wrapper_opt(&raw.dissolve_material),
        dissolve_color: parse_display_color_opt(&raw.dissolve_color, file, card_id)?,
        dissolve_sound: parse_string_wrapper_opt(&raw.dissolve_sound),
        effect_target: parse_object_predicate_opt(&raw.effect_target, file, card_id)?,
        effect_address: parse_string_wrapper_opt(&raw.effect_address),
        effect_duration_milliseconds: raw
            .effect_duration_milliseconds
            .map(|ms| Milliseconds::new(ms as u32)),
        effect_scale: raw.effect_scale,
        effect_sound: parse_string_wrapper_opt(&raw.effect_sound),
        card_trail_targets: parse_object_predicate_opt(&raw.card_trail_targets, file, card_id)?,
        card_trail_address: parse_string_wrapper_opt(&raw.card_trail_address),
        trail_duration_milliseconds: raw
            .trail_duration_milliseconds
            .map(|ms| Milliseconds::new(ms as u32)),
    })
}

fn parse_uuid(s: &str, file: &Path, field: &'static str) -> Result<Uuid, TabulaError> {
    Uuid::parse_str(s).map_err(|e| TabulaError::InvalidField {
        file: file.to_path_buf(),
        card_id: None,
        field,
        message: e.to_string(),
    })
}

fn parse_effect_type(
    s: &str,
    file: &Path,
    card_id: Uuid,
) -> Result<CardEffectRowType, TabulaError> {
    CardEffectRowType::try_from(s).map_err(|_| TabulaError::InvalidField {
        file: file.to_path_buf(),
        card_id: Some(card_id),
        field: "effect-type",
        message: format!("unknown effect type '{s}'"),
    })
}

fn parse_effect_trigger(
    s: &str,
    file: &Path,
    card_id: Uuid,
) -> Result<CardEffectRowTrigger, TabulaError> {
    CardEffectRowTrigger::try_from(s).map_err(|_| TabulaError::InvalidField {
        file: file.to_path_buf(),
        card_id: Some(card_id),
        field: "effect-trigger",
        message: format!("unknown effect trigger '{s}'"),
    })
}

fn parse_object_predicate(
    s: &str,
    file: &Path,
    card_id: Uuid,
) -> Result<CardEffectRowObjectPredicate, TabulaError> {
    CardEffectRowObjectPredicate::try_from(s).map_err(|_| TabulaError::InvalidField {
        file: file.to_path_buf(),
        card_id: Some(card_id),
        field: "object-predicate",
        message: format!("unknown object predicate '{s}'"),
    })
}

fn parse_object_predicate_opt(
    opt: &Option<String>,
    file: &Path,
    card_id: Uuid,
) -> Result<Option<CardEffectRowObjectPredicate>, TabulaError> {
    match opt {
        Some(s) => parse_object_predicate(s, file, card_id).map(Some),
        None => Ok(None),
    }
}

fn parse_string_wrapper_opt<T: StringWrapper>(opt: &Option<String>) -> Option<T> {
    opt.as_ref().and_then(|s| T::from_string_value(s).ok())
}

fn parse_display_color_opt(
    opt: &Option<String>,
    file: &Path,
    card_id: Uuid,
) -> Result<Option<DisplayColor>, TabulaError> {
    match opt {
        Some(s) => {
            DisplayColor::from_string_value(s).map(Some).map_err(|e| TabulaError::InvalidField {
                file: file.to_path_buf(),
                card_id: Some(card_id),
                field: "dissolve-color",
                message: e,
            })
        }
        None => Ok(None),
    }
}
