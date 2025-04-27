use battle_data::battle::battle_data::BattleData;
use battle_data::battle::battle_turn_step::BattleTurnStep;
use battle_data::battle::effect_source::EffectSource;
use battle_data::battle_animations::battle_animation::BattleAnimation;
use core_data::numerics::TurnId;
use core_data::types::PlayerName;
use logging::battle_trace;

use crate::dreamwell_phase::dreamwell;
use crate::judgment_phase::judgment;
use crate::zone_mutations::deck;

/// Start a turn for `player`.
pub fn run(battle: &mut BattleData, player: PlayerName) {
    battle_trace!("Starting turn for", battle, player);
    battle.turn.active_player = player;
    battle.priority = player;
    battle.turn.turn_id += TurnId(1);
    let source = EffectSource::Game { controller: player };
    battle.push_animation(|| BattleAnimation::StartTurn { player });
    judgment::run(battle, battle.turn.active_player, source);
    dreamwell::activate(battle, battle.turn.active_player, source);
    battle.step = BattleTurnStep::Draw;
    deck::draw_cards(battle, source, player, 1);
    battle.step = BattleTurnStep::Main;
}
