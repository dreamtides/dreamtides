use asset_paths::{dissolve_material, hovl_projectile, wow_sound};
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardId, CardIdType};
use battle_state::battle_cards::stack_card_state::StackCardTargets;
use core_data::display_color;
use core_data::identifiers::CardName;
use display_data::command::{Command, DissolveCardCommand, FireProjectileCommand, GameObjectId};

use crate::core::response_builder::ResponseBuilder;

/// Displays card effects to a target during effect resolution by adding
/// commands directly to the response builder.
pub fn apply(
    builder: &mut ResponseBuilder,
    battle: &BattleState,
    source_id: CardId,
    targets: &StackCardTargets,
) -> Option<()> {
    match battle.cards.card(source_id).name {
        CardName::Immolate => {
            builder.push(Command::FireProjectile(
                FireProjectileCommand::builder()
                    .source_id(GameObjectId::CardId(source_id))
                    .target_id(target_id(targets)?)
                    .projectile(hovl_projectile::address(1, "Projectile 3 black fire"))
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
        _ => {}
    }

    Some(())
}

fn target_id(targets: &StackCardTargets) -> Option<GameObjectId> {
    Some(GameObjectId::CardId(target_card_id(targets)?))
}

fn target_card_id(targets: &StackCardTargets) -> Option<CardId> {
    match targets {
        StackCardTargets::Character(character_id) => Some(character_id.card_id()),
        StackCardTargets::StackCard(stack_card_id) => Some(stack_card_id.card_id()),
    }
}
