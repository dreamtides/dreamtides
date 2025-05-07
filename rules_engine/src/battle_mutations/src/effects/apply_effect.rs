use ability_data::effect::Effect;
use ability_data::standard_effect::StandardEffect;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardId, CharacterId};
use battle_state::core::effect_source::EffectSource;
use bit_set::BitSet;

use crate::character_mutations::dissolve;

pub fn execute(battle: &mut BattleState, source: EffectSource, effect: &Effect, targets: &BitSet) {
    match effect {
        Effect::Effect(standard) => apply_standard_effect(battle, source, standard, targets),
        _ => todo!("Implement this"),
    }
}

fn apply_standard_effect(
    battle: &mut BattleState,
    source: EffectSource,
    effect: &StandardEffect,
    targets: &BitSet,
) {
    match effect {
        StandardEffect::DissolveCharacter { .. } => dissolve(battle, source, targets),
        _ => todo!("Implement {:?}", effect),
    }
}

fn dissolve(battle: &mut BattleState, source: EffectSource, targets: &BitSet) {
    for target in targets {
        dissolve::apply(
            battle,
            source,
            source.controller().opponent(),
            CharacterId(CardId(target)),
        );
    }
}
