use core_data::identifiers::BattleId;
use core_data::types::PlayerName;
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::battle::all_cards::AllCards;
use crate::battle::battle_status::BattleStatus;
use crate::battle::battle_turn_step::BattleTurnStep;
use crate::battle::player_map::PlayerMap;
use crate::battle::turn_data::TurnData;
use crate::battle_player::battle_player_state::BattlePlayerState;

#[derive(Clone, Debug)]
pub struct BattleState {
    /// Unique identifier for this battle
    pub id: BattleId,

    /// All cards in this battle
    pub cards: AllCards,

    /// Player data for all players in this battle
    pub players: PlayerMap<BattlePlayerState>,

    /// Status of this battle, including whether it has ended.
    pub status: BattleStatus,

    /// Player who is currently next to act when a stack of cards is active.
    pub stack_priority: Option<PlayerName>,

    /// Current turn
    pub turn: TurnData,

    /// Current step within the turn
    pub step: BattleTurnStep,

    /// Seed used to initialize the random number generator
    pub seed: u64,

    /// Random number generator for this battle
    pub rng: Xoshiro256PlusPlus,
}
