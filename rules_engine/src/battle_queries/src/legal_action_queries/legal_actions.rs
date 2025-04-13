use action_data::battle_action::BattleAction;
use battle_data::battle::battle_data::BattleData;
use core_data::types::PlayerName;
use tracing::instrument;

#[derive(Debug, Default, Clone, Copy)]
pub struct LegalActions {
    /// Include 'interface only' actions in the response which don't affect the
    /// game, e.g. removing a declared attacker.
    ///
    /// These are excluded from AI agent options in order to prevent infinite
    /// loops of game actions which do not progress the game state.
    pub for_human_player: bool,
}

/// List of all legal actions the named player can take in the
/// current battle state.
#[instrument(name = "legal_actions_compute", level = "trace", skip(battle))]
pub fn compute(
    battle: &BattleData,
    player: PlayerName,
    _options: LegalActions,
) -> Vec<BattleAction> {
    let is_active_player = battle.turn.active_player == player;
    let player_data = battle.player(player);
    let mut actions = Vec::new();

    if is_active_player {
        actions.push(BattleAction::EndTurn);
        for card in battle.cards.hand_cards(player) {
            if let Some(cost) = card.properties.cost {
                if cost <= player_data.current_energy {
                    actions.push(BattleAction::PlayCard(card.id.card_identifier_for_display()));
                }
            }
        }
    }

    actions
}
