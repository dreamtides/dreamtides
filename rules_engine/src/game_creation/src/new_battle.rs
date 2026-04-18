use std::sync::Arc;

use battle_mutations::card_mutations::move_card;
use battle_queries::battle_card_queries::card;
use battle_queries::battle_trace;
use battle_state::battle::animation_data::AnimationData;
use battle_state::battle::battle_history::BattleHistory;
use battle_state::battle::battle_rules_config::BalanceMode;
use battle_state::battle::battle_state::{BattleState, RequestContext};
use battle_state::battle_cards::dreamwell_data::Dreamwell;
use battle_state::battle_player::battle_player_state::CreateBattlePlayer;
use battle_state::battle_trace::battle_tracing::BattleTracing;
use battle_state::core::effect_source::EffectSource;
use core_data::identifiers::BattleId;
use core_data::types::PlayerName;
use tabula_data::tabula::Tabula;

use crate::new_test_battle;

/// Creates a new battle and starts it using a given seed and
/// [`CreateBattlePlayer`] specification.
#[expect(clippy::too_many_arguments)]
pub fn create_and_start(
    battle_id: BattleId,
    tabula: Arc<Tabula>,
    seed: u64,
    dreamwell: Dreamwell,
    player_one: CreateBattlePlayer,
    player_two: CreateBattlePlayer,
    request_context: RequestContext,
    first_player: PlayerName,
    front_row_size: Option<usize>,
    back_row_size: Option<usize>,
    opening_hand_card_name: Option<&str>,
) -> BattleState {
    let mut battle = new_test_battle::create_and_start(
        battle_id,
        tabula,
        seed,
        dreamwell,
        player_one,
        player_two,
        request_context,
        first_player,
        front_row_size,
        back_row_size,
        BalanceMode::FourFiveCards,
    );
    battle.animations = Some(AnimationData::default());
    battle.tracing = Some(BattleTracing::default());
    battle.action_history = Some(BattleHistory::default());
    if let Some(displayed_name) = opening_hand_card_name {
        swap_opening_hand_card(&mut battle, PlayerName::One, displayed_name);
    }
    battle
}

fn swap_opening_hand_card(battle: &mut BattleState, player: PlayerName, displayed_name: &str) {
    if battle
        .cards
        .hand(player)
        .iter()
        .any(|id| card::get_definition(battle, id).displayed_name == displayed_name)
    {
        battle_trace!(
            "Requested opening hand card already in hand",
            battle,
            player,
            displayed_name
        );
        return;
    }

    let Some(deck_card_id) = battle
        .cards
        .all_deck_cards(player)
        .find(|id| card::get_definition(battle, *id).displayed_name == displayed_name)
    else {
        battle_trace!(
            "Requested opening hand card not found in deck",
            battle,
            player,
            displayed_name
        );
        return;
    };

    let source = EffectSource::Game { controller: player };
    let Some(hand_card_id) = battle.cards.hand(player).iter().next() else {
        move_card::from_deck_to_hand(battle, source, player, deck_card_id);
        battle_trace!(
            "Added requested opening hand card to empty hand",
            battle,
            player,
            displayed_name
        );
        return;
    };

    move_card::from_hand_to_deck(battle, source, player, hand_card_id);
    move_card::from_deck_to_hand(battle, source, player, deck_card_id);
    battle_trace!("Swapped requested opening hand card into hand", battle, player, displayed_name);
}
