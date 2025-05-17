use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::HandCardId;
use battle_state::battle_cards::card_set::CardSet;
use core_data::card_types::CardType;
use core_data::identifiers::CardName;
use core_data::numerics::Energy;
use core_data::types::PlayerName;

use crate::battle_card_queries::card_properties;

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

        if !has_legal_targets(battle, player, battle.cards.name(card_id)) {
            continue;
        }

        if !has_legal_additional_costs(battle, player, battle.cards.name(card_id), cost) {
            continue;
        }

        legal_cards.push(card_id);
    }

    legal_cards
}

fn has_legal_targets(battle: &BattleState, controller: PlayerName, card: CardName) -> bool {
    match card {
        CardName::MinstrelOfFallingLight => true,
        CardName::Immolate => !battle.cards.battlefield(controller.opponent()).is_empty(),
        CardName::RippleOfDefiance => battle
            .cards
            .stack_set(controller.opponent())
            .iter()
            .any(|id| card_properties::card_type(battle, id) == CardType::Event),
        CardName::Abolish => !battle.cards.stack_set(controller.opponent()).is_empty(),
        CardName::Dreamscatter => true,
    }
}

fn has_legal_additional_costs(
    battle: &BattleState,
    controller: PlayerName,
    card: CardName,
    paid: Energy,
) -> bool {
    match card {
        CardName::Dreamscatter => battle.players.player(controller).current_energy > paid,
        _ => true,
    }
}
