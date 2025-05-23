use asset_paths::{dissolve_material, hovl_projectile};
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
            builder.push(Command::FireProjectile(FireProjectileCommand {
                source_id: GameObjectId::CardId(source_id),
                target_id: target_id(targets)?,
                projectile: hovl_projectile::address(1, "Projectile 3 black fire"),
                travel_duration: None,
                fire_sound: None,
                impact_sound: None,
                additional_hit: None,
                additional_hit_delay: None,
                wait_duration: None,
                hide_on_hit: false,
                jump_to_position: None,
            }));

            builder.push(Command::DissolveCard(DissolveCardCommand {
                target: target_card_id(targets)?,
                reverse: false,
                material: dissolve_material::material(15),
                color: display_color::ORANGE_500,
                dissolve_speed: None,
            }));

            builder.run_with_next_battle_view(Command::DissolveCard(DissolveCardCommand {
                target: target_card_id(targets)?,
                reverse: true,
                material: dissolve_material::material(15),
                color: display_color::ORANGE_500,
                dissolve_speed: None,
            }));
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
