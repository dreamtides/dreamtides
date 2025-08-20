use battle_queries::battle_trace;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::{LegalActions, PrimaryLegalAction};
use battle_state::actions::battle_actions::BattleAction;
use battle_state::actions::debug_battle_action::DebugBattleAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardId, DeckCardId, HandCardId};
use battle_state::core::effect_source::EffectSource;
use core_data::identifiers::CardName;
use core_data::types::PlayerName;
use quest_state::quest::card_descriptor;

use crate::actions::apply_battle_action;
use crate::card_mutations::{battle_deck, move_card};

pub fn execute(battle: &mut BattleState, player: PlayerName, action: DebugBattleAction) {
    battle_trace!("Executing debug action", battle, player, action);
    let source = EffectSource::Game { controller: player };
    match action {
        DebugBattleAction::DrawCard { player: player_name } => {
            battle_deck::draw_card(battle, source, player_name);
        }
        DebugBattleAction::SetEnergy { player: player_name, energy } => {
            battle.players.player_mut(player_name).current_energy = energy;
        }
        DebugBattleAction::SetPoints { player: player_name, points } => {
            battle.players.player_mut(player_name).points = points;
        }
        DebugBattleAction::SetProducedEnergy { player: player_name, energy } => {
            battle.players.player_mut(player_name).produced_energy = energy;
        }
        DebugBattleAction::SetSparkBonus { player: player_name, spark } => {
            battle.players.player_mut(player_name).spark_bonus = spark;
        }
        DebugBattleAction::AddCardToHand { player: player_name, card: card_name } => {
            add_to_hand(battle, player_name, source, card_name);
        }
        DebugBattleAction::AddCardToBattlefield { player: player_name, card: card_name } => {
            let card_count = battle.cards.all_cards().count();
            battle_deck::add_cards(battle, player_name, vec![card_descriptor::get_base_identity(
                card_name,
            )]);
            let new_card_id = DeckCardId(CardId(card_count));
            move_card::from_deck_to_battlefield(battle, source, player_name, new_card_id);
        }
        DebugBattleAction::AddCardToVoid { player: player_name, card: card_name } => {
            let card_count = battle.cards.all_cards().count();
            battle_deck::add_cards(battle, player_name, vec![card_descriptor::get_base_identity(
                card_name,
            )]);
            let new_card_id = DeckCardId(CardId(card_count));
            move_card::from_deck_to_void(battle, source, player_name, new_card_id);
        }
        DebugBattleAction::MoveHandToDeck { player: player_name } => {
            let hand_cards: Vec<HandCardId> = battle.cards.hand(player_name).iter().collect();
            for card_id in hand_cards {
                move_card::from_hand_to_deck(battle, source, player_name, card_id);
            }
        }
        DebugBattleAction::SetCardsRemainingInDeck { player: player_name, cards: target_count } => {
            let deck_cards: Vec<DeckCardId> = battle.cards.all_deck_cards(player_name).collect();
            let current_count = deck_cards.len();
            if current_count > target_count {
                let cards_to_move = current_count - target_count;
                for card_id in deck_cards.into_iter().take(cards_to_move) {
                    move_card::from_deck_to_void(battle, source, player_name, card_id);
                }
            }
        }
        DebugBattleAction::OpponentPlayCard { card: card_name } => {
            let card_id = add_to_hand(battle, player.opponent(), source, card_name);
            apply_battle_action::execute_without_tracking_history(
                battle,
                player.opponent(),
                BattleAction::PlayCardFromHand(card_id),
            );
            make_prompt_choices(battle, player.opponent());
        }
        DebugBattleAction::OpponentContinue => {
            let legal = legal_actions::compute(battle, player.opponent());
            let action = get_continue_action(&legal);
            apply_battle_action::execute_without_tracking_history(
                battle,
                player.opponent(),
                action,
            );
        }
    }
}

fn add_to_hand(
    battle: &mut BattleState,
    player: PlayerName,
    source: EffectSource,
    card_name: CardName,
) -> HandCardId {
    let card_count = battle.cards.all_cards().count();
    battle_deck::add_cards(battle, player, vec![card_descriptor::get_base_identity(card_name)]);
    let new_card_id = DeckCardId(CardId(card_count));
    move_card::from_deck_to_hand(battle, source, player, new_card_id)
}

fn get_continue_action(actions: &LegalActions) -> BattleAction {
    let LegalActions::Standard { actions } = actions else {
        panic!("Expected standard legal actions");
    };
    match actions.primary {
        PrimaryLegalAction::PassPriority => BattleAction::PassPriority,
        PrimaryLegalAction::EndTurn => BattleAction::EndTurn,
        PrimaryLegalAction::StartNextTurn => BattleAction::StartNextTurn,
    }
}

fn make_prompt_choices(battle: &mut BattleState, opponent: PlayerName) {
    while let Some(current_prompt) = battle.prompts.front()
        && current_prompt.player == opponent
    {
        let legal = legal_actions::compute(battle, current_prompt.player);
        let all_actions = legal.all();
        let Some(random) = all_actions.first() else {
            break;
        };
        apply_battle_action::execute_without_tracking_history(battle, opponent, *random);
    }
}
