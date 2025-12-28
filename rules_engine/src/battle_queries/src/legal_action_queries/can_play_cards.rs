use std::hash::Hash;

use ability_data::static_ability::StandardStaticAbility;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{AbilityId, CardId, CardIdType, HandCardId, VoidCardId};
use battle_state::battle_cards::ability_list::CanPlayRestriction;
use battle_state::battle_cards::card_set::CardSet;
use core_data::card_types::CardType;
use core_data::identifiers::AbilityNumber;
use core_data::numerics::Energy;
use core_data::types::PlayerName;

use crate::battle_card_queries::{card, card_properties};
use crate::card_ability_queries::effect_queries;
use crate::legal_action_queries::{
    has_legal_additional_costs, has_legal_targets, legal_actions_cache, legal_modal_effect_choices,
};
use crate::panic_with;

/// Whether only cards with the `fast` property should be returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FastOnly {
    Yes,
    No,
}

/// A card that can be played from the void via a given ability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CanPlayFromVoid {
    pub card_id: VoidCardId,
    pub via_ability_id: AbilityId,
}

/// Cost and ability ID of a card that can be played from the void.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FromVoidWithCost {
    pub cost: Energy,
    pub via_ability_id: AbilityId,
}

/// Returns the set of cards in a player's hand that are playable based on their
/// own internal state & costs. If `fast_only` is set, only cards with the
/// "fast" property are returned.
///
/// This does *not* check whether it is legal to play cards in the larger
/// current battle state, e.g. whether it is the player's turn.
pub fn from_hand(
    battle: &BattleState,
    player: PlayerName,
    fast_only: FastOnly,
) -> CardSet<HandCardId> {
    let mut candidates = legal_actions_cache::play_card_candidates(battle, player, fast_only);
    candidates.intersect_with(battle.cards.hand(player));

    let mut legal_cards = CardSet::new();
    for card_id in &candidates {
        let energy_cost = card_properties::converted_energy_cost(battle, card_id);
        add_if_meets_restriction(battle, player, card_id, energy_cost, &mut legal_cards);
    }
    legal_cards
}

/// Returns the set of cards in a player's void that are playable based on their
/// own internal state & costs. If `fast_only` is set, only cards with the
/// "fast" property are returned.
///
/// This does *not* check whether it is legal to play cards in the larger
/// current battle state, e.g. whether it is the player's turn.
pub fn from_void(
    battle: &BattleState,
    player: PlayerName,
    fast_only: FastOnly,
) -> CardSet<VoidCardId> {
    let mut candidates = legal_actions_cache::play_from_void_candidates(battle, player, fast_only);
    candidates.intersect_with(battle.cards.void(player));
    let mut legal_cards = CardSet::new();

    for card_id in &candidates {
        let Some(from_void_with_cost) = can_play_from_void_energy_cost(battle, card_id) else {
            continue;
        };

        add_if_meets_restriction(
            battle,
            player,
            card_id,
            from_void_with_cost.cost,
            &mut legal_cards,
        );
    }

    legal_cards
}

/// Returns the energy cost of playing a card from the void with a given
/// ability.
///
/// Panics if the card cannot be played from the void or has no energy cost.
pub fn play_from_void_energy_cost(
    battle: &BattleState,
    card_id: VoidCardId,
    ability_id: AbilityId,
) -> Energy {
    let abilities = card::ability_list(battle, ability_id.card_id);
    let cost = abilities
        .static_abilities
        .iter()
        .find(|a| a.ability_number == ability_id.ability_number)
        .and_then(|ability_data| {
            can_play_from_void_with_static_ability(
                battle,
                card_id,
                ability_id.ability_number,
                ability_data.ability.standard_static_ability(),
            )
            .map(|c| c.cost)
        });

    if let Some(cost) = cost {
        cost
    } else {
        panic_with!("Card has no void energy cost", battle, card_id, ability_id);
    }
}

/// Check if a card can be played from the void, and returns the energy cost
/// of playing it if it can be. If there are multiple abilities that can be
/// used to play the card, returns the one with the lowest cost. This does not
/// e.g. validate that the player has sufficient energy to pay this cost.
///
/// Other costs are handled in the 'meets_restriction' step.
pub fn can_play_from_void_energy_cost(
    battle: &BattleState,
    card_id: VoidCardId,
) -> Option<FromVoidWithCost> {
    card::ability_list(battle, card_id)
        .static_abilities
        .iter()
        .filter_map(|ability_data| {
            can_play_from_void_with_static_ability(
                battle,
                card_id,
                ability_data.ability_number,
                ability_data.ability.standard_static_ability(),
            )
        })
        .min()
}

impl From<CanPlayFromVoid> for CardId {
    fn from(value: CanPlayFromVoid) -> Self {
        value.card_id.card_id()
    }
}

/// Returns the energy cost of playing a card from the void with a given static
/// ability, if it can be played.
fn can_play_from_void_with_static_ability(
    battle: &BattleState,
    card_id: VoidCardId,
    ability_number: AbilityNumber,
    ability: &StandardStaticAbility,
) -> Option<FromVoidWithCost> {
    let ability_id = AbilityId { card_id: card_id.card_id(), ability_number };
    match ability {
        StandardStaticAbility::PlayFromVoid(play) => {
            let cost = match play.energy_cost {
                Some(energy_cost) => energy_cost,
                // Cards with no energy cost (e.g. modal cards) are treated as
                // having a cost of 0
                None => card_properties::converted_energy_cost(battle, card_id),
            };
            Some(FromVoidWithCost { cost, via_ability_id: ability_id })
        }
        StandardStaticAbility::PlayOnlyFromVoid => {
            let cost = card_properties::converted_energy_cost(battle, card_id);
            Some(FromVoidWithCost { cost, via_ability_id: ability_id })
        }
        _ => None,
    }
}

/// Adds a card to the collection of legal cards if it meets the restriction.
fn add_if_meets_restriction<TId>(
    battle: &BattleState,
    player: PlayerName,
    card_id: TId,
    cost: Energy,
    legal_cards: &mut CardSet<TId>,
) where
    TId: Into<CardId> + Copy + CardIdType,
{
    let card_id_generic: CardId = card_id.into();

    let meets = match card::get(battle, card_id_generic).can_play_restriction {
        Some(restriction) => meets_restriction(battle, player, restriction, cost),
        None => {
            // No fast version of the 'can play' restriction, check all card
            // abilities.
            has_legal_targets::for_event(battle, player, card_id_generic.card_id())
                && has_legal_additional_costs::for_event(
                    battle,
                    player,
                    card_id_generic.card_id(),
                    cost,
                )
                && legal_modal_effect_choices::event_has_legal_choices(
                    battle,
                    player,
                    card_id_generic.card_id(),
                )
        }
    };

    if meets {
        legal_cards.insert(card_id);
    }
}

fn meets_restriction(
    battle: &BattleState,
    controller: PlayerName,
    restriction: CanPlayRestriction,
    energy_cost: Energy,
) -> bool {
    match restriction {
        CanPlayRestriction::Unrestricted => true,
        CanPlayRestriction::EnemyCharacterOnBattlefield => {
            !battle.cards.battlefield(controller.opponent()).is_empty()
        }
        CanPlayRestriction::DissolveEnemyCharacter => {
            let prevent = effect_queries::prevent_dissolved_set(battle);
            battle.cards.battlefield(controller.opponent()).iter().any(|c| !prevent.contains(c))
        }
        CanPlayRestriction::EnemyCardOnStack => {
            !battle.cards.stack_set(controller.opponent()).is_empty()
        }
        CanPlayRestriction::EnemyEventCardOnStack => battle
            .cards
            .stack_set(controller.opponent())
            .iter()
            .any(|id| card_properties::card_type(battle, id) == CardType::Event),
        CanPlayRestriction::EnemyCharacterCardOnStack => battle
            .cards
            .stack_set(controller.opponent())
            .iter()
            .any(|id| card_properties::card_type(battle, id) == CardType::Character),
        CanPlayRestriction::AdditionalEnergyAvailable(required_energy) => {
            battle.players.player(controller).current_energy - energy_cost >= required_energy
        }
    }
}
