use battle_state::actions::battle_actions::DebugBattleAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardId, DeckCardId};
use battle_state::core::effect_source::EffectSource;
use core_data::types::PlayerName;
use tracing_macros::battle_trace;

use crate::card_mutations::{deck, move_card};

pub fn execute(battle: &mut BattleState, player: PlayerName, action: DebugBattleAction) {
    battle_trace!("Executing debug action", battle, player, action);
    let source = EffectSource::Game { controller: player };
    match action {
        DebugBattleAction::DrawCard(player_name) => {
            deck::draw_card(battle, source, player_name);
        }
        DebugBattleAction::SetEnergy(player_name, energy) => {
            battle.players.player_mut(player_name).current_energy = energy;
        }
        DebugBattleAction::AddCardToHand(player_name, card_name) => {
            let card_count = battle.cards.all_cards().count();
            deck::add_cards(battle, player_name, vec![card_name]);
            let new_card_id = DeckCardId(CardId(card_count));
            move_card::from_deck_to_hand(battle, source, player_name, new_card_id);
        }
        DebugBattleAction::SetPoints(player_name, points) => {
            battle.players.player_mut(player_name).points = points;
        }
        DebugBattleAction::SetProducedEnergy(player_name, energy) => {
            battle.players.player_mut(player_name).produced_energy = energy;
        }
        DebugBattleAction::SetSparkBonus(player_name, spark) => {
            battle.players.player_mut(player_name).spark_bonus = spark;
        }
        DebugBattleAction::AddCardToBattlefield(player_name, card_name) => {
            let card_count = battle.cards.all_cards().count();
            deck::add_cards(battle, player_name, vec![card_name]);
            let new_card_id = DeckCardId(CardId(card_count));
            move_card::from_deck_to_battlefield(battle, source, player_name, new_card_id);
        }
        DebugBattleAction::AddCardToVoid(player_name, card_name) => {
            let card_count = battle.cards.all_cards().count();
            deck::add_cards(battle, player_name, vec![card_name]);
            let new_card_id = DeckCardId(CardId(card_count));
            move_card::from_deck_to_void(battle, source, player_name, new_card_id);
        }
    }
}
