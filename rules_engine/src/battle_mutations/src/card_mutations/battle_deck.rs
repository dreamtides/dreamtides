use std::sync::Arc;

use battle_queries::battle_card_queries::{card, card_abilities};
use battle_queries::legal_action_queries::legal_actions_cache;
use battle_queries::{battle_trace, panic_with};
use battle_state::battle::all_cards::CreatedCard;
use battle_state::battle::animation_data::AnimationStep;
use battle_state::battle::battle_animation_data::BattleAnimation;
use battle_state::battle::battle_card_definitions::{
    BattleCardDefinitions, BattleCardDefinitionsCard,
};
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{BattleDeckCardId, CardIdType, HandCardId};
use battle_state::battle_cards::card_set::CardSet;
use battle_state::core::effect_source::EffectSource;
use battle_state::triggers::trigger::Trigger;
use core_data::numerics::Energy;
use core_data::types::PlayerName;
use rand::Rng;
use rand_xoshiro::Xoshiro256PlusPlus;
use tabula_data::card_definitions::card_definition::CardDefinition;

use crate::card_mutations::move_card;
use crate::player_mutations::energy;

const HAND_SIZE_LIMIT: usize = 10;

/// Draw a card from `player`'s deck and put it into their hand. If their deck
/// is empty, all cards from the void will be shuffled back into the deck.
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
        let total_available =
            snapshot.cards.all_deck_cards(player).count() + snapshot.cards.void(player).len();
        if total_available < count as usize {
            panic_with!(
                "Cannot draw enough cards from deck and void combined",
                battle,
                count,
                total_available,
                player
            );
        }
        // Ensure void cards are shuffled into deck for animation purposes
        while snapshot.cards.all_deck_cards(player).count() < count as usize {
            shuffle_void_into_deck(&mut snapshot, player);
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
        let ability_list = battle.card_definitions.get_ability_list(*card);
        let definition = battle.card_definitions.get_definition(*card);
        let can_play_restriction = ability_list.can_play_restriction;
        cards.push(CreatedCard {
            identity: *card,
            can_play_restriction,
            base_energy_cost: definition.energy_cost,
            base_spark: definition.spark,
            card_type: definition.card_type,
            is_fast: definition.is_fast,
        });
    }
    battle.cards.create_cards_in_deck(player, cards);
}

/// Adds a list of cards to a player's deck
pub fn debug_add_cards(battle: &mut BattleState, player: PlayerName, cards: &[CardDefinition]) {
    let mut new_cards = Vec::new();
    let mut new_quest = (*battle.players.player(player).quest).clone();
    let mut new_deck = battle.players.player(player).quest.deck.clone();

    for definition in cards {
        let def_arc = Arc::new(definition.clone());
        let list = Arc::new(card_abilities::build_from_definition(&def_arc));
        let quest_card_id = new_deck.push_card_and_get_id(definition.clone());
        new_cards.push(BattleCardDefinitionsCard {
            ability_list: list,
            definition: def_arc,
            quest_deck_card_id: quest_card_id,
            owner: player,
        });
    }

    new_quest.deck = new_deck;
    battle.players.player_mut(player).quest = Arc::new(new_quest);
    let response = BattleCardDefinitions::append(&battle.card_definitions, new_cards);
    battle.card_definitions = Arc::new(response.cache);

    battle.cards.create_cards_in_deck(player, response.created);

    legal_actions_cache::populate(battle);
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum SetRevealedToPlayer {
    Yes,
    No,
}

/// Ensures that at least `count` cards are known at the top of a player's deck.
///
/// If [SetRevealedToPlayer] is `Yes`, adds a `revealed_to_player_override` to
/// the cards to indicate that they are revealed to their owner while in the
/// deck.
///
/// Any new cards that are required are picked randomly from the deck and
/// inserted below any existing known cards at the top of the deck. A new deck
/// is created if there are no cards left in the deck.
///
/// Returns exactly `count` cards from the top of the deck.
pub fn realize_top_of_deck(
    battle: &mut BattleState,
    player: PlayerName,
    count: u32,
    set_revealed_to_player: SetRevealedToPlayer,
) -> Vec<BattleDeckCardId> {
    let current_top_count = battle.cards.top_of_deck(player).len() as u32;
    let needed_cards = count.saturating_sub(current_top_count);

    for _ in 0..needed_cards {
        if let Some(card_id) =
            random_element(battle.cards.shuffled_into_deck(player), &mut battle.rng)
        {
            battle.cards.move_card_to_top_of_deck(player, card_id);
        } else {
            // Deck is empty, try to shuffle void back into deck
            if !battle.cards.void(player).is_empty() {
                battle_trace!("Shuffling void into deck for realize_top_of_deck", battle, player);
                shuffle_void_into_deck(battle, player);
                if let Some(card_id) =
                    random_element(battle.cards.shuffled_into_deck(player), &mut battle.rng)
                {
                    battle.cards.move_card_to_top_of_deck(player, card_id);
                } else {
                    panic_with!(
                        "Failed to find card after shuffling void into deck",
                        battle,
                        player
                    );
                }
            } else {
                panic_with!(
                    "Cannot realize top of deck: both deck and void are empty",
                    battle,
                    player
                );
            }
        }
    }

    let result =
        battle.cards.top_of_deck(player).iter().take(count as usize).copied().collect::<Vec<_>>();
    if set_revealed_to_player == SetRevealedToPlayer::Yes {
        for card_id in &result {
            *card::get_mut(battle, *card_id).revealed_to_player_override.player_mut(player) = true;
        }
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
        // Deck is empty, try to shuffle void back into deck
        if !battle.cards.void(player).is_empty() {
            battle_trace!("Shuffling void into deck", battle, player);
            shuffle_void_into_deck(battle, player);
            battle.triggers.push(source, Trigger::DrewAllCardsInCopyOfDeck(player));
            return draw_card_internal(battle, source, player, with_animation);
        } else {
            // Both deck and void are empty, cannot draw
            panic_with!("Cannot draw card: both deck and void are empty", battle, player);
        }
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

fn shuffle_void_into_deck(battle: &mut BattleState, player: PlayerName) {
    battle.cards.shuffle_void_into_deck(player);
}

/// Returns a random element from the given set.
fn random_element(
    set: &CardSet<BattleDeckCardId>,
    rng: &mut Xoshiro256PlusPlus,
) -> Option<BattleDeckCardId> {
    if set.is_empty() {
        return None;
    }

    let len = set.len();
    let random_index = rng.random_range(0..len);
    set.get_at_index(random_index)
}
