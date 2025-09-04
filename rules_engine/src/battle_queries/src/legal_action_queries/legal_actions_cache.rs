use std::sync::Arc;

use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CharacterId, HandCardId, VoidCardId};
use battle_state::battle_cards::card_set::CardSet;
use battle_state::battle_player::legal_actions_cache_data::{
    LegalActionsCacheData, LegalActionsForAvailableEnergy,
};
use battle_state::battle_player::player_map::PlayerMap;
use core_data::numerics::Energy;
use core_data::types::PlayerName;

use crate::battle_card_queries::card_properties;
use crate::legal_action_queries::can_play_cards::FastOnly;
use crate::legal_action_queries::{can_activate_abilities, can_play_cards};

pub fn populate(battle: &mut BattleState) {
    let one = for_player(battle, PlayerName::One, 16);
    let two = for_player(battle, PlayerName::Two, 16);
    battle.legal_actions_cache = Arc::new(PlayerMap { one, two });
}

#[inline]
pub fn play_card_candidates(
    battle: &BattleState,
    player: PlayerName,
    fast_only: FastOnly,
) -> CardSet<HandCardId> {
    let energy = battle.players.player(player).current_energy;
    let cache = &battle.legal_actions_cache.player(player).actions_for_available_energy;
    if energy.0 as usize >= cache.len() {
        return play_card_candidates_cache_miss(battle, player, energy, fast_only);
    }
    match fast_only {
        FastOnly::Yes => cache[energy.0 as usize].play_from_hand_fast.clone(),
        FastOnly::No => cache[energy.0 as usize].play_from_hand.clone(),
    }
}

#[inline]
pub fn play_from_void_candidates(
    battle: &BattleState,
    player: PlayerName,
    fast_only: FastOnly,
) -> CardSet<VoidCardId> {
    let energy = battle.players.player(player).current_energy;
    let cache = &battle.legal_actions_cache.player(player).actions_for_available_energy;
    if energy.0 as usize >= cache.len() {
        return play_from_void_candidates_cache_miss(battle, player, energy, fast_only);
    }
    match fast_only {
        FastOnly::Yes => cache[energy.0 as usize].play_from_void_fast.clone(),
        FastOnly::No => cache[energy.0 as usize].play_from_void.clone(),
    }
}

#[inline]
pub fn activate_ability_candidates(
    battle: &BattleState,
    player: PlayerName,
    fast_only: FastOnly,
) -> CardSet<CharacterId> {
    let energy = battle.players.player(player).current_energy;
    let cache = &battle.legal_actions_cache.player(player).actions_for_available_energy;
    if energy.0 as usize >= cache.len() {
        return activate_ability_candidates_cache_miss(battle, player, energy, fast_only);
    }
    match fast_only {
        FastOnly::Yes => cache[energy.0 as usize].activate_abilities_fast.clone(),
        FastOnly::No => cache[energy.0 as usize].activate_abilities.clone(),
    }
}

fn for_player(battle: &BattleState, player: PlayerName, max_energy: u32) -> LegalActionsCacheData {
    let mut result = vec![];
    for e in 0..max_energy {
        result.push(for_available_energy(battle, player, Energy(e)));
    }
    LegalActionsCacheData { actions_for_available_energy: result }
}

fn for_available_energy(
    battle: &BattleState,
    player: PlayerName,
    energy: Energy,
) -> LegalActionsForAvailableEnergy {
    let mut result = LegalActionsForAvailableEnergy::default();
    for card_id in battle.cards.all_cards() {
        if battle.cards[card_id].owner != player {
            continue;
        }

        if card_properties::converted_energy_cost(battle, card_id) <= energy {
            result.play_from_hand.insert(HandCardId(card_id));
            if card_properties::is_fast(battle, card_id) {
                result.play_from_hand_fast.insert(HandCardId(card_id));
            }
        }

        if let Some(play_from_void) =
            can_play_cards::can_play_from_void_energy_cost(battle, VoidCardId(card_id))
            && play_from_void.cost <= energy
        {
            result.play_from_void.insert(VoidCardId(card_id));
            if card_properties::is_fast(battle, card_id) {
                result.play_from_void_fast.insert(VoidCardId(card_id));
            }
        }

        if let Some(ability_cost) = can_activate_abilities::activated_ability_energy_cost(
            battle,
            CharacterId(card_id),
            FastOnly::No,
        ) && ability_cost <= energy
        {
            result.activate_abilities.insert(CharacterId(card_id));
        }

        if let Some(ability_cost) = can_activate_abilities::activated_ability_energy_cost(
            battle,
            CharacterId(card_id),
            FastOnly::Yes,
        ) && ability_cost <= energy
        {
            result.activate_abilities_fast.insert(CharacterId(card_id));
        }
    }

    result
}

#[cold]
fn play_card_candidates_cache_miss(
    battle: &BattleState,
    player: PlayerName,
    energy: Energy,
    fast_only: FastOnly,
) -> CardSet<HandCardId> {
    let cache_data = for_available_energy(battle, player, energy);
    match fast_only {
        FastOnly::Yes => cache_data.play_from_hand_fast,
        FastOnly::No => cache_data.play_from_hand,
    }
}

#[cold]
fn play_from_void_candidates_cache_miss(
    battle: &BattleState,
    player: PlayerName,
    energy: Energy,
    fast_only: FastOnly,
) -> CardSet<VoidCardId> {
    let cache_data = for_available_energy(battle, player, energy);
    match fast_only {
        FastOnly::Yes => cache_data.play_from_void_fast,
        FastOnly::No => cache_data.play_from_void,
    }
}

#[cold]
fn activate_ability_candidates_cache_miss(
    battle: &BattleState,
    player: PlayerName,
    energy: Energy,
    fast_only: FastOnly,
) -> CardSet<CharacterId> {
    let cache_data = for_available_energy(battle, player, energy);
    match fast_only {
        FastOnly::Yes => cache_data.activate_abilities_fast,
        FastOnly::No => cache_data.activate_abilities,
    }
}
