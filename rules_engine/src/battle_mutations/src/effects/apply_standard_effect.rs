use ability_data::cost::Cost;
use ability_data::quantity_expression_data::QuantityExpression;
use ability_data::standard_effect::StandardEffect;
use battle_queries::battle_card_queries::{card, card_properties};
use battle_queries::battle_player_queries::quantity_expression;
use battle_queries::battle_trace;
use battle_state::battle::battle_animation_data::{BattleAnimation, TargetedEffectName};
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardIdType;
use battle_state::battle_cards::battle_card_state::CardObjectId;
use battle_state::battle_cards::card_set::CardSet;
use battle_state::battle_cards::stack_card_state::EffectTargets;
use battle_state::core::effect_source::EffectSource;
use battle_state::core::should_animate::ShouldAnimate;
use battle_state::prompt_types::prompt_data::{
    PromptConfiguration, PromptData, PromptType, SelectDeckCardOrderPrompt,
};
use core_data::numerics::{Energy, Points, Spark};
use core_data::types::PlayerName;

use crate::card_mutations::battle_deck::SetRevealedToPlayer;
use crate::card_mutations::{battle_deck, counterspell, move_card, spark};
use crate::character_mutations::dissolve;
use crate::effects::apply_effect::EffectWasApplied;
use crate::effects::{counterspell_unless_pays_cost, discard_cards, pay_cost, targeting};
use crate::player_mutations::{energy, points};

/// Applies a [StandardEffect] to the given [BattleState].
///
/// You should almost always use `apply_effect::execute()` instead of calling
/// this directly.
pub fn apply(
    battle: &mut BattleState,
    source: EffectSource,
    effect: &StandardEffect,
    targets: &mut Option<EffectTargets>,
) -> Option<EffectWasApplied> {
    battle_trace!("Applying effect", battle, effect, targets);
    match effect {
        StandardEffect::BanishWhenLeavesPlay { .. } => banish_when_leaves_play(battle, targets),
        StandardEffect::Counterspell { .. } => counterspell(battle, source, targets),
        StandardEffect::CounterspellUnlessPaysCost { cost, .. } => {
            counterspell_unless_pays_cost::execute(battle, source, targets, cost)
        }
        StandardEffect::DiscardCards { count } => {
            discard_cards::execute(battle, source, source.controller(), *count)
        }
        StandardEffect::DrawCards { count } => {
            draw_cards(battle, source, source.controller(), *count)
        }
        StandardEffect::DrawCardsForEach { count, for_each } => {
            draw_cards_for_each(battle, source, *count, for_each)
        }
        StandardEffect::DissolveCharacter { .. } => dissolve(battle, source, targets),
        StandardEffect::Foresee { count } => foresee(battle, source, targets, *count),
        StandardEffect::GainEnergy { gains } => gain_energy(battle, source, *gains),
        StandardEffect::GainPoints { gains } => gain_points(battle, source, *gains),
        StandardEffect::GainsSpark { gains, .. } => gains_spark(battle, source, targets, *gains),
        StandardEffect::OpponentPaysCost { cost } => opponent_pays_cost(battle, source, cost),
        StandardEffect::ReturnFromYourVoidToHand { .. } => {
            return_from_your_void_to_hand(battle, source, targets)
        }
        StandardEffect::ReturnUpToCountFromYourVoidToHand { .. } => {
            return_up_to_count_from_your_void_to_hand(battle, source, targets)
        }
        StandardEffect::ReturnToHand { .. } => {
            return_from_battlefield_to_hand(battle, source, targets)
        }
        StandardEffect::PreventDissolveThisTurn { .. } => {
            prevent_dissolve_this_turn(battle, targets)
        }
        StandardEffect::PutCardsFromYourDeckIntoVoid { count } => {
            put_cards_from_your_deck_into_void(battle, source, *count)
        }
        _ => todo!("Implement {:?}", effect),
    }
}

fn banish_when_leaves_play(
    battle: &mut BattleState,
    targets: &mut Option<EffectTargets>,
) -> Option<EffectWasApplied> {
    let id = targeting::stack_card_id(targets)?;
    battle.ability_state.banish_when_leaves_play.insert(id.card_id());
    Some(EffectWasApplied)
}

fn counterspell(
    battle: &mut BattleState,
    source: EffectSource,
    targets: &mut Option<EffectTargets>,
) -> Option<EffectWasApplied> {
    let id = targeting::stack_card_id(targets)?;
    counterspell::execute(battle, source, id);
    Some(EffectWasApplied)
}

fn draw_cards(
    battle: &mut BattleState,
    source: EffectSource,
    player: PlayerName,
    count: u32,
) -> Option<EffectWasApplied> {
    if count > 0 {
        battle_deck::draw_cards(battle, source, player, count);
        Some(EffectWasApplied)
    } else {
        None
    }
}

fn draw_cards_for_each(
    battle: &mut BattleState,
    source: EffectSource,
    count: u32,
    for_each: &QuantityExpression,
) -> Option<EffectWasApplied> {
    let matching = quantity_expression::count(battle, source, for_each);
    let quantity = count * matching;
    if quantity > 0 {
        battle_deck::draw_cards(battle, source, source.controller(), quantity);
        Some(EffectWasApplied)
    } else {
        None
    }
}

fn dissolve(
    battle: &mut BattleState,
    source: EffectSource,
    targets: &mut Option<EffectTargets>,
) -> Option<EffectWasApplied> {
    let id = targeting::character_id(targets)?;
    dissolve::execute(battle, source, id);
    Some(EffectWasApplied)
}

fn foresee(
    battle: &mut BattleState,
    source: EffectSource,
    _targets: &mut Option<EffectTargets>,
    count: u32,
) -> Option<EffectWasApplied> {
    let player = source.controller();
    let cards = battle_deck::realize_top_of_deck(battle, player, count, SetRevealedToPlayer::Yes);

    if !cards.is_empty() {
        let prompt = SelectDeckCardOrderPrompt {
            initial: cards.clone(),
            moved: CardSet::new(),
            deck: cards,
            void: CardSet::new(),
        };

        battle.prompts.push_back(PromptData {
            source,
            player,
            prompt_type: PromptType::SelectDeckCardOrder { prompt },
            configuration: PromptConfiguration::default(),
        });

        Some(EffectWasApplied)
    } else {
        None
    }
}

fn gain_energy(
    battle: &mut BattleState,
    source: EffectSource,
    gains: Energy,
) -> Option<EffectWasApplied> {
    let player = source.controller();
    battle_trace!("Gaining energy", battle, player, gains);
    energy::gain(battle, player, source, gains);
    Some(EffectWasApplied)
}

fn gain_points(
    battle: &mut BattleState,
    source: EffectSource,
    gains: Points,
) -> Option<EffectWasApplied> {
    let player = source.controller();
    battle_trace!("Gaining points", battle, player, gains);
    points::gain(battle, player, source, gains, ShouldAnimate::Yes);
    Some(EffectWasApplied)
}

fn gains_spark(
    battle: &mut BattleState,
    source: EffectSource,
    targets: &mut Option<EffectTargets>,
    gains: Spark,
) -> Option<EffectWasApplied> {
    let id = targeting::character_id(targets)?;
    spark::gain(battle, source, id, gains);
    Some(EffectWasApplied)
}

fn opponent_pays_cost(
    battle: &mut BattleState,
    source: EffectSource,
    cost: &Cost,
) -> Option<EffectWasApplied> {
    pay_cost::execute(battle, source, source.controller().opponent(), cost);
    Some(EffectWasApplied)
}

fn prevent_dissolve_this_turn(
    battle: &mut BattleState,
    targets: &mut Option<EffectTargets>,
) -> Option<EffectWasApplied> {
    let id = targeting::character_id(targets)?;
    let object_id = card::get(battle, id).object_id;
    battle
        .ability_state
        .until_end_of_turn
        .prevent_dissolved
        .push(CardObjectId { card_id: id, object_id });
    Some(EffectWasApplied)
}

fn put_cards_from_your_deck_into_void(
    battle: &mut BattleState,
    source: EffectSource,
    count: u32,
) -> Option<EffectWasApplied> {
    if count == 0 {
        return None;
    }

    let player = source.controller();
    let cards = battle_deck::realize_top_of_deck(battle, player, count, SetRevealedToPlayer::No);

    battle.push_animation(source, || BattleAnimation::PutCardsFromDeckIntoVoid {
        player,
        cards: cards.clone(),
    });
    battle_trace!("Putting cards from deck into void", battle, player, cards);

    for card_id in cards {
        move_card::from_deck_to_void(battle, source, player, card_id);
    }

    Some(EffectWasApplied)
}

fn return_from_your_void_to_hand(
    battle: &mut BattleState,
    source: EffectSource,
    targets: &mut Option<EffectTargets>,
) -> Option<EffectWasApplied> {
    let id = targeting::void_card_id(battle, targets)?;
    move_card::from_void_to_hand(battle, source, source.controller(), id);
    Some(EffectWasApplied)
}

fn return_up_to_count_from_your_void_to_hand(
    battle: &mut BattleState,
    source: EffectSource,
    targets: &mut Option<EffectTargets>,
) -> Option<EffectWasApplied> {
    let void_cards = targeting::void_card_targets(targets)?;

    let controller = source.controller();
    for void_card_target in void_cards {
        move_card::from_void_to_hand(battle, source, controller, void_card_target.card_id);
    }
    Some(EffectWasApplied)
}

fn return_from_battlefield_to_hand(
    battle: &mut BattleState,
    source: EffectSource,
    targets: &mut Option<EffectTargets>,
) -> Option<EffectWasApplied> {
    let id = targeting::character_id(targets)?;
    battle.push_animation(source, || BattleAnimation::ApplyTargetedEffect {
        effect_name: TargetedEffectName::Dissolve,
        targets: vec![id.card_id()],
    });
    move_card::from_battlefield_to_hand(
        battle,
        source,
        card_properties::controller(battle, id),
        id,
    );
    Some(EffectWasApplied)
}
