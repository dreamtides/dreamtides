use asset_paths::{dissolve_material, hovl, wow_sound};
use battle_queries::battle_card_queries::{card, card_properties};
use battle_state::battle::battle_animation::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardId, CardIdType};
use battle_state::battle_cards::zone::Zone;
use battle_state::core::effect_source::EffectSource;
use core_data::display_color;
use core_data::display_types::{AudioClipAddress, EffectAddress, Milliseconds, ProjectileAddress};
use core_data::identifiers::CardName;
use display_data::card_view::{CardEffects, ClientCardId};
use display_data::command::{
    Command, DisplayEffectCommand, DissolveCardCommand, FireProjectileCommand, GameObjectId,
    SetCardTrailCommand,
};
use masonry::flex_style::FlexVector3;
use quest_state::quest::card_descriptor;
use strum::IntoDiscriminant;

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
    let controller = card_properties::controller(battle, source_id);
    let effect_name = animation.discriminant().to_string();
    match card_descriptor::get_base_card_id(card::get(battle, source_id).identity) {
        CardName::TestDissolve if effect_name == "ApplyTargetedEffect" => {
            animations::push_snapshot(builder, battle);
            for target_id in find_target_ids(animation) {
                builder.push(Command::FireProjectile(
                    FireProjectileCommand::builder()
                        .source_id(adapter::card_game_object_id(source_id))
                        .target_id(adapter::card_game_object_client_id(&target_id))
                        .projectile(hovl::projectile(1, "Projectile 3 black fire"))
                        .fire_sound(wow_sound::rpg_magic(
                            3,
                            "Fire Magic/RPG3_FireMagicArrow_Projectile01",
                        ))
                        .impact_sound(wow_sound::rpg_magic(3, "Fire Magic/RPG3_FireMagic_Impact01"))
                        .build(),
                ));
                builder.push(Command::DissolveCard(
                    DissolveCardCommand::builder()
                        .target(target_id.clone())
                        .material(dissolve_material::material(15))
                        .color(display_color::ORANGE_500)
                        .reverse(false)
                        .build(),
                ));
                builder.run_with_next_battle_view(Command::DissolveCard(
                    DissolveCardCommand::builder()
                        .target(target_id.clone())
                        .material(dissolve_material::material(15))
                        .color(display_color::ORANGE_500)
                        .reverse(true)
                        .sound(wow_sound::rpg_magic(
                            3,
                            "Fire Magic/RPG3_FireMagicBall_LightImpact03",
                        ))
                        .build(),
                ));
            }
        }

        CardName::TestNamedDissolve if effect_name == "ApplyTargetedEffect" => {
            animations::push_snapshot(builder, battle);
            for target_id in find_target_ids(animation) {
                builder.push(Command::FireProjectile(
                    FireProjectileCommand::builder()
                        .source_id(adapter::card_game_object_id(source_id))
                        .target_id(adapter::card_game_object_client_id(&target_id))
                        .projectile(hovl::projectile(1, "Projectile 3 black fire"))
                        .fire_sound(wow_sound::rpg_magic(
                            3,
                            "Fire Magic/RPG3_FireMagicArrow_Projectile01",
                        ))
                        .impact_sound(wow_sound::rpg_magic(3, "Fire Magic/RPG3_FireMagic_Impact01"))
                        .build(),
                ));
                builder.push(Command::DissolveCard(
                    DissolveCardCommand::builder()
                        .target(target_id.clone())
                        .material(dissolve_material::material(15))
                        .color(display_color::ORANGE_500)
                        .reverse(false)
                        .build(),
                ));
                builder.run_with_next_battle_view(Command::DissolveCard(
                    DissolveCardCommand::builder()
                        .target(target_id.clone())
                        .material(dissolve_material::material(15))
                        .color(display_color::ORANGE_500)
                        .reverse(true)
                        .sound(wow_sound::rpg_magic(
                            3,
                            "Fire Magic/RPG3_FireMagicBall_LightImpact03",
                        ))
                        .build(),
                ));
            }
        }

        CardName::TestCounterspell if effect_name == "ApplyTargetedEffect" => {
            animations::push_snapshot(builder, battle);
            for target_id in find_target_ids(animation) {
                builder.push(Command::FireProjectile(
                    FireProjectileCommand::builder()
                        .source_id(adapter::card_game_object_id(source_id))
                        .target_id(adapter::card_game_object_client_id(&target_id))
                        .projectile(hovl::projectile(1, "Projectile 6 blue fire"))
                        .fire_sound(wow_sound::rpg_magic(3, "Wind Magic/RPG3_WindMagic_Cast01"))
                        .impact_sound(wow_sound::rpg_magic(3, "Wind Magic/RPG3_WindMagic_Impact01"))
                        .build(),
                ));
            }
        }

        CardName::TestCounterspellCharacter if effect_name == "ApplyTargetedEffect" => {
            animations::push_snapshot(builder, battle);
            for target_id in find_target_ids(animation) {
                builder.push(Command::FireProjectile(
                    FireProjectileCommand::builder()
                        .source_id(adapter::card_game_object_id(source_id))
                        .target_id(adapter::card_game_object_client_id(&target_id))
                        .projectile(hovl::projectile(1, "Projectile 6 blue fire"))
                        .fire_sound(wow_sound::rpg_magic(3, "Wind Magic/RPG3_WindMagic_Cast01"))
                        .impact_sound(wow_sound::rpg_magic(3, "Wind Magic/RPG3_WindMagic_Impact01"))
                        .build(),
                ));
            }
        }

        CardName::TestCounterspellUnlessPays if effect_name == "ApplyTargetedEffect" => {
            animations::push_snapshot(builder, battle);
            for target_id in find_target_ids(animation) {
                builder.push(Command::FireProjectile(
                    FireProjectileCommand::builder()
                        .source_id(adapter::card_game_object_id(source_id))
                        .target_id(adapter::card_game_object_client_id(&target_id))
                        .projectile(hovl::projectile(1, "Projectile 10 blue laser"))
                        .fire_sound(wow_sound::rpg_magic(3, "Water Magic/RPG3_WaterMagic_Cast01"))
                        .impact_sound(wow_sound::rpg_magic(
                            3,
                            "Water Magic/RPG3_WaterMagic_Impact03",
                        ))
                        .build(),
                ));
            }
        }

        CardName::TestVariableEnergyDraw if effect_name == "DrawCards" => {
            animations::push_snapshot(builder, battle);
            builder.push(Command::DisplayEffect(DisplayEffectCommand {
                target: GameObjectId::Deck(builder.to_display_player(controller)),
                effect: hovl::magic_circle("1"),
                duration: Milliseconds::new(500),
                scale: FlexVector3::new(5.0, 5.0, 5.0),
                sound: Some(wow_sound::rpg_magic(
                    3,
                    "Light Magic/RPG3_LightMagicEpic_HealingWing_P1",
                )),
            }));
        }

        CardName::TestFastMultiActivatedAbilityDrawCardCharacter
            if effect_name == "ActivatedAbility" =>
        {
            animations::push_snapshot(builder, battle);
            builder.push(Command::DisplayEffect(DisplayEffectCommand {
                target: adapter::card_game_object_id(source_id),
                effect: hovl::magic_circle("2"),
                duration: Milliseconds::new(500),
                scale: FlexVector3::new(5.0, 5.0, 5.0),
                sound: Some(AudioClipAddress::new(
                    "Assets/ThirdParty/WowSound/RPG Magic Sound Effects Pack 3/Light Magic/RPG3_LightMagic_Cast03.wav",
                )),
            }));
        }

        CardName::TestReturnOneOrTwoVoidEventCardsToHand
            if effect_name == "SelectedTargetsForCard" =>
        {
            animations::push_snapshot(builder, battle);
            builder.push(Command::SetCardTrail(SetCardTrailCommand {
                card_ids: find_target_ids(animation),
                trail: ProjectileAddress::new("Assets/ThirdParty/Hovl Studio/AAA Projectiles Vol 1/Prefabs/Dreamtides/Projectile 7 pink.prefab"),
                duration: Milliseconds::new(500),
            }));
        }

        CardName::TestModalReturnToHandOrDrawTwo if effect_name == "ApplyTargetedEffect" => {
            animations::push_snapshot(builder, battle);
            for target_id in find_target_ids(animation) {
                builder.push(Command::FireProjectile(
                    FireProjectileCommand::builder()
                        .source_id(adapter::card_game_object_id(source_id))
                        .target_id(adapter::card_game_object_client_id(&target_id))
                        .projectile(hovl::projectile(1, "Projectile 5 red"))
                        .fire_sound(wow_sound::rpg_magic(3, "Plasma Magic/RPG3_PlasmaMagic_Cast01"))
                        .impact_sound(wow_sound::rpg_magic(
                            3,
                            "Plasma Magic/RPG3_PlasmaMagic_MediumImpact01",
                        ))
                        .build(),
                ));
            }
        }

        CardName::TestModalReturnToHandOrDrawTwo if effect_name == "DrawCards" => {
            animations::push_snapshot(builder, battle);
            builder.push(Command::DisplayEffect(DisplayEffectCommand {
                target: GameObjectId::Deck(builder.to_display_player(controller)),
                effect: hovl::magic_circle("9"),
                duration: Milliseconds::new(500),
                scale: FlexVector3::new(2.5, 2.5, 2.5),
                sound: Some(wow_sound::rpg_magic(3, "Fire Magic/RPG3_FireMagic_Cast04")),
            }));

            builder.push(Command::SetCardTrail(SetCardTrailCommand {
                card_ids: find_target_ids(animation),
                trail: hovl::projectile(1, "Projectile 5 red"),
                duration: Milliseconds::new(1500),
            }));
        }

        _ => {}
    }

    Some(())
}

/// Returns the persistent visual effects for a given card.
pub fn persistent_card_effects(battle: &BattleState, card_id: CardId) -> CardEffects {
    CardEffects { looping_effect: looping_card_effect(battle, card_id) }
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
