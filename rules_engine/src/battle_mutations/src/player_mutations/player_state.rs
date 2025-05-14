use battle_state::battle::battle_state::BattleState;
use core_data::types::PlayerName;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::card_mutations::player_hand;

/// Returns a new [BattleState] with a new randomly-generated seed and with the
/// given player's hand randomized with their deck.
pub fn randomize_battle_player(battle: &BattleState, player: PlayerName) -> BattleState {
    let mut result = battle.logical_clone();
    let seed = rand::rng().random();
    result.rng = Xoshiro256PlusPlus::seed_from_u64(seed);
    result.seed = seed;
    player_hand::randomize_player_hand(&mut result, player);
    result
}
