use asset_paths::hovl_projectile;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardId, CardIdType};
use battle_state::battle_cards::stack_card_state::StackCardTargets;
use core_data::identifiers::CardName;
use display_data::command::{Command, CommandSequence, FireProjectileCommand, GameObjectId};

/// Returns an optional [CommandSequence] to display card effects to a target
/// during effect resolution.
pub fn effect(
    battle: &BattleState,
    source_id: CardId,
    targets: &StackCardTargets,
) -> Option<CommandSequence> {
    match battle.cards.card(source_id).name {
        CardName::Immolate => {
            Some(CommandSequence::from_command(Command::FireProjectile(FireProjectileCommand {
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
            })))
        }
        _ => None,
    }
}

fn target_id(targets: &StackCardTargets) -> Option<GameObjectId> {
    match targets {
        StackCardTargets::Character(character_id) => {
            Some(GameObjectId::CardId(character_id.card_id()))
        }
        StackCardTargets::StackCard(stack_card_id) => {
            Some(GameObjectId::CardId(stack_card_id.card_id()))
        }
    }
}
