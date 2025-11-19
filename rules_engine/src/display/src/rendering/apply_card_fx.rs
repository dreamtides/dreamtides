use battle_queries::battle_card_queries::{card, card_properties};
use battle_state::battle::battle_animation_data::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardId, CardIdType};
use battle_state::battle_cards::zone::Zone;
use battle_state::core::effect_source::EffectSource;
use core_data::display_types::{EffectAddress, Milliseconds};
use core_data::types::PlayerName;
use display_data::card_view::{CardEffects, ClientCardId};
use display_data::command::{
    Command, DisplayEffectCommand, DissolveCardCommand, FireProjectileCommand, GameObjectId,
    SetCardTrailCommand,
};
use masonry::flex_style::FlexVector3;
use tabula_data::card_effect_definitions::card_effect_row::{
    CardEffectRow, CardEffectRowObjectPredicate, CardEffectRowTrigger, CardEffectRowType,
};
use tracing::warn;

use crate::core::adapter;
use crate::core::response_builder::ResponseBuilder;
use crate::rendering::animations;

/// Apply visual & sound effects for a specific card's ability.
pub fn apply_effect(
    builder: &mut ResponseBuilder,
    effect_source: EffectSource,
    animation: &BattleAnimation,
    battle: &BattleState,
) -> Option<()> {
    let source_id = effect_source.card_id()?;
    let base_card_id = card::get_base_card_id(battle, source_id);
    let trigger = animation_trigger(animation)?;
    let controller = card_properties::controller(battle, source_id);
    let tabula_ref = builder.tabula();
    let rows: Vec<&CardEffectRow> = tabula_ref
        .card_effects
        .iter()
        .filter(|r| r.card_id == base_card_id && r.effect_trigger == trigger)
        .collect();
    if rows.is_empty() {
        return Some(());
    }
    animations::push_snapshot(builder, battle);
    let animation_targets = find_target_ids(animation);
    for row in rows {
        match row.effect_type {
            CardEffectRowType::FireProjectile => {
                fire_projectile(
                    builder,
                    battle,
                    row,
                    source_id,
                    controller,
                    animation,
                    &animation_targets,
                );
            }
            CardEffectRowType::DissolveTargets => {
                dissolve_targets(builder, row, &animation_targets, false);
            }
            CardEffectRowType::ReverseDissolveTargets => {
                dissolve_targets(builder, row, &animation_targets, true);
            }
            CardEffectRowType::DisplayEffect => {
                display_effect(builder, battle, row, source_id, controller, animation);
            }
            CardEffectRowType::SetCardTrail => {
                set_card_trail(builder, row, source_id, animation, &animation_targets);
            }
        }
    }
    Some(())
}

/// Returns the persistent visual effects for a given card.
pub fn persistent_card_effects(battle: &BattleState, card_id: CardId) -> CardEffects {
    CardEffects { looping_effect: looping_card_effect(battle, card_id), ..Default::default() }
}

/// Returns true if the given card has applied the 'anchored' effect.
pub fn is_anchored(battle: &BattleState, card_id: CardId) -> bool {
    battle
        .ability_state
        .until_end_of_turn
        .prevent_dissolved
        .iter()
        .any(|&cid| cid.card_id.card_id() == card_id)
}

fn looping_card_effect(battle: &BattleState, card_id: CardId) -> Option<EffectAddress> {
    let controller = card_properties::controller(battle, card_id);
    if !battle.cards.contains_card(controller, card_id, Zone::Battlefield) {
        return None;
    }

    if is_anchored(battle, card_id) {
        return Some(EffectAddress::new(
            "Assets/ThirdParty/Hovl Studio/Magic circles/Dreamtides/Looping/Magic shield 4 loop.prefab",
        ));
    }
    None
}

fn find_target_ids(animation: &BattleAnimation) -> Vec<ClientCardId> {
    match animation {
        BattleAnimation::SelectedTargetsForCard { targets, .. } => {
            targets.card_ids().iter().map(|id| adapter::client_card_id(*id)).collect()
        }
        BattleAnimation::ApplyTargetedEffect { targets, .. } => {
            targets.iter().map(|id| adapter::client_card_id(*id)).collect()
        }
        BattleAnimation::DrawCards { cards, .. } => {
            cards.iter().map(|id| adapter::client_card_id(id.card_id())).collect()
        }
        _ => vec![],
    }
}

fn animation_trigger(animation: &BattleAnimation) -> Option<CardEffectRowTrigger> {
    match animation {
        BattleAnimation::ApplyTargetedEffect { .. } => {
            Some(CardEffectRowTrigger::ApplyTargetedEffect)
        }
        BattleAnimation::DrawCards { .. } => Some(CardEffectRowTrigger::DrawCards),
        BattleAnimation::ActivatedAbility { .. } => Some(CardEffectRowTrigger::ActivatedAbility),
        BattleAnimation::SelectedTargetsForCard { .. } => {
            Some(CardEffectRowTrigger::SelectedTargetsForCard)
        }
        _ => None,
    }
}

fn resolve_object_predicate(
    builder: &ResponseBuilder,
    _battle: &BattleState,
    predicate: &CardEffectRowObjectPredicate,
    source_id: CardId,
    controller: PlayerName,
    _animation: &BattleAnimation,
    animation_targets: &[ClientCardId],
) -> Vec<GameObjectId> {
    match predicate {
        CardEffectRowObjectPredicate::ThisCard => {
            vec![adapter::card_game_object_id(source_id)]
        }
        CardEffectRowObjectPredicate::ForEachTarget => {
            animation_targets.iter().map(adapter::card_game_object_client_id).collect()
        }
        CardEffectRowObjectPredicate::ControllerDeck => {
            vec![GameObjectId::Deck(builder.to_display_player(controller))]
        }
    }
}

fn resolve_card_ids_predicate(
    predicate: &CardEffectRowObjectPredicate,
    source_id: CardId,
    animation_targets: &[ClientCardId],
) -> Vec<ClientCardId> {
    match predicate {
        CardEffectRowObjectPredicate::ThisCard => vec![adapter::client_card_id(source_id)],
        CardEffectRowObjectPredicate::ForEachTarget => animation_targets.to_vec(),
        CardEffectRowObjectPredicate::ControllerDeck => vec![],
    }
}

fn fire_projectile(
    builder: &mut ResponseBuilder,
    battle: &BattleState,
    row: &CardEffectRow,
    source_id: CardId,
    controller: PlayerName,
    animation: &BattleAnimation,
    animation_targets: &[ClientCardId],
) {
    let projectile_address = match &row.projectile_address {
        Some(a) => a.as_ref().clone(),
        None => {
            warn!(?row.card_id, "Missing projectile_address for FireProjectile effect");
            return;
        }
    };
    let source_pred = match &row.projectile_source {
        Some(p) => p,
        None => {
            warn!(?row.card_id, "Missing projectile_source for FireProjectile effect");
            return;
        }
    };
    let target_pred = match &row.projectile_target {
        Some(p) => p,
        None => {
            warn!(?row.card_id, "Missing projectile_target for FireProjectile effect");
            return;
        }
    };
    let sources = resolve_object_predicate(
        builder,
        battle,
        source_pred,
        source_id,
        controller,
        animation,
        animation_targets,
    );
    let targets = resolve_object_predicate(
        builder,
        battle,
        target_pred,
        source_id,
        controller,
        animation,
        animation_targets,
    );
    if targets.is_empty() {
        warn!(?row.card_id, "No targets resolved for FireProjectile effect");
        return;
    }
    for s in &sources {
        for t in &targets {
            let mut cmd = FireProjectileCommand::builder()
                .source_id(s.clone())
                .target_id(t.clone())
                .projectile(projectile_address.clone())
                .build();
            cmd.fire_sound = row.projectile_fire_sound.as_ref().map(|v| v.as_ref().clone());
            cmd.impact_sound = row.projectile_impact_sound.as_ref().map(|v| v.as_ref().clone());
            builder.push(Command::FireProjectile(cmd));
        }
    }
}

fn dissolve_targets(
    builder: &mut ResponseBuilder,
    row: &CardEffectRow,
    animation_targets: &[ClientCardId],
    reverse: bool,
) {
    let material = match &row.dissolve_material {
        Some(m) => m.as_ref().clone(),
        None => {
            warn!(?row.card_id, "Missing dissolve_material for dissolve effect");
            return;
        }
    };
    let color = match &row.dissolve_color {
        Some(c) => *c.as_ref(),
        None => {
            warn!(?row.card_id, "Missing dissolve_color for dissolve effect");
            return;
        }
    };
    for card_id in animation_targets {
        let mut cmd = DissolveCardCommand::builder()
            .target(card_id.clone())
            .material(material.clone())
            .color(color)
            .reverse(reverse)
            .build();
        cmd.sound = row.dissolve_sound.as_ref().map(|v| v.as_ref().clone());
        if reverse {
            builder.run_with_next_battle_view(Command::DissolveCard(cmd));
        } else {
            builder.push(Command::DissolveCard(cmd));
        }
    }
}

fn display_effect(
    builder: &mut ResponseBuilder,
    battle: &BattleState,
    row: &CardEffectRow,
    source_id: CardId,
    controller: PlayerName,
    animation: &BattleAnimation,
) {
    let target_pred = match &row.effect_target {
        Some(p) => p,
        None => {
            warn!(?row.card_id, "Missing effect_target for DisplayEffect effect");
            return;
        }
    };
    let targets = resolve_object_predicate(
        builder,
        battle,
        target_pred,
        source_id,
        controller,
        animation,
        &find_target_ids(animation),
    );
    if targets.is_empty() {
        warn!(?row.card_id, "No targets resolved for DisplayEffect effect");
        return;
    }
    let effect_address = match &row.effect_address {
        Some(e) => e.as_ref().clone(),
        None => {
            warn!(?row.card_id, "Missing effect_address for DisplayEffect effect");
            return;
        }
    };
    let duration = match &row.effect_duration_milliseconds {
        Some(d) => *d.as_ref(),
        None => {
            warn!(?row.card_id, "Missing effect_duration_milliseconds for DisplayEffect effect; defaulting to 500ms");
            Milliseconds::new(500)
        }
    };
    let scale = match row.effect_scale {
        Some(s) => s,
        None => {
            warn!(?row.card_id, "Missing effect_scale for DisplayEffect effect");
            return;
        }
    };
    for t in targets {
        builder.push(Command::DisplayEffect(DisplayEffectCommand {
            target: t,
            effect: effect_address.clone(),
            duration,
            scale: FlexVector3::new(scale as f32, scale as f32, scale as f32),
            sound: row.effect_sound.as_ref().map(|v| v.as_ref().clone()),
        }));
    }
}

fn set_card_trail(
    builder: &mut ResponseBuilder,
    row: &CardEffectRow,
    source_id: CardId,
    _animation: &BattleAnimation,
    animation_targets: &[ClientCardId],
) {
    let targets_pred = match &row.card_trail_targets {
        Some(p) => p,
        None => {
            warn!(?row.card_id, "Missing card_trail_targets for SetCardTrail effect");
            return;
        }
    };
    let card_ids = resolve_card_ids_predicate(targets_pred, source_id, animation_targets);
    if card_ids.is_empty() {
        warn!(?row.card_id, "No card ids resolved for SetCardTrail effect");
        return;
    }
    let trail = match &row.card_trail_address {
        Some(t) => t.as_ref().clone(),
        None => {
            warn!(?row.card_id, "Missing card_trail_address for SetCardTrail effect");
            return;
        }
    };
    let duration = match &row.trail_duration_milliseconds {
        Some(d) => *d.as_ref(),
        None => {
            warn!(?row.card_id, "Missing trail_duration_milliseconds for SetCardTrail effect");
            return;
        }
    };
    builder.push(Command::SetCardTrail(SetCardTrailCommand { card_ids, trail, duration }));
}
