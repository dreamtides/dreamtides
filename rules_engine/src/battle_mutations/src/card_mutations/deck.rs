use battle_queries::battle_card_queries::card_abilities;
use battle_state::battle::all_cards::CreatedCard;
use battle_state::battle::animation_data::AnimationStep;
use battle_state::battle::battle_animation::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardIdType, DeckCardId, HandCardId};
use battle_state::battle_cards::card_set::CardSet;
use battle_state::core::effect_source::EffectSource;
use core_data::identifiers::CardName;
use core_data::numerics::Energy;
use core_data::types::PlayerName;
use rand::seq::IteratorRandom;
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::card_mutations::{create_test_deck, move_card};
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
    if battle.animations.is_some() {
        draw_cards_with_animation(battle, source, player, count);
    } else {
        draw_cards_internal(battle, source, player, count);
    }
}

/// Internal implementation of draw_cards without animation support.
fn draw_cards_internal(
    battle: &mut BattleState,
    source: EffectSource,
    player: PlayerName,
    count: u32,
) {
    for _ in 0..count {
        draw_card_internal(battle, source, player, false);
    }
}

/// Implementation of draw_cards with animation support.
fn draw_cards_with_animation(
    battle: &mut BattleState,
    source: EffectSource,
    player: PlayerName,
    count: u32,
) {
    let pre_draw_snapshot = battle.logical_clone();
    let mut drawn_cards = Vec::new();

    for _ in 0..count {
        if let Some(card_id) = draw_card_internal(battle, source, player, false) {
            drawn_cards.push(card_id);
        }
    }

    if !drawn_cards.is_empty()
        && let Some(animations) = &mut battle.animations
    {
        animations.steps.push(AnimationStep {
            snapshot: pre_draw_snapshot,
            animation: BattleAnimation::DrawCards { player, cards: drawn_cards },
        });
    }
}

/// Adds a list of cards by name to a player's deck
pub fn add_cards(battle: &mut BattleState, player: PlayerName, cards: Vec<CardName>) {
    battle.cards.create_cards_in_deck(
        player,
        cards
            .into_iter()
            .map(|name| CreatedCard {
                name,
                can_play_restriction: card_abilities::query_by_name(name).can_play_restriction,
            })
            .collect(),
    );
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
        energy::gain(battle, player, source, Energy(1));
        let p = battle.turn_history.current_action_history.player_mut(player);
        p.hand_size_limit_exceeded = true;
        return None;
    }

    let Some(id) = random_element(battle.cards.deck(player), &mut battle.rng) else {
        create_test_deck::add(battle, player);
        return draw_card_internal(battle, source, player, with_animation);
    };
    if with_animation {
        battle.push_animation(|| BattleAnimation::DrawCards {
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
