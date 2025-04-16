use battle_data::battle::battle_data::BattleData;
use battle_data::battle_animations::battle_animation::BattleAnimation;
use core_data::effect_source::EffectSource;
use core_data::numerics::TurnId;
use core_data::types::PlayerName;

use crate::dreamwell_phase::dreamwell;
use crate::judgment_phase::judgment;
use crate::zone_mutations::deck;

/// Start a turn for `player`.
pub fn run(battle: &mut BattleData, player: PlayerName, source: EffectSource) {
    battle.turn.active_player = player;
    battle.turn.turn_id += TurnId(1);
    battle.push_animation(|| BattleAnimation::StartTurn { player });
    judgment::run(battle, battle.turn.active_player, source);
    dreamwell::activate(battle, battle.turn.active_player, source);
    deck::draw_cards(battle, source, player, 1);
}
