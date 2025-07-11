use ability_data::cost::Cost;
use ability_data::quantity_expression_data::QuantityExpression;
use ability_data::standard_effect::StandardEffect;
use battle_queries::battle_player_queries::quantity_expression;
use battle_queries::battle_trace;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle_cards::card_set::CardSet;
use battle_state::battle_cards::stack_card_state::EffectTargets;
use battle_state::core::effect_source::EffectSource;
use battle_state::prompt_types::prompt_data::{
    PromptConfiguration, PromptData, PromptType, SelectDeckCardOrderPrompt,
};
use core_data::numerics::Spark;
use core_data::types::PlayerName;

use crate::card_mutations::{counterspell, deck, spark};
use crate::character_mutations::dissolve;
use crate::effects::apply_effect::EffectWasApplied;
use crate::effects::{counterspell_unless_pays_cost, pay_cost, targeting};
use crate::prompt_mutations::prompts;

/// Applies a [StandardEffect] to the given [BattleState].
///
/// You should almost always use `apply_effect::execute()` instead of calling
/// this directly.
pub fn apply(
    battle: &mut BattleState,
    source: EffectSource,
    effect: &StandardEffect,
    targets: Option<&EffectTargets>,
) -> Option<EffectWasApplied> {
    battle_trace!("Applying effect", battle, effect, targets);
    match effect {
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
        _ => todo!("Implement {:?}", effect),
    }
}

fn counterspell(
    battle: &mut BattleState,
    source: EffectSource,
    targets: Option<&EffectTargets>,
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
    targets: Option<&EffectTargets>,
) -> Option<EffectWasApplied> {
    let id = targeting::character_id(targets)?;
    dissolve::execute(battle, source, id);
    Some(EffectWasApplied)
}

fn foresee(
    battle: &mut BattleState,
    source: EffectSource,
    _targets: Option<&EffectTargets>,
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

        prompts::set(battle, PromptData {
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
    targets: Option<&EffectTargets>,
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
