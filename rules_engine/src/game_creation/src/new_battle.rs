use battle_data::battle::battle_data::BattleData;
use battle_data::battle::battle_status::BattleStatus;
use battle_data::battle::battle_turn_step::BattleTurnStep;
use battle_data::battle::turn_data::TurnData;
use battle_data::cards::all_cards::AllCards;
use battle_data::cards::card_properties::CardProperties;
use battle_data::cards::zone::Zone;
use battle_data::player::player_data::PlayerData;
use core_data::identifiers::{BattleId, CardIdentity};
use core_data::numerics::{Spark, TurnNumber};
use core_data::types::PlayerName;
use uuid::Uuid;

/// Creates a new battle and starts it.
pub fn create_and_start(id: BattleId) -> BattleData {
    let mut battle = BattleData {
        id,
        user: PlayerData::default(),
        enemy: PlayerData::default(),
        cards: AllCards::default(),
        status: BattleStatus::Setup,
        turn: TurnData { active_player: PlayerName::User, turn_number: TurnNumber::default() },
        step: BattleTurnStep::Judgment,
    };
    create_cards(&mut battle);
    battle.status = BattleStatus::Playing;
    battle
}

fn create_cards(battle: &mut BattleData) {
    for _ in 0..30 {
        let identity = CardIdentity(Uuid::new_v4());
        battle.cards.create_card(identity, PlayerName::User, Zone::Deck, CardProperties {
            spark: Spark(2),
        });
        battle.cards.create_card(identity, PlayerName::Enemy, Zone::Deck, CardProperties {
            spark: Spark(2),
        });
    }
}
