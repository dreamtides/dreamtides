use asset_paths::{dissolve_material, hovl, wow_sound};
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardId, CardIdType};
use battle_state::battle_cards::stack_card_state::StackCardTargets;
use core_data::display_color;
use core_data::display_types::Milliseconds;
use core_data::identifiers::CardName;
use core_data::types::PlayerName;
use display_data::command::{
    Command, DisplayEffectCommand, DissolveCardCommand, FireProjectileCommand, GameObjectId,
};
use masonry::flex_style::FlexVector3;

use crate::core::response_builder::ResponseBuilder;

/// Displays card effects to a target during effect resolution by adding
/// commands directly to the response builder.
pub fn apply(
    builder: &mut ResponseBuilder,
    battle: &BattleState,
    controller: PlayerName,
    source_id: CardId,
    targets: &Option<StackCardTargets>,
) -> Option<()> {
    match battle.cards.card(source_id).name {
        CardName::Immolate => {
            builder.push(Command::FireProjectile(
                FireProjectileCommand::builder()
                    .source_id(GameObjectId::CardId(source_id))
                    .target_id(target_id(targets)?)
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
                    .target(target_card_id(targets)?)
                    .material(dissolve_material::material(15))
                    .color(display_color::ORANGE_500)
                    .reverse(false)
                    .build(),
            ));
            builder.run_with_next_battle_view(Command::DissolveCard(
                DissolveCardCommand::builder()
                    .target(target_card_id(targets)?)
                    .material(dissolve_material::material(15))
                    .color(display_color::ORANGE_500)
                    .reverse(true)
                    .sound(wow_sound::rpg_magic(3, "Fire Magic/RPG3_FireMagicBall_LightImpact03"))
                    .build(),
            ));
        }

        CardName::Abolish => {
            builder.push(Command::FireProjectile(
                FireProjectileCommand::builder()
                    .source_id(GameObjectId::CardId(source_id))
                    .target_id(target_id(targets)?)
                    .projectile(hovl::projectile(1, "Projectile 6 blue fire"))
                    .fire_sound(wow_sound::rpg_magic(3, "Wind Magic/RPG3_WindMagic_Cast01"))
                    .impact_sound(wow_sound::rpg_magic(3, "Wind Magic/RPG3_WindMagic_Impact01"))
                    .build(),
            ));
        }

        CardName::RippleOfDefiance => {
            builder.push(Command::FireProjectile(
                FireProjectileCommand::builder()
                    .source_id(GameObjectId::CardId(source_id))
                    .target_id(target_id(targets)?)
                    .projectile(hovl::projectile(1, "Projectile 10 blue laser"))
                    .fire_sound(wow_sound::rpg_magic(3, "Water Magic/RPG3_WaterMagic_Cast01"))
                    .impact_sound(wow_sound::rpg_magic(3, "Water Magic/RPG3_WaterMagic_Impact03"))
                    .build(),
            ));
        }

        CardName::Dreamscatter => {
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

        _ => {}
    }

    Some(())
}

fn target_id(targets: &Option<StackCardTargets>) -> Option<GameObjectId> {
    Some(GameObjectId::CardId(target_card_id(targets)?))
}

fn target_card_id(targets: &Option<StackCardTargets>) -> Option<CardId> {
    match targets {
        Some(StackCardTargets::Character(character_id)) => Some(character_id.card_id()),
        Some(StackCardTargets::StackCard(stack_card_id)) => Some(stack_card_id.card_id()),
        _ => None,
    }
}
