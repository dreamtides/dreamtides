# MCTS Positioning Redesign

## Problem

The current MCTS search catastrophically fails at positioning decisions due to
search tree explosion. The multi-step positioning flow (BeginPositioning ->
SelectCharacterForPositioning -> AssignColumn, repeated per character) creates a
combinatorial branching factor that overwhelms the search budget. With 8
possible characters in 16 possible positions, the tree is wide/shallow with
almost no node reuse (tree_node_count ~ visit_count).

Additional problems:
- Rollout policy always converts EndTurn to BeginPositioning, so both players
  always position every turn. No concept of strategically skipping positioning.
- No "hold back when losing" logic. Characters are worth more as future
  blockers than as 1-3 point attackers when behind, but rollouts always send
  characters to fight.
- Blocking and attacking converge in rollouts: a character "saved" by blocking
  this turn gets re-positioned next turn, erasing the strategic value of
  defensive play.
- Opponent card-play actions from randomized hands cause further tree explosion
  with minimal strategic signal.
- The half-measure heuristic hacks added to uct_search.rs have been making the
  AI worse.

## Approach: Positioning Assignments as Atomic MCTS Actions

Replace the multi-step positioning flow with heuristic-generated "position
assignments" -- complete board plans evaluated as atomic MCTS actions. A
heuristic generator produces up to 6 candidate assignments before MCTS begins.
Each assignment specifies where every eligible character goes (front rank column
or stay back). MCTS evaluates these candidates alongside other standard actions
(card plays, EndTurn).

Key principles:
- Positioning is a geometric/tactical problem suited to heuristic computation.
- "Should I attack or defend?" is a strategic question suited to MCTS rollout
  evaluation.
- Put each problem in the right solver's hands.

## Forking Strategy

The new system is implemented as a separate `GameAI` variant alongside the
existing Monte Carlo implementation. The existing `uct_search.rs` is left
untouched. The new search lives in a new file (e.g., `uct_search_v2.rs` or
similar) with its own entry point. A new `GameAI` variant (e.g.,
`MonteCarloV2(u32)`) dispatches to it from `agent_search.rs`.

This allows direct head-to-head comparison via the existing `ai_matchup`
harness:
```
just matchup '{"MonteCarlo": 50}' '{"MonteCarloV2": 50}' --matches 20 --deck core-11
```

Both starting-player configurations are tested since the existing harness
alternates player positions on odd match indices.

## Position Assignment Data Model

```rust
enum CharacterPlacement {
    StayBack,
    MoveToFrontRank(u8),  // column
}

struct PositionAssignment {
    placements: Vec<(CharacterId, CharacterPlacement)>,
}
```

Each assignment covers all eligible characters (back rank, no summoning
sickness, not already moved this turn). The struct is kept minimal for
performance -- no strings, no precomputed scores. Logging generates
human-readable descriptions at log time from the placements and board state.

### How assignments enter MCTS

Before MCTS begins, if `can_begin_positioning` is true, the generator produces
up to 6 candidates. The root action list becomes: all standard legal actions
except BeginPositioning, plus one synthetic action per positioning assignment.

Each assignment is evaluated by `search_action_candidate()` like any other
action. Applying an assignment atomically moves all specified characters to
their front-rank columns and transitions to the ending phase, bypassing the
multi-step positioning state machine.

### How assignments translate to real actions

After MCTS selects a winning assignment, the AI emits the standard action
sequence: BeginPositioning, then (SelectCharacterForPositioning +
MoveCharacterToFrontRank) for each placed character, then EndTurn. The game
state machine is unchanged.

## Assignment Generator

The generator takes the current board state and produces up to 6 candidate
assignments covering the strategic spectrum.

### Inputs

- Eligible characters (back rank, no summoning sickness, not moved this turn)
  and their spark values
- Opponent front-rank characters and their spark values (current threats)
- Opponent back-rank characters and their spark values (future threats)
- AI's current front-rank characters (already positioned from prior turns)

### Always-included candidates

1. **Hold all back** -- every eligible character stays. Always included. The
   pure defensive / save-for-future-blocking option.

2. **Efficient blocking** -- assign the cheapest character that wins each
   matchup (own spark >= opponent spark) to block the highest-spark opponents.
   Prioritize blocking highest-spark opponents first. Leave remaining characters
   in back rank. Generate 1-2 variants varying which characters are assigned.

3. **Chump blocking** -- assign the weakest characters to block the strongest
   opponents. They die, but prevent points. Leave stronger characters back for
   future value. This is critical for "buying time to draw answers" -- the key
   failure mode of the current system.

4. **Attack-focused** -- send characters to empty columns, but only characters
   whose spark is high enough that the opponent likely cannot profitably block
   (character spark exceeds opponent's largest uncommitted back-rank character).
   Omitted entirely if no character qualifies, rather than generating a bad
   attack.

5. **Mixed** -- block the single highest-spark opponent threat with an efficient
   blocker, then attack with the strongest remaining character if it qualifies
   as a profitable attacker. Remaining characters stay back.

### Scoring and selection

If more than 6 candidates are generated, rank by a fast heuristic:
`points_prevented_by_blocks - spark_of_characters_expected_to_die +
points_expected_from_attacks * discount` where the discount reflects the
opponent's ability to block based on their back-rank characters. Take the top 6.

In practice, with typical board states (2-4 eligible characters, 1-3 opponent
front-rank threats), the generator produces 4-6 candidates naturally.

### Edge cases

- No eligible characters: only "hold all" (a no-op), skip positioning entirely.
- No opponent front-rank characters and no empty columns: only "hold all".
- One eligible character: at most 3-4 candidates (hold, block best, attack,
  chump-block best).

### Key heuristic considerations

**Chump blocking value:** A 1-spark character blocking a 6-spark attacker
prevents 6 points at the cost of a 1-spark body. Net value is +5, not -1.

**Attack profitability:** Attacking does NOT simply score points equal to spark.
On the opponent's subsequent turn, they can position blockers. A 3-spark
character attacking into a column where the opponent has a 6-spark character
available results in the 3-spark character being dissolved for zero points. The
generator must factor in opponent back-rank threats when evaluating attacks.

## Opponent Modeling During Tree Search

When the tree policy reaches an opponent decision point (not an AI fast-card
response point), instead of expanding individual opponent actions as tree nodes,
play out the opponent's choices via a greedy heuristic and continue tree
traversal from the resulting state. This eliminates opponent card-play branching
from the tree.

**AI response points are preserved:** If the AI has fast cards available and the
opponent plays a card onto the stack, the tree branches on the AI's response
(counter or pass). This is the only opponent-turn branching that remains in the
tree.

The greedy opponent heuristic:
- Play the highest-cost affordable card from hand, repeat until energy exhausted
  or no playable cards remain.
- Activate abilities greedily.
- Generate a single best heuristic positioning assignment and apply it
  atomically.

This preserves the value of hand randomization (the AI naturally "plays around"
different opponent hands across iterations) while focusing all tree depth on AI
decisions.

## Rollout Turn Structure

A single rollout turn (for either player):

1. **Dreamwell phase** -- applied normally (deterministic).
2. **Draw phase** -- applied normally.
3. **Dawn phase** -- triggers fire normally.
4. **Card play** -- greedy heuristic: repeatedly play the highest-cost
   affordable card from hand until energy exhausted or no playable cards remain.
   Activate abilities greedily.
5. **Positioning** -- generate the single best heuristic assignment and apply
   atomically. The generator can and will produce "hold all" as the best option
   when defensive play is correct.
6. **Ending + Judgment** -- resolved normally.

### Key differences from current rollouts

- Card play is greedy instead of random -- more realistic play from both sides.
- Positioning is a single atomic step instead of the multi-step state machine.
- No forced "always position" -- the generator produces "hold all" when
  appropriate, fixing the core problem of defensive play being invisible.
- Fast cards are skipped in rollouts entirely. The tree handles AI fast-card
  decisions. Rollout-level counterspell modeling adds noise without signal. If
  this turns out to lose important signal, a simple heuristic can be added later.

## Decision Logging

The existing JSONL decision log (`ai_decisions.jsonl`) is extended with a new
optional field for positioning decisions:

```
positioning_candidates: Option<Vec<PositioningCandidate>>
```

Each `PositioningCandidate` contains:
- The list of placements (character ID + column or stay-back).
- The heuristic score from the generator.
- The MCTS results after evaluation (avg reward, visit count,
  wins/losses/draws).

The logging path (only when `log_ai_decisions` is enabled) generates
human-readable descriptions at log time from the placements and board state.
Example descriptions: `"hold-all"`, `"block-5@col2(with-3)+hold-rest"`,
`"chump-1@col6+attack-4@col1"`.

Example log entry for a positioning decision:
```json
{
  "chosen_action": "PositionAssignment #3",
  "positioning_candidates": [
    {
      "description": "hold-all",
      "heuristic_score": 0.0,
      "avg_reward": -0.42,
      "visits": 8333
    },
    {
      "description": "block-6@col2(with-3)",
      "heuristic_score": 3.0,
      "avg_reward": -0.31,
      "visits": 8333
    },
    {
      "description": "chump-1@col2+attack-4@col5",
      "heuristic_score": 2.5,
      "avg_reward": -0.18,
      "visits": 8333
    }
  ]
}
```

This makes it easy to diagnose: "the generator didn't consider X" vs "MCTS
evaluated X but chose Y."

## What Gets Removed (from the new V2 search only)

- `should_override_positioning()` -- the heuristic hack for filtering
  BeginPositioning.
- `heuristic_standard_action()` -- the rollout hack converting EndTurn to
  BeginPositioning.
- `heuristic_select_positioning_character()` -- replaced by the assignment
  generator.
- `heuristic_assign_column()` -- replaced by the assignment generator.
- The `rollout_action()` match arms for `SelectPositioningCharacter` and
  `AssignColumn` -- during rollouts, positioning is now atomic.

The existing `uct_search.rs` retains all of these for the old `MonteCarlo`
variant.

## Evaluation Plan

After implementation, run the new AI against the old AI using the existing
`ai_matchup` harness:

```
just matchup '{"MonteCarlo": 50}' '{"MonteCarloV2": 50}' --matches 20 --deck core-11
```

The harness alternates starting player on odd match indices, covering both
configurations. Key metrics:
- Win rate of new vs old AI.
- Average game length (longer games suggest more defensive play, which is
  expected and desirable if the new AI is properly evaluating blocking).
- Decision log analysis: are the positioning candidates reasonable? Is MCTS
  finding meaningful reward differences between candidates?

If the new AI is weaker, the decision logs will reveal whether the problem is
candidate generation (bad options) or MCTS evaluation (good options but wrong
choice).
