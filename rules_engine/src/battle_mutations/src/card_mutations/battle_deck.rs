use std::sync::Arc;

use battle_queries::battle_card_queries::{card, card_abilities};
use battle_queries::{battle_trace, panic_with};
use battle_state::battle::ability_cache::AbilityCache;
use battle_state::battle::all_cards::CreatedCard;
use battle_state::battle::animation_data::AnimationStep;
use battle_state::battle::battle_animation::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardIdType, DeckCardId, HandCardId};
use battle_state::battle_cards::card_set::CardSet;
use battle_state::core::effect_source::EffectSource;
use battle_state::triggers::trigger::Trigger;
use core_data::identifiers::CardIdentity;
use core_data::numerics::Energy;
use core_data::types::PlayerName;
use rand::seq::IteratorRandom;
use rand_xoshiro::Xoshiro256PlusPlus;
use tabula_data::card_definition::CardDefinition;

use crate::card_mutations::move_card;
use crate::player_mutations::energy;

const HAND_SIZE_LIMIT: usize = 10;

/// Draw a card from `player`'s deck and put it into their hand. If their deck
/// is empty, it will be replaced with a new shuffled copy of the deck.
///
/// Returns the new [HandCardId] for the card if a card was drawn, or None if no
/// card was drawn (e.g. if the player's hand size limit was exceeded or the
/// draw was prevented by a game effect).
pub fn draw_card(
    battle: &mut BattleState,
    source: EffectSource,
    player: PlayerName,
) -> Option<HandCardId> {
    draw_card_internal(battle, source, player, true)
}

/// Draw a number of cards from `player`'s deck and put them into their hand.
pub fn draw_cards(battle: &mut BattleState, source: EffectSource, player: PlayerName, count: u32) {
    let should_animate = battle.animations.is_some();
    let pre_draw_snapshot = if should_animate {
        let mut snapshot = battle.logical_clone();
        while snapshot.cards.all_deck_cards(player).count() < count as usize {
            // Make sure the cards to be drawn are present in the deck. This
            // won't necessarily happen in exactly the same order, but we just
            // need them to visually be somewhere in the deck.
            add_deck_copy(&mut snapshot, player);
        }
        Some(snapshot)
    } else {
        None
    };

    let mut drawn_cards = Vec::new();
    for _ in 0..count {
        if let Some(card_id) = draw_card_internal(battle, source, player, false) {
            if should_animate {
                drawn_cards.push(card_id);
            }
        }
    }

    battle_trace!("Drew cards", battle, player, drawn_cards);
    if should_animate
        && !drawn_cards.is_empty()
        && let Some(animations) = &mut battle.animations
        && let Some(snapshot) = pre_draw_snapshot
    {
        animations.steps.push(AnimationStep {
            source,
            snapshot,
            animation: BattleAnimation::DrawCards { player, cards: drawn_cards },
        });
    }
}

/// Adds a copy of a player's quest deck to their battle deck.
pub fn add_deck_copy(battle: &mut BattleState, player: PlayerName) {
    let mut cards = Vec::new();
    for card in &battle.players.player(player).deck {
        let can_play_restriction = battle
            .ability_cache
            .try_get_by_identity(card.identity)
            .map(|list| list.can_play_restriction)
            .unwrap_or(None);
        cards.push(CreatedCard {
            identity: card.identity,
            can_play_restriction,
            base_energy_cost: card.definition.energy_cost,
            base_spark: card.definition.spark,
            card_type: card.definition.card_type,
            is_fast: card.definition.is_fast,
        });
    }
    battle.cards.create_cards_in_deck(player, cards);
}

/// Adds a list of cards to a player's deck
pub fn add_cards(battle: &mut BattleState, player: PlayerName, cards: Vec<CardDefinition>) {
    let mut pairs = Vec::new();
    let mut i = 0usize;
    loop {
        let id = CardIdentity(i);
        if let Some(list) = battle.ability_cache.try_get_by_identity(id) {
            let def = battle
                .ability_cache
                .try_get_definition(id)
                .expect("missing definition for identity");
            pairs.push((id, list, def));
            i += 1;
        } else {
            break;
        }
    }

    let mut created = Vec::new();
    for definition in cards {
        let identity = CardIdentity(i);
        i += 1;
        let ability_list = card_abilities::build_from_definition(identity, &definition);
        let can_play_restriction = ability_list.can_play_restriction;
        pairs.push((identity, Arc::new(ability_list), Arc::new(definition.clone())));
        created.push(CreatedCard {
            identity,
            can_play_restriction,
            base_energy_cost: definition.energy_cost,
            base_spark: definition.spark,
            card_type: definition.card_type,
            is_fast: definition.is_fast,
        });
    }

    battle.ability_cache = Arc::new(AbilityCache::from_pairs(pairs));
    battle.cards.create_cards_in_deck(player, created);
}

/// Ensures that at least `count` cards are known at the top of a player's deck.
///
/// Adds a `revealed_to_player_override` to the cards to indicate that they are
/// revealed to their owner while in the deck.
///
/// Any new cards that are required are picked randomly from the deck and
/// inserted below any existing known cards at the top of the deck.
///
/// Returns exactly `count` cards from the top of the deck.
pub fn realize_top_of_deck(
    battle: &mut BattleState,
    player: PlayerName,
    count: u32,
) -> Vec<DeckCardId> {
    let current_top_count = battle.cards.top_of_deck(player).len() as u32;
    let needed_cards = count.saturating_sub(current_top_count);

    for _ in 0..needed_cards {
        if let Some(card_id) =
            random_element(battle.cards.shuffled_into_deck(player), &mut battle.rng)
        {
            battle.cards.move_card_to_top_of_deck(player, card_id);
        } else {
            battle_trace!("Adding new deck for realize_top_of_deck", battle, player);
            add_deck_copy(battle, player);
            if let Some(card_id) =
                random_element(battle.cards.shuffled_into_deck(player), &mut battle.rng)
            {
                battle.cards.move_card_to_top_of_deck(player, card_id);
            } else {
                panic_with!("Failed to find card after adding new deck", battle, player);
            }
        }
    }

    let result =
        battle.cards.top_of_deck(player).iter().take(count as usize).copied().collect::<Vec<_>>();
    for card_id in &result {
        *card::get_mut(battle, *card_id).revealed_to_player_override.player_mut(player) = true;
    }
    result
}

fn draw_card_internal(
    battle: &mut BattleState,
    source: EffectSource,
    player: PlayerName,
    with_animation: bool,
) -> Option<HandCardId> {
    if battle.cards.hand(player).len() >= HAND_SIZE_LIMIT {
        // If a player exceeds the hand size limit, they instead gain 1
        // energy for each card they would have drawn.
        battle_trace!("Hand size limit exceeded", battle, player);
        energy::gain(battle, player, source, Energy(1));
        let p = battle.turn_history.current_action_history.player_mut(player);
        p.hand_size_limit_exceeded = true;
        return None;
    }

    let id = if let Some(top_card) = battle.cards.top_of_deck_mut(player).last() {
        *top_card
    } else if let Some(random_card) =
        random_element(battle.cards.shuffled_into_deck(player), &mut battle.rng)
    {
        random_card
    } else {
        battle_trace!("Adding new deck copy", battle, player);
        add_deck_copy(battle, player);
        battle.triggers.push(source, Trigger::DrewAllCardsInCopyOfDeck(player));
        return draw_card_internal(battle, source, player, with_animation);
    };

    if with_animation {
        battle_trace!("Drawing card", battle, player, id);
        battle.push_animation(source, || BattleAnimation::DrawCards {
            player,
            cards: vec![HandCardId(id.card_id())],
        });
    }

    Some(move_card::from_deck_to_hand(battle, source, player, id))
}

/// Returns a random element from the given set.
fn random_element(set: &CardSet<DeckCardId>, rng: &mut Xoshiro256PlusPlus) -> Option<DeckCardId> {
    set.iter().choose(rng)
}
