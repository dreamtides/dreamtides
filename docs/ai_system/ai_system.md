# AI System

The AI subsystem selects actions for computer-controlled opponents during
battles. It uses Monte Carlo Tree Search with the UCT algorithm, parallelized
across candidate actions via rayon, with information-set sampling to handle
hidden cards.

## Agent Types

The `GameAI` enum (ai_data/src/game_ai.rs) defines six agent variants:

- **MonteCarlo(n)**: The production AI. Runs multi-threaded UCT search with
  `n * 1000` max iterations per candidate action. The default opponent uses
  `MonteCarlo(50)` (50,000 iterations).
- **MonteCarloSingleThreaded(n)**: Same algorithm, forced single-threaded.
  Used for deterministic benchmarking.
- **RandomAction**: Picks a uniformly random legal action. Used in testing.
- **FirstAvailableAction**: Returns the first legal action from the sorted
  list. Deterministic and trivially fast.
- **WaitFiveSeconds**: Sleeps five seconds then returns the first action.
  Used for UI timing tests.
- **AlwaysPanic**: Panics if invoked. Sentinel for players that should never
  be asked to act.

A debug panel (display/src/panels/set_opponent_agent_panel.rs) allows
switching the opponent agent at runtime during development.

## Agent Selection Entry Point

The public entry point is `agent_search::select_action()` in the ai_agents
crate. It:

- Asserts the given player is next to act and has legal actions.
- Short-circuits immediately if exactly one legal action exists.
- Randomizes the opponent's hidden information (see Information Hiding below).
- Dispatches to the appropriate strategy based on the GameAI variant.
- Logs the elapsed search time.

For Monte Carlo variants, it constructs a UctConfig and delegates to
`uct_search::search()`.

## Monte Carlo Tree Search (UCT)

The search algorithm lives in the ai_uct crate. It follows the standard UCT
(Upper Confidence bounds applied to Trees) variant of MCTS.

### Search Architecture

The search uses root parallelization: each legal action from the current
position gets its own independent search tree. These per-action searches run
in parallel via rayon. After all searches complete, the action whose root
node has the highest average reward is selected.

### Per-Action Search Loop

For each candidate action, the system creates a petgraph-backed search graph
with a single root node and runs a configurable number of iterations. Each
iteration:

- Re-randomizes the opponent's hidden state with a fresh seed.
- Applies the candidate action to a cloned battle state.
- Runs three MCTS phases: tree policy (selection/expansion), default policy
  (random rollout), and backpropagation.

### Tree Policy

Starting from the root, the tree policy walks down existing nodes. At each
node, if any untried action exists, it expands by creating a new child node.
If all actions have been tried, it selects the best child using the UCT
formula and continues. Each node tracks its tried actions in a Vec for a
small performance gain over iterating edges.

### Default Policy (Evaluation)

The default policy plays out the game to completion using uniformly random
moves. The result is +1.0 if the maximizing player won, -1.0 otherwise.
There is no heuristic evaluation or early termination -- full random rollouts
to game end consistently outperform heuristic alternatives.

### UCT Formula

Child selection uses the UCT1 formula: exploitation (average reward) plus an
exploration term scaled by the constant 1/sqrt(2), following Kocsis and
Szepesvari's recommendation.

### Backpropagation

Rewards propagate from the leaf back to the root. Each node's visit count
increments by one. The reward is added if the node's player matches the
maximizing player and negated otherwise (standard negamax-style for
two-player games).

## Information Hiding

The AI never sees the opponent's actual hand. Before search begins,
`randomize_battle_player()` (battle_mutations) creates a clone of the battle
state and randomizes two things about the opponent:

- **Hand cards**: Shuffles the opponent's hand back into their deck, then
  redraws the same number of cards. The AI knows hand size but not contents.
- **Dreamwell order**: Shuffles the shared dreamwell deck while preserving
  phase grouping.

During Monte Carlo search, each iteration re-randomizes with a different
seed. This implements information-set MCTS (ISMCTS), where each simulation
samples a different possible hidden state. Less frequent randomization
improves raw performance but consistently reduces play quality.

## Iteration Budget

The iteration count per action scales dynamically based on game context.

The base budget is `max_iterations_per_action * max_total_actions_multiplier`
(default multiplier is 6). This total is divided among candidate actions,
capped at max_iterations_per_action each. With the default MonteCarlo(50),
the total budget is 300,000 iterations split across all legal actions.

A phase-based multiplier adjusts the base:

- **Prompt responses**: 0.5x. Prompts need fast responses and are typically
  less strategically significant.
- **First main-phase action at full energy**: 1.5x. This is the most
  impactful decision point with the widest option space.
- **Other main-phase actions**: 1.0x.
- **Non-main phase or opponent's turn**: 0.75x. Priority responses and
  end-phase actions have fewer meaningful choices.

## Integration with the Battle Loop

The action loop in handle_battle_action::execute() drives AI integration.
After applying each action, it checks who acts next via
`legal_actions::next_to_act()`. This function is player-type-agnostic -- it
determines the next player purely from game state (prompts, stack priority,
turn phase). The caller checks whether the returned player is an Agent or
User.

When the next player is an AI agent:

- Incremental render updates are sent to the human client so it can animate
  intermediate states.
- The system checks for a speculative response (see below).
- If no speculative hit, `agent_search::select_action()` runs.
- The AI's chosen action feeds back into the same loop, continuing without
  returning to the client.

The loop only returns when a human player needs to act or the game ends.
Multiple AI actions (responding to priority, auto-executing, main phase
plays) chain within a single loop execution.

When exactly one legal action exists (pass priority, start next turn,
single-choice prompt), it is auto-executed without consulting the AI.

## Speculative Search

When control returns to the human player, the system speculatively
pre-computes the AI's response. It assumes the human will take the "primary
legal action" (pass priority, end turn, or start next turn), clones the
battle state, simulates that action plus any auto-executes, then spawns
`agent_search::select_action()` on a background thread via
`tokio::task::spawn_blocking`.

The result is stored in the StateProvider behind an Arc/Mutex/Condvar. When
the human actually acts:

- If the action matches the prediction, the pre-computed result is used
  (blocking briefly if the search is still running). This is a cache hit.
- If the action differs, the speculative result is discarded and a fresh
  search runs. This is a cache miss.

Since pass priority and end turn are the most common human actions, the
speculative search frequently hits, significantly reducing perceived AI
response latency.

## Crate Organization

The AI system spans three crates:

- **ai_data** (Layer 1): Contains only the GameAI enum. Minimal dependencies
  so other crates can reference AI types without pulling in search logic.
- **ai_agents** (Layer 6): The public-facing entry point. Dispatches to the
  appropriate search strategy and handles information randomization.
- **ai_uct** (Layer 6): The MCTS/UCT implementation. Contains the search
  graph data structures (petgraph-backed), the search algorithm, iteration
  budget logic, and optional Graphviz DOT export for debugging search trees.

## Debugging

When `log_ai_search_diagram` is enabled in LoggingOptions, the system exports
a Graphviz DOT file of the best action's search tree (depth-limited to 3
levels). Tracing is suppressed to warn level during search to avoid noise
from the thousands of simulated actions per decision.

The ai_matchup binary (ai_matchup crate) pits two AI agents against each
other across multiple matches with position swapping, reporting win rates and
timing statistics.
