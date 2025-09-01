use std::sync::Arc;

use battle_state::battle::animation_data::AnimationData;
use battle_state::battle::battle_history::BattleHistory;
use battle_state::battle::battle_state::{BattleState, RequestContext};
use battle_state::battle_cards::dreamwell_data::Dreamwell;
use battle_state::battle_player::battle_player_state::CreateBattlePlayer;
use battle_state::battle_trace::battle_tracing::BattleTracing;
use core_data::identifiers::BattleId;
use tabula_data::tabula::Tabula;

use crate::new_test_battle;

/// Creates a new battle and starts it using a given seed and
/// [`CreateBattlePlayer`] specification.
pub fn create_and_start(
    battle_id: BattleId,
    tabula: Arc<Tabula>,
    seed: u64,
    dreamwell: Dreamwell,
    player_one: CreateBattlePlayer,
    player_two: CreateBattlePlayer,
    request_context: RequestContext,
) -> BattleState {
    let mut battle = new_test_battle::create_and_start(
        battle_id,
        tabula,
        seed,
        dreamwell,
        player_one,
        player_two,
        request_context,
    );
    battle.animations = Some(AnimationData::default());
    battle.tracing = Some(BattleTracing::default());
    battle.action_history = Some(BattleHistory::default());
    battle
}
