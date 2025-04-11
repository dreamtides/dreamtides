use core_data::identifiers::BattleId;
use core_data::types::PlayerName;
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::battle::battle_status::BattleStatus;
use crate::battle::battle_turn_step::BattleTurnStep;
use crate::battle::request_context::RequestContext;
use crate::battle::turn_data::TurnData;
use crate::cards::all_cards::AllCards;
use crate::player::player_data::PlayerData;

/// Contains data types for a "battle", a single instance of playing a game
/// against an enemy.
#[derive(Clone, Debug)]
pub struct BattleData {
    pub id: BattleId,
    pub user: PlayerData,
    pub enemy: PlayerData,
    pub cards: AllCards,
    pub status: BattleStatus,
    pub turn: TurnData,
    pub step: BattleTurnStep,
    pub rng: Xoshiro256PlusPlus,
    pub request_context: RequestContext,
}

impl BattleData {
    pub fn player(&self, player_name: PlayerName) -> &PlayerData {
        match player_name {
            PlayerName::User => &self.user,
            PlayerName::Enemy => &self.enemy,
        }
    }

    pub fn player_mut(&mut self, player_name: PlayerName) -> &mut PlayerData {
        match player_name {
            PlayerName::User => &mut self.user,
            PlayerName::Enemy => &mut self.enemy,
        }
    }
}
