use battle_state::battle::battle_state::BattleState;
use core_data::types::PlayerName;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::card_mutations::{dreamwell_deck, player_hand};

/// Returns a new [BattleState] with a new randomly-generated seed and with the
/// given player's hand randomized with their deck.
pub fn randomize_battle_player(battle: &BattleState, player: PlayerName, seed: u64) -> BattleState {
    let mut result = battle.logical_clone();
    result.rng = Xoshiro256PlusPlus::seed_from_u64(seed);
    result.seed = seed;
    player_hand::randomize_player_hand(&mut result, player);
    dreamwell_deck::randomize(&mut result);
    result
}
