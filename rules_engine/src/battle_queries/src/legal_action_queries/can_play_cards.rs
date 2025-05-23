use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardIdType, HandCardId};
use battle_state::battle_cards::ability_list::CanPlayRestriction;
use core_data::numerics::Energy;
use core_data::types::PlayerName;

use crate::battle_card_queries::card_properties;
use crate::legal_action_queries::{has_legal_additional_costs, has_legal_targets};

/// Whether only cards with the `fast` property should be returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FastOnly {
    Yes,
    No,
}

/// Returns the set of cards in a player's hand that are playable based on their
/// own internal state & costs. If `fast_only` is set, only cards with the
/// "fast" property are returned.
///
/// This does *not* check whether it is legal to play cards in the larger
/// current battle state, e.g. whether it is the player's turn.
pub fn from_hand(battle: &BattleState, player: PlayerName, fast_only: FastOnly) -> Vec<HandCardId> {
    // Tested using a bitset here, but Vec is consistently faster (4% overall
    // benchmark improvement) possibly due to better random element selection.
    let mut legal_cards = Vec::new();
    for card_id in battle.cards.hand(player) {
        if fast_only == FastOnly::Yes && !card_properties::is_fast(battle, card_id) {
            continue;
        }

        let Some(cost) = card_properties::cost(battle, card_id) else {
            continue;
        };

        if cost > battle.players.player(player).current_energy {
            continue;
        }

        let meets = match battle.cards.card(card_id).can_play_restriction {
            Some(restriction) => meets_restriction(battle, player, restriction, cost),
            None => {
                // No fast version of the 'can play' restriction, check all card
                // abilities.
                has_legal_targets::for_event(battle, player, card_id.card_id())
                    && has_legal_additional_costs::for_event(
                        battle,
                        player,
                        card_id.card_id(),
                        cost,
                    )
            }
        };

        if meets {
            legal_cards.push(card_id);
        }
    }

    legal_cards
}

fn meets_restriction(
    battle: &BattleState,
    controller: PlayerName,
    restriction: CanPlayRestriction,
    energy_cost: Energy,
) -> bool {
    match restriction {
        CanPlayRestriction::Unrestricted => true,
        CanPlayRestriction::EnemyCharacter => {
            !battle.cards.battlefield(controller.opponent()).is_empty()
        }
        CanPlayRestriction::EnemyStackCard => {
            !battle.cards.stack_set(controller.opponent()).is_empty()
        }
        CanPlayRestriction::EnemyStackCardOfType(card_type) => battle
            .cards
            .stack_set(controller.opponent())
            .iter()
            .any(|id| card_properties::card_type(battle, id) == card_type),
        CanPlayRestriction::AdditionalEnergyAvailable(required_energy) => {
            battle.players.player(controller).current_energy - energy_cost >= required_energy
        }
    }
}
