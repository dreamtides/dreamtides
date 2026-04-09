use std::cmp;
use std::cmp::Ordering;
use std::f64::consts;

use ability_data::effect::ModelEffectChoiceIndex;
use battle_mutations::actions::apply_battle_action;
use battle_mutations::player_mutations::player_state;
use battle_queries::battle_card_queries::card_properties;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::{
    ForPlayer, LegalActions, PrimaryLegalAction, StandardLegalActions,
};
use battle_queries::panic_with;
use battle_state::actions::battle_actions::{
    BattleAction, CardOrderSelectionTarget, DeckCardSelectedOrder,
};
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use battle_state::battle::card_id::{
    BattleDeckCardId, CharacterId, HandCardId, StackCardId, VoidCardId,
};
use battle_state::battle_cards::card_set::CardSet;
use battle_state::prompt_types::prompt_data::SelectDeckCardOrderPrompt;
use chrono::Utc;
use core_data::types::PlayerName;
use ordered_float::OrderedFloat;
use petgraph::Direction;
use petgraph::prelude::NodeIndex;
use petgraph::visit::EdgeRef;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;
use rayon::prelude::*;
use tracing::debug;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;

use crate::decision_log::{
    ActionResult, BudgetDetails, DecisionLogEntry, DepthLevelStats, TreeTraversalAccumulator,
};
use crate::uct_config::UctConfig;
use crate::uct_tree::{SearchEdge, SearchGraph, SearchNode, SelectionMode};
use crate::{decision_log, log_search_results};

/// Monte Carlo search algorithm (V5: heuristic rollouts).
///
/// Identical to V1 except rollouts bias toward playing cards instead of
/// random pass/end decisions. See `heuristic_standard_action` for details.
pub fn search(
    initial_battle: &BattleState,
    player: PlayerName,
    config: &UctConfig,
) -> BattleAction {
    let legal = legal_actions::compute(initial_battle, player);
    let budget = compute_budget(&legal, config, initial_battle, player);

    let all_actions = legal.all();
    let filter_bp = should_override_positioning(initial_battle, player);
    let filtered_actions: Vec<BattleAction> = if filter_bp {
        debug!("Filtering BeginPositioning: opponent back-rank threat exceeds AI back-rank spark");
        all_actions.iter().copied().filter(|a| *a != BattleAction::BeginPositioning).collect()
    } else {
        all_actions
    };

    let action_results: Vec<_> = filtered_actions
        .par_iter()
        .with_min_len(if config.single_threaded { usize::MAX } else { 1 })
        .map(|&action| {
            search_action_candidate(
                initial_battle,
                player,
                budget.iterations_per_action,
                action,
                None,
            )
        })
        .collect();

    let Some(best_result) = action_results.iter().max_by_key(|result| {
        if result.visit_count == 0 {
            OrderedFloat(-f64::INFINITY)
        } else {
            OrderedFloat(result.total_reward.0 / result.visit_count as f64)
        }
    }) else {
        panic_with!("No legal actions available", initial_battle, player);
    };

    let action = best_result.action;
    let num_actions = filtered_actions.len();
    let total_iterations = budget.iterations_per_action * num_actions as u32;
    let num_threads = rayon::current_num_threads();

    debug!(?total_iterations, ?action, ?num_threads, "Picked AI action (V5)");
    if initial_battle.request_context.logging_options.log_ai_search_diagram {
        log_search_results::log_results_diagram(
            &best_result.graph,
            best_result.root,
            action,
            &initial_battle.request_context,
        );
    }

    if initial_battle.request_context.logging_options.log_ai_decisions {
        write_decision_log_entry(
            initial_battle,
            player,
            &action_results,
            best_result,
            &budget,
            num_actions,
            num_threads,
        );
    }

    action
}

fn search_action_candidate(
    initial_battle: &BattleState,
    player: PlayerName,
    iterations_per_action: u32,
    action: BattleAction,
    randomize_player_seed: Option<u64>,
) -> ActionSearchResult {
    let subscriber = tracing_subscriber::registry().with(EnvFilter::new("warn"));
    tracing::subscriber::with_default(subscriber, || {
        let mut graph = SearchGraph::default();
        let root = graph.add_node(SearchNode {
            player,
            total_reward: OrderedFloat(0.0),
            visit_count: 0,
            tried: Vec::new(),
        });

        let mut randomize_player_rng = Xoshiro256PlusPlus::seed_from_u64(
            randomize_player_seed.unwrap_or_else(|| rand::rng().random()),
        );

        let mut wins = 0u32;
        let mut losses = 0u32;
        let mut draws = 0u32;
        let log_tree_stats = initial_battle.request_context.logging_options.log_ai_decisions;
        let mut traversal_stats =
            if log_tree_stats { Some(TreeTraversalAccumulator::default()) } else { None };

        for _ in 0..iterations_per_action {
            let mut battle = player_state::randomize_battle_player(
                initial_battle,
                player.opponent(),
                randomize_player_rng.random(),
            );
            battle.request_context.logging_options.enable_action_legality_check = false;

            apply_battle_action::execute(&mut battle, player, action);

            let node =
                next_evaluation_target(&mut battle, &mut graph, root, traversal_stats.as_mut());
            let reward = evaluate(&mut battle, player);
            back_propagate_rewards(&mut graph, player, node, reward);

            match reward.0 {
                r if r > 0.0 => wins += 1,
                r if r < 0.0 => losses += 1,
                _ => draws += 1,
            }
        }

        let total_reward = graph[root].total_reward;
        let visit_count = graph[root].visit_count;
        let tree_node_count = graph.node_count();
        let tree_max_depth = crate::decision_log::compute_tree_depth(&graph, root);
        let depth_stats = traversal_stats.map(TreeTraversalAccumulator::into_depth_stats);
        ActionSearchResult {
            action,
            graph,
            root,
            total_reward,
            visit_count,
            wins,
            losses,
            draws,
            tree_node_count,
            tree_max_depth,
            depth_stats,
        }
    })
}

struct ActionSearchResult {
    action: BattleAction,
    graph: SearchGraph,
    root: NodeIndex,
    total_reward: OrderedFloat<f64>,
    visit_count: u32,
    wins: u32,
    losses: u32,
    draws: u32,
    tree_node_count: usize,
    tree_max_depth: u32,
    depth_stats: Option<Vec<DepthLevelStats>>,
}

fn next_evaluation_target(
    battle: &mut BattleState,
    graph: &mut SearchGraph,
    from_node: NodeIndex,
    mut stats: Option<&mut TreeTraversalAccumulator>,
) -> NodeIndex {
    let mut node = from_node;
    let mut depth = 0usize;
    while let Some(player) = legal_actions::next_to_act(battle) {
        let actions = legal_actions::compute(battle, player);
        let explored = &graph[node].tried;
        if let Some(action) = actions.find_missing(explored) {
            if let Some(s) = stats.as_mut() {
                s.record_expansion(depth, player, &action);
            }
            return add_child(battle, graph, player, node, action);
        } else {
            if let Some(s) = stats.as_mut() {
                s.record_selection(depth, player);
            }
            let best = best_child(graph, node, &actions, SelectionMode::Exploration);
            battle.request_context.logging_options.enable_action_legality_check = false;
            apply_battle_action::execute(battle, player, best.action);
            node = best.node;
        }
        depth += 1;
    }
    node
}

fn add_child(
    battle: &mut BattleState,
    graph: &mut SearchGraph,
    player: PlayerName,
    parent: NodeIndex,
    action: BattleAction,
) -> NodeIndex {
    battle.request_context.logging_options.enable_action_legality_check = false;
    graph[parent].tried.push(action);
    apply_battle_action::execute(battle, player, action);
    let child = graph.add_node(SearchNode {
        player,
        total_reward: OrderedFloat(0.0),
        visit_count: 0,
        tried: Vec::new(),
    });
    graph.add_edge(parent, child, SearchEdge { action });
    child
}

struct BestChild {
    action: BattleAction,
    node: NodeIndex,
}

fn best_child(
    graph: &SearchGraph,
    from_node: NodeIndex,
    legal: &LegalActions,
    selection_mode: SelectionMode,
) -> BestChild {
    let parent_visits = graph[from_node].visit_count;

    graph
        .edges(from_node)
        .filter(|e| legal.contains(e.weight().action, ForPlayer::Agent))
        .max_by_key(|edge| {
            let target = edge.target();
            child_score(
                parent_visits,
                graph[target].visit_count,
                graph[target].total_reward,
                selection_mode,
            )
        })
        .map(|edge| BestChild { action: edge.weight().action, node: edge.target() })
        .expect("No legal children available")
}

fn back_propagate_rewards(
    graph: &mut SearchGraph,
    maximizing_player: PlayerName,
    leaf_node: NodeIndex,
    reward: OrderedFloat<f64>,
) {
    let mut node = leaf_node;
    loop {
        let weight = &mut graph[node];
        weight.visit_count += 1;
        weight.total_reward += if weight.player == maximizing_player { reward } else { -reward };

        node = match graph.neighbors_directed(node, Direction::Incoming).next() {
            Some(n) => n,
            _ => break,
        };
    }
}

/// Scores a given [BattleState] for the maximizing player.
///
/// V5 rollouts bias toward playing cards instead of passing/ending.
fn evaluate(battle: &mut BattleState, maximizing_player: PlayerName) -> OrderedFloat<f64> {
    while let Some(player) = legal_actions::next_to_act(battle) {
        let legal = legal_actions::compute(battle, player);
        let Some(action) = rollout_action(battle, player, &legal) else {
            panic_with!("No legal actions available", battle, player);
        };
        apply_battle_action::execute(battle, player, action);
    }

    let BattleStatus::GameOver { winner } = battle.status else {
        panic_with!("Battle has not ended", battle);
    };
    let reward = if winner == Some(maximizing_player) {
        1.0
    } else if winner.is_some() {
        -1.0
    } else {
        0.0
    };
    OrderedFloat(reward)
}

fn should_override_positioning(battle: &BattleState, player: PlayerName) -> bool {
    let opponent = player.opponent();
    let opp_front = &battle.cards.battlefield(opponent).front;

    if opp_front.iter().any(Option::is_some) {
        return false;
    }

    let opp_back_max_spark = battle
        .cards
        .battlefield(opponent)
        .back
        .iter()
        .filter_map(|slot| *slot)
        .filter_map(|id| card_properties::spark(battle, opponent, id))
        .map(|s| s.0)
        .max()
        .unwrap_or(0);

    let own_back_max_spark = battle
        .cards
        .battlefield(player)
        .back
        .iter()
        .filter_map(|slot| *slot)
        .filter_map(|id| card_properties::spark(battle, player, id))
        .map(|s| s.0)
        .max()
        .unwrap_or(0);

    opp_back_max_spark > own_back_max_spark
}

fn rollout_action(
    battle: &BattleState,
    player: PlayerName,
    legal: &LegalActions,
) -> Option<BattleAction> {
    match legal {
        LegalActions::Standard { actions } => {
            heuristic_standard_action(battle, player, legal, actions)
        }
        LegalActions::SelectPositioningCharacter { eligible } => {
            heuristic_select_positioning_character(battle, player, eligible)
        }
        LegalActions::AssignColumn { character, block_targets, attack_column } => {
            heuristic_assign_column(battle, player, *character, block_targets, *attack_column)
        }
        LegalActions::SelectCharacterPrompt { valid } => {
            heuristic_select_character_target(battle, player, valid)
        }
        LegalActions::SelectVoidCardPrompt { valid, current, maximum_selection } => {
            heuristic_select_void_card(battle, valid, current, *maximum_selection)
        }
        LegalActions::SelectHandCardPrompt { valid, current, target_count } => {
            heuristic_select_hand_card(battle, valid, current, *target_count)
        }
        LegalActions::ModalEffectPrompt { valid_choices } => {
            Some(BattleAction::SelectModalEffectChoice(ModelEffectChoiceIndex(
                valid_choices.iter().next()?,
            )))
        }
        LegalActions::SelectActivatedAbilityPrompt { choice_count } if *choice_count > 0 => {
            Some(BattleAction::SelectActivatedAbilityChoice(0))
        }
        LegalActions::SelectStackCardPrompt { valid } => {
            heuristic_select_stack_card_target(battle, player, valid)
        }
        LegalActions::SelectEnergyValuePrompt { maximum, .. } => {
            Some(BattleAction::SelectEnergyAdditionalCost(*maximum))
        }
        LegalActions::SelectDeckCardOrder { current } => {
            heuristic_select_deck_card_order(battle, current)
        }
        _ => legal.random_action(),
    }
}

/// V5 heuristic for Standard action selection during rollouts.
///
/// 85% of the time, prefers playing cards/abilities over passing or ending
/// the turn, selecting the highest-scored playable card. 15% of the time,
/// falls back to random selection for MCTS exploration diversity.
fn heuristic_standard_action(
    battle: &BattleState,
    _player: PlayerName,
    legal: &LegalActions,
    actions: &StandardLegalActions,
) -> Option<BattleAction> {
    // 15% of the time, use fully random action (with BeginPositioning override)
    if fastrand::u8(..100) < 15 {
        let action = legal.random_action()?;
        if action == BattleAction::EndTurn && actions.can_begin_positioning {
            return Some(BattleAction::BeginPositioning);
        }
        return Some(action);
    }

    // 85% of the time: if any cards/abilities are playable, 70% play the
    // best one, 30% take the primary action. The 30% pass rate prevents
    // rollout loops where both players endlessly play cards without
    // advancing the game.
    let has_plays = !actions.play_card_from_hand.is_empty()
        || !actions.play_card_from_void.is_empty()
        || !actions.activate_abilities_for_character.is_empty();

    if has_plays && fastrand::u8(..100) < 70 {
        return pick_best_play(battle, actions);
    }

    // No card plays available — fall through to primary action
    if actions.can_begin_positioning {
        Some(BattleAction::BeginPositioning)
    } else {
        Some(match actions.primary {
            PrimaryLegalAction::PassPriority => BattleAction::PassPriority,
            PrimaryLegalAction::EndTurn => BattleAction::EndTurn,
            PrimaryLegalAction::StartNextTurn => BattleAction::StartNextTurn,
        })
    }
}

/// Picks the best card play or ability activation based on a simple
/// heuristic score: spark * 3 + energy cost + fast bonus.
fn pick_best_play(battle: &BattleState, actions: &StandardLegalActions) -> Option<BattleAction> {
    let mut best_action = None;
    let mut best_score = f64::NEG_INFINITY;

    for card_id in actions.play_card_from_hand.iter() {
        let cost = f64::from(card_properties::converted_energy_cost(battle, card_id).0);
        let spark = f64::from(card_properties::base_spark(battle, card_id).unwrap_or_default().0);
        let fast_bonus = if card_properties::is_fast(battle, card_id) { 5.0 } else { 0.0 };
        let score = spark * 3.0 + cost + fast_bonus;
        if score > best_score {
            best_score = score;
            best_action = Some(BattleAction::PlayCardFromHand(card_id));
        }
    }

    for card_id in actions.play_card_from_void.iter() {
        let cost = f64::from(card_properties::converted_energy_cost(battle, card_id).0);
        let spark = f64::from(card_properties::base_spark(battle, card_id).unwrap_or_default().0);
        let score = spark * 3.0 + cost;
        if score > best_score {
            best_score = score;
            best_action = Some(BattleAction::PlayCardFromVoid(card_id));
        }
    }

    for character_id in actions.activate_abilities_for_character.iter() {
        let score = 2.0;
        if score > best_score {
            best_score = score;
            best_action = Some(BattleAction::ActivateAbilityForCharacter(character_id));
        }
    }

    best_action
}

fn heuristic_select_positioning_character(
    battle: &BattleState,
    player: PlayerName,
    eligible: &CardSet<CharacterId>,
) -> Option<BattleAction> {
    if eligible.is_empty() {
        return Some(BattleAction::EndTurn);
    }

    let opponent = player.opponent();
    let opp_front = &battle.cards.battlefield(opponent).front;
    let has_block_targets = opp_front.iter().any(Option::is_some);

    if !has_block_targets {
        let opp_back_max_spark = battle
            .cards
            .battlefield(opponent)
            .back
            .iter()
            .filter_map(|slot| *slot)
            .filter_map(|id| card_properties::spark(battle, opponent, id))
            .map(|s| s.0)
            .max()
            .unwrap_or(0);

        let own_max_spark = eligible
            .iter()
            .filter_map(|id| card_properties::spark(battle, player, id))
            .map(|s| s.0)
            .max()
            .unwrap_or(0);

        if opp_back_max_spark > own_max_spark {
            return Some(BattleAction::EndTurn);
        }
    }

    let index = fastrand::usize(..eligible.len());
    eligible.iter().nth(index).map(BattleAction::SelectCharacterForPositioning)
}

fn heuristic_assign_column(
    battle: &BattleState,
    player: PlayerName,
    character: CharacterId,
    block_targets: &[u8],
    attack_column: Option<u8>,
) -> Option<BattleAction> {
    let own_spark = card_properties::spark(battle, player, character).unwrap_or_default().0;
    let opponent = player.opponent();

    let mut best_block_col = None;
    let mut best_block_spark = 0u32;
    for &col in block_targets {
        if let Some(opp_id) = battle.cards.battlefield(opponent).front[col as usize] {
            let opp_spark = card_properties::spark(battle, opponent, opp_id).unwrap_or_default().0;
            if opp_spark > best_block_spark {
                best_block_spark = opp_spark;
                best_block_col = Some(col);
            }
        }
    }

    if let Some(col) = best_block_col
        && best_block_spark >= own_spark
    {
        return Some(BattleAction::MoveCharacterToFrontRank(character, col));
    }

    if let Some(col) = attack_column {
        return Some(BattleAction::MoveCharacterToFrontRank(character, col));
    }

    if !block_targets.is_empty() {
        let index = fastrand::usize(..block_targets.len());
        Some(BattleAction::MoveCharacterToFrontRank(character, block_targets[index]))
    } else {
        None
    }
}

/// Heuristic for character target selection during rollouts.
///
/// Most targeted effects are harmful, so prefer opponent's highest-spark
/// character. If only own characters are valid targets, pick the lowest
/// spark (sacrifice the least valuable).
fn heuristic_select_character_target(
    battle: &BattleState,
    player: PlayerName,
    valid: &CardSet<CharacterId>,
) -> Option<BattleAction> {
    let mut best_target = None;
    let mut best_score = f64::NEG_INFINITY;

    for character_id in valid.iter() {
        let controller = card_properties::controller(battle, character_id);
        let spark =
            f64::from(card_properties::base_spark(battle, character_id).unwrap_or_default().0);
        let score = if controller != player {
            // Opponent character: prefer highest spark (most impactful target)
            spark
        } else {
            // Own character: prefer lowest spark (sacrifice least valuable)
            -spark
        };
        if score > best_score {
            best_score = score;
            best_target = Some(character_id);
        }
    }

    best_target.map(BattleAction::SelectCharacterTarget)
}

/// Heuristic for stack card target selection during rollouts.
///
/// For prevent/counter effects, target the highest-cost card on the stack.
fn heuristic_select_stack_card_target(
    battle: &BattleState,
    player: PlayerName,
    valid: &CardSet<StackCardId>,
) -> Option<BattleAction> {
    let mut best_target = None;
    let mut best_score = f64::NEG_INFINITY;

    for card_id in valid.iter() {
        let controller = card_properties::controller(battle, card_id);
        let cost = f64::from(card_properties::converted_energy_cost(battle, card_id).0);
        // Prefer targeting opponent's most expensive cards
        let score = if controller != player { cost + 10.0 } else { -cost };
        if score > best_score {
            best_score = score;
            best_target = Some(card_id);
        }
    }

    best_target.map(BattleAction::SelectStackCardTarget)
}

/// Heuristic for void card selection during rollouts.
///
/// Picks the highest-cost unselected card (reclaim the best) and submits
/// when the selection is full.
fn heuristic_select_void_card(
    battle: &BattleState,
    valid: &CardSet<VoidCardId>,
    current: &CardSet<VoidCardId>,
    maximum_selection: usize,
) -> Option<BattleAction> {
    if current.len() >= maximum_selection {
        return Some(BattleAction::SubmitVoidCardTargets);
    }

    let mut best_card = None;
    let mut best_score = f64::NEG_INFINITY;

    for card_id in valid.iter() {
        if current.contains(card_id) {
            continue;
        }
        let cost = f64::from(card_properties::converted_energy_cost(battle, card_id).0);
        let spark = f64::from(card_properties::base_spark(battle, card_id).unwrap_or_default().0);
        let score = spark * 3.0 + cost;
        if score > best_score {
            best_score = score;
            best_card = Some(card_id);
        }
    }

    match best_card {
        Some(id) => Some(BattleAction::SelectVoidCardTarget(id)),
        None => Some(BattleAction::SubmitVoidCardTargets),
    }
}

/// Heuristic for hand card selection during rollouts.
///
/// When forced to select hand cards (usually discard), picks the
/// lowest-cost card first. Submits when target count is reached.
fn heuristic_select_hand_card(
    battle: &BattleState,
    valid: &CardSet<HandCardId>,
    current: &CardSet<HandCardId>,
    target_count: usize,
) -> Option<BattleAction> {
    if current.len() >= target_count {
        return Some(BattleAction::SubmitHandCardTargets);
    }

    let mut best_card = None;
    let mut best_score = f64::INFINITY;

    for card_id in valid.iter() {
        if current.contains(card_id) {
            continue;
        }
        let cost = f64::from(card_properties::converted_energy_cost(battle, card_id).0);
        let spark = f64::from(card_properties::base_spark(battle, card_id).unwrap_or_default().0);
        // Lower score = discard first (cheapest, least spark)
        let score = spark * 3.0 + cost;
        if score < best_score {
            best_score = score;
            best_card = Some(card_id);
        }
    }

    match best_card {
        Some(id) => Some(BattleAction::SelectHandCardTarget(id)),
        None => Some(BattleAction::SubmitHandCardTargets),
    }
}

/// Heuristic for Foresee (deck card ordering) during rollouts.
///
/// Puts the highest-value card on top of the deck and sends the
/// lowest-value card to void. Submits when all cards are placed.
fn heuristic_select_deck_card_order(
    battle: &BattleState,
    current: &SelectDeckCardOrderPrompt,
) -> Option<BattleAction> {
    let next_card = current.initial.iter().find(|c| !current.moved.contains(**c));
    let Some(&card_id) = next_card else {
        return Some(BattleAction::SubmitDeckCardOrder);
    };

    let cost = f64::from(card_properties::converted_energy_cost(battle, card_id).0);
    let spark = f64::from(card_properties::base_spark(battle, card_id).unwrap_or_default().0);
    let value = spark * 3.0 + cost;

    // High-value cards go on top of deck, low-value go to void
    let target = if value >= 3.0 {
        CardOrderSelectionTarget::Deck(0)
    } else {
        CardOrderSelectionTarget::Void
    };

    Some(BattleAction::SelectOrderForDeckCard(DeckCardSelectedOrder { card_id, target }))
}

fn child_score(
    parent_visits: u32,
    child_visits: u32,
    reward: OrderedFloat<f64>,
    selection_mode: SelectionMode,
) -> OrderedFloat<f64> {
    let exploitation = reward / f64::from(child_visits);
    let exploration =
        f64::sqrt((2.0 * f64::ln(f64::from(parent_visits))) / f64::from(child_visits));
    let exploration_bias = match selection_mode {
        SelectionMode::Exploration => consts::FRAC_1_SQRT_2,
        SelectionMode::RewardOnly => 0.0,
    };
    exploitation + (exploration_bias * exploration)
}

struct BudgetInfo {
    iterations_per_action: u32,
    base_iterations: u32,
    multiplier: f64,
    multiplier_reason: &'static str,
}

fn compute_budget(
    legal: &LegalActions,
    config: &UctConfig,
    battle: &BattleState,
    agent: PlayerName,
) -> BudgetInfo {
    let base_iterations = match legal.len() {
        0 => config.max_iterations_per_action,
        action_count => {
            let total_budget =
                config.max_iterations_per_action * config.max_total_actions_multiplier;
            let distributed_iterations = (total_budget as f64 / action_count as f64) as u32;
            cmp::min(distributed_iterations, config.max_iterations_per_action)
        }
    };

    let is_main =
        battle.turn.active_player == agent && matches!(battle.phase, BattleTurnPhase::Main);
    let player_state = battle.players.player(battle.turn.active_player);

    let (multiplier, multiplier_reason) = match is_main {
        _ if legal.is_prompt() => (0.5, "prompt"),
        true if player_state.current_energy >= player_state.produced_energy => (1.5, "first_main"),
        true => (1.0, "main"),
        _ => (0.75, "other"),
    };
    let (applied_multiplier, applied_reason) = match config.iteration_multiplier_override {
        Some(m) => (m, "override"),
        None => (multiplier, multiplier_reason),
    };
    BudgetInfo {
        iterations_per_action: ((base_iterations as f64) * applied_multiplier) as u32,
        base_iterations,
        multiplier: applied_multiplier,
        multiplier_reason: applied_reason,
    }
}

fn write_decision_log_entry(
    battle: &BattleState,
    player: PlayerName,
    action_results: &[ActionSearchResult],
    best_result: &ActionSearchResult,
    budget: &BudgetInfo,
    num_actions: usize,
    num_threads: usize,
) {
    let best_avg = if best_result.visit_count == 0 {
        0.0
    } else {
        best_result.total_reward.0 / best_result.visit_count as f64
    };

    let chosen_action = best_result.action;
    let mut results: Vec<ActionResult> = action_results
        .iter()
        .map(|r| {
            let avg =
                if r.visit_count == 0 { 0.0 } else { r.total_reward.0 / r.visit_count as f64 };
            ActionResult {
                action: format!("{:?}", r.action),
                action_short: r.action.battle_action_string(),
                total_reward: r.total_reward.0,
                visit_count: r.visit_count,
                avg_reward: avg,
                wins: r.wins,
                losses: r.losses,
                draws: r.draws,
                tree_node_count: r.tree_node_count,
                tree_max_depth: r.tree_max_depth,
                depth_stats: if r.action == chosen_action { r.depth_stats.clone() } else { None },
            }
        })
        .collect();
    results.sort_by(|a, b| b.avg_reward.partial_cmp(&a.avg_reward).unwrap_or(Ordering::Equal));

    let entry = DecisionLogEntry {
        timestamp: Utc::now().to_rfc3339(),
        player: format!("{:?}", player),
        chosen_action: format!("{:?}", best_result.action),
        chosen_action_short: best_result.action.battle_action_string(),
        chosen_avg_reward: best_avg,
        game_state: decision_log::build_game_state_snapshot(battle),
        budget: BudgetDetails {
            iterations_per_action: budget.iterations_per_action,
            base_iterations: budget.base_iterations,
            total_iterations: budget.iterations_per_action * num_actions as u32,
            num_actions,
            multiplier: budget.multiplier,
            multiplier_reason: budget.multiplier_reason.to_string(),
            num_threads,
        },
        action_results: results,
    };
    decision_log::write_decision_log(&entry, &battle.request_context);
}
