use battle_queries::battle_trace;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CharacterId;
use battle_state::core::effect_source::EffectSource;
use battle_state::core::should_animate::ShouldAnimate;
use core_data::numerics::Points;
use core_data::types::PlayerName;

use crate::character_mutations::dissolve;
use crate::player_mutations::points;

/// Resolves one column of front-rank combat during the Judgment phase.
///
/// The non-active player's front-rank characters are attackers. The active
/// player's front-rank characters are blockers. Returns true if all 8
/// positions have been processed.
pub fn run(battle: &mut BattleState, player: PlayerName, source: EffectSource) -> bool {
    let position = battle.turn.judgment_position;
    let opponent = player.opponent();
    battle_trace!("Judgment phase resolving position", battle, position, player);

    let attacker_id = battle.cards.battlefield(opponent).front[position as usize];
    let blocker_id = battle.cards.battlefield(player).front[position as usize];

    match (attacker_id, blocker_id) {
        (Some(attacker), Some(blocker)) => {
            battle.turn.judgment_participants.push((opponent, attacker, position));
            battle.turn.judgment_participants.push((player, blocker, position));
            let attacker_spark = battle.cards.spark(opponent, attacker).unwrap_or_default();
            let blocker_spark = battle.cards.spark(player, blocker).unwrap_or_default();
            if attacker_spark > blocker_spark {
                dissolve::execute(battle, source, blocker);
            } else if blocker_spark > attacker_spark {
                dissolve::execute(battle, source, attacker);
            } else {
                dissolve::execute(battle, source, blocker);
                dissolve::execute(battle, source, attacker);
            }
        }
        (Some(attacker), None) => {
            battle.turn.judgment_participants.push((opponent, attacker, position));
            let spark = battle.cards.spark(opponent, attacker).unwrap_or_default();
            points::gain(battle, opponent, source, Points(spark.0), ShouldAnimate::Yes);
        }
        _ => {}
    }

    if position >= 7 {
        true
    } else {
        battle.turn.judgment_position = position + 1;
        false
    }
}

/// After all Judgment columns resolve, move surviving participants back to
/// the back rank. Characters that were dissolved during Judgment are already
/// in the Void and are skipped.
pub fn return_participants_to_back_rank(battle: &mut BattleState) {
    let participants: Vec<(PlayerName, CharacterId, u8)> =
        battle.turn.judgment_participants.drain(..).collect();
    for (player, character_id, _column) in participants {
        let bf = battle.cards.battlefield_mut(player);
        if bf.is_in_front_rank(character_id) {
            bf.return_to_back_rank(character_id);
        }
    }
}
