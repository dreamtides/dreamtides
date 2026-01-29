use std::path::PathBuf;

use tabula_data_v2::card_effect_row::{
    CardEffectRowObjectPredicate, CardEffectRowTrigger, CardEffectRowType, build_card_effect_row,
};
use tabula_data_v2::toml_loader::CardEffectRowRaw;

fn test_file() -> PathBuf {
    PathBuf::from("test.toml")
}

fn raw_fire_projectile() -> CardEffectRowRaw {
    CardEffectRowRaw {
        card_id: "36c2a4e1-3212-4933-979a-73f109f9b256".to_string(),
        effect_type: "FireProjectile".to_string(),
        effect_trigger: "ApplyTargetedEffect".to_string(),
        projectile_source: Some("ThisCard".to_string()),
        projectile_target: Some("ForEachTarget".to_string()),
        projectile_address: Some("Assets/Projectiles/Fire.prefab".to_string()),
        projectile_fire_sound: Some("Assets/Sounds/Fire.wav".to_string()),
        projectile_impact_sound: Some("Assets/Sounds/Impact.wav".to_string()),
        dissolve_material: None,
        dissolve_color: None,
        dissolve_sound: None,
        effect_target: None,
        effect_address: None,
        effect_duration_milliseconds: None,
        effect_scale: None,
        effect_sound: None,
        card_trail_targets: None,
        card_trail_address: None,
        trail_duration_milliseconds: None,
    }
}

fn raw_dissolve_targets() -> CardEffectRowRaw {
    CardEffectRowRaw {
        card_id: "36c2a4e1-3212-4933-979a-73f109f9b256".to_string(),
        effect_type: "DissolveTargets".to_string(),
        effect_trigger: "ApplyTargetedEffect".to_string(),
        projectile_source: None,
        projectile_target: None,
        projectile_address: None,
        projectile_fire_sound: None,
        projectile_impact_sound: None,
        dissolve_material: Some("Assets/Dissolves/Dissolve15.mat".to_string()),
        dissolve_color: Some("#FFC107".to_string()),
        dissolve_sound: Some("Assets/Sounds/Dissolve.wav".to_string()),
        effect_target: None,
        effect_address: None,
        effect_duration_milliseconds: None,
        effect_scale: None,
        effect_sound: None,
        card_trail_targets: None,
        card_trail_address: None,
        trail_duration_milliseconds: None,
    }
}

fn raw_display_effect() -> CardEffectRowRaw {
    CardEffectRowRaw {
        card_id: "a6e8e100-9aa6-4a53-82ec-4ab161829533".to_string(),
        effect_type: "DisplayEffect".to_string(),
        effect_trigger: "DrawCards".to_string(),
        projectile_source: None,
        projectile_target: None,
        projectile_address: None,
        projectile_fire_sound: None,
        projectile_impact_sound: None,
        dissolve_material: None,
        dissolve_color: None,
        dissolve_sound: None,
        effect_target: Some("ControllerDeck".to_string()),
        effect_address: Some("Assets/Effects/MagicCircle.prefab".to_string()),
        effect_duration_milliseconds: Some(500),
        effect_scale: Some(5.0),
        effect_sound: Some("Assets/Sounds/Magic.wav".to_string()),
        card_trail_targets: None,
        card_trail_address: None,
        trail_duration_milliseconds: None,
    }
}

fn raw_set_card_trail() -> CardEffectRowRaw {
    CardEffectRowRaw {
        card_id: "1ce137cb-4234-45f7-abd2-d0ce97549885".to_string(),
        effect_type: "SetCardTrail".to_string(),
        effect_trigger: "SelectedTargetsForCard".to_string(),
        projectile_source: None,
        projectile_target: None,
        projectile_address: None,
        projectile_fire_sound: None,
        projectile_impact_sound: None,
        dissolve_material: None,
        dissolve_color: None,
        dissolve_sound: None,
        effect_target: None,
        effect_address: None,
        effect_duration_milliseconds: None,
        effect_scale: None,
        effect_sound: None,
        card_trail_targets: Some("ForEachTarget".to_string()),
        card_trail_address: Some("Assets/Trails/Pink.prefab".to_string()),
        trail_duration_milliseconds: Some(500),
    }
}

#[test]
fn build_fire_projectile_effect_succeeds() {
    let raw = raw_fire_projectile();
    let result = build_card_effect_row(&raw, &test_file());

    assert!(result.is_ok());
    let effect = result.unwrap();
    assert_eq!(effect.effect_type, CardEffectRowType::FireProjectile);
    assert_eq!(effect.effect_trigger, CardEffectRowTrigger::ApplyTargetedEffect);
    assert_eq!(effect.projectile_source, Some(CardEffectRowObjectPredicate::ThisCard));
    assert_eq!(effect.projectile_target, Some(CardEffectRowObjectPredicate::ForEachTarget));
    assert!(effect.projectile_address.is_some());
    assert!(effect.projectile_fire_sound.is_some());
    assert!(effect.projectile_impact_sound.is_some());
}

#[test]
fn build_dissolve_targets_effect_succeeds() {
    let raw = raw_dissolve_targets();
    let result = build_card_effect_row(&raw, &test_file());

    assert!(result.is_ok());
    let effect = result.unwrap();
    assert_eq!(effect.effect_type, CardEffectRowType::DissolveTargets);
    assert!(effect.dissolve_material.is_some());
    assert!(effect.dissolve_color.is_some());
    assert!(effect.dissolve_sound.is_some());
}

#[test]
fn build_display_effect_succeeds() {
    let raw = raw_display_effect();
    let result = build_card_effect_row(&raw, &test_file());

    assert!(result.is_ok());
    let effect = result.unwrap();
    assert_eq!(effect.effect_type, CardEffectRowType::DisplayEffect);
    assert_eq!(effect.effect_trigger, CardEffectRowTrigger::DrawCards);
    assert_eq!(effect.effect_target, Some(CardEffectRowObjectPredicate::ControllerDeck));
    assert!(effect.effect_address.is_some());
    assert!(effect.effect_duration_milliseconds.is_some());
    assert_eq!(effect.effect_scale, Some(5.0));
    assert!(effect.effect_sound.is_some());
}

#[test]
fn build_set_card_trail_effect_succeeds() {
    let raw = raw_set_card_trail();
    let result = build_card_effect_row(&raw, &test_file());

    assert!(result.is_ok());
    let effect = result.unwrap();
    assert_eq!(effect.effect_type, CardEffectRowType::SetCardTrail);
    assert_eq!(effect.effect_trigger, CardEffectRowTrigger::SelectedTargetsForCard);
    assert_eq!(effect.card_trail_targets, Some(CardEffectRowObjectPredicate::ForEachTarget));
    assert!(effect.card_trail_address.is_some());
    assert!(effect.trail_duration_milliseconds.is_some());
}

#[test]
fn build_effect_invalid_uuid_fails() {
    let mut raw = raw_fire_projectile();
    raw.card_id = "not-a-uuid".to_string();

    let result = build_card_effect_row(&raw, &test_file());
    assert!(result.is_err());
}

#[test]
fn build_effect_invalid_effect_type_fails() {
    let mut raw = raw_fire_projectile();
    raw.effect_type = "InvalidType".to_string();

    let result = build_card_effect_row(&raw, &test_file());
    assert!(result.is_err());
}

#[test]
fn build_effect_invalid_trigger_fails() {
    let mut raw = raw_fire_projectile();
    raw.effect_trigger = "InvalidTrigger".to_string();

    let result = build_card_effect_row(&raw, &test_file());
    assert!(result.is_err());
}

#[test]
fn build_effect_invalid_predicate_fails() {
    let mut raw = raw_fire_projectile();
    raw.projectile_source = Some("InvalidPredicate".to_string());

    let result = build_card_effect_row(&raw, &test_file());
    assert!(result.is_err());
}

#[test]
fn build_effect_invalid_color_fails() {
    let mut raw = raw_dissolve_targets();
    raw.dissolve_color = Some("not-a-color".to_string());

    let result = build_card_effect_row(&raw, &test_file());
    assert!(result.is_err());
}

#[test]
fn build_effect_color_parsing() {
    let raw = raw_dissolve_targets();
    let result = build_card_effect_row(&raw, &test_file());

    assert!(result.is_ok());
    let effect = result.unwrap();
    let color = effect.dissolve_color.unwrap();

    // #FFC107 = RGB(255, 193, 7)
    assert!((color.red - 1.0).abs() < 0.01);
    assert!((color.green - 0.757).abs() < 0.01);
    assert!((color.blue - 0.027).abs() < 0.01);
}
