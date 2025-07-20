use ability_data::cost::Cost;
use ability_data::quantity_expression_data::QuantityExpression;
use ability_data::standard_effect::StandardEffect;
use battle_queries::battle_card_queries::card_properties;
use battle_queries::battle_player_queries::quantity_expression;
use battle_queries::battle_trace;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardIdType;
use battle_state::battle_cards::card_set::CardSet;
use battle_state::battle_cards::stack_card_state::EffectTargets;
use battle_state::core::effect_source::EffectSource;
use battle_state::prompt_types::prompt_data::{
    PromptConfiguration, PromptData, PromptType, SelectDeckCardOrderPrompt,
};
use core_data::numerics::Spark;
use core_data::types::PlayerName;

use crate::card_mutations::{counterspell, deck, move_card, spark};
use crate::character_mutations::dissolve;
use crate::effects::apply_effect::EffectWasApplied;
use crate::effects::{counterspell_unless_pays_cost, pay_cost, targeting};

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
        StandardEffect::DrawCards { count } => {
            draw_cards(battle, source, source.controller(), *count)
        }
        StandardEffect::DrawCardsForEach { count, for_each } => {
            draw_cards_for_each(battle, source, *count, for_each)
        }
        StandardEffect::DissolveCharacter { .. } => dissolve(battle, source, targets),
        StandardEffect::Foresee { count } => foresee(battle, source, targets, *count),
        StandardEffect::GainsSpark { gains, .. } => gains_spark(battle, source, targets, *gains),
        StandardEffect::OpponentPaysCost { cost } => opponent_pays_cost(battle, source, cost),
        StandardEffect::ReturnFromYourVoidToHand { .. } => {
            return_from_your_void_to_hand(battle, source, targets)
        }
        StandardEffect::ReturnUpToCountFromYourVoidToHand { .. } => {
            return_up_to_count_from_your_void_to_hand(battle, source, targets)
        }
        StandardEffect::ReturnToHand { .. } => return_to_hand(battle, source, targets),
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
        deck::draw_cards(battle, source, player, count);
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
        deck::draw_cards(battle, source, source.controller(), quantity);
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
    let cards = deck::realize_top_of_deck(battle, player, count);

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
        move_card::from_void_to_hand(battle, source, controller, void_card_target.id);
    }
    Some(EffectWasApplied)
}

fn return_to_hand(
    battle: &mut BattleState,
    source: EffectSource,
    targets: &mut Option<EffectTargets>,
) -> Option<EffectWasApplied> {
    let id = targeting::character_id(targets)?;
    move_card::from_battlefield_to_hand(
        battle,
        source,
        card_properties::controller(battle, id),
        id,
    );
    Some(EffectWasApplied)
}
