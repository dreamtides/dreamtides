use ability_data::effect::Effect;
use ability_data::standard_effect::StandardEffect;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CharacterId;
use battle_state::battle_cards::stack_card_state::StackCardTargets;
use battle_state::core::effect_source::EffectSource;
use tracing_macros::panic_with;

use crate::character_mutations::dissolve;

pub fn execute(
    battle: &mut BattleState,
    source: EffectSource,
    effect: &Effect,
    targets: &StackCardTargets,
) {
    match effect {
        Effect::Effect(standard) => apply_standard_effect(battle, source, standard, targets),
        _ => todo!("Implement this"),
    }
}

fn apply_standard_effect(
    battle: &mut BattleState,
    source: EffectSource,
    effect: &StandardEffect,
    targets: &StackCardTargets,
) {
    match effect {
        StandardEffect::DissolveCharacter { .. } => dissolve(battle, source, targets),
        _ => todo!("Implement {:?}", effect),
    }
}

fn dissolve(battle: &mut BattleState, source: EffectSource, targets: &StackCardTargets) {
    let id = character_id(battle, targets);
    dissolve::apply(battle, source, source.controller().opponent(), id);
}

fn character_id(battle: &mut BattleState, targets: &StackCardTargets) -> CharacterId {
    match targets {
        StackCardTargets::Character(character_id) => *character_id,
        _ => {
            panic_with!("Stack card targets should be a character", battle)
        }
    }
}
