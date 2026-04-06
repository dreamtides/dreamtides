---
name: ai-decisions
description: Use when analyzing AI battle decisions, investigating why the AI chose an action, reviewing MCTS search statistics, or debugging AI behavior. Triggers on AI decision log, MCTS analysis, why did the AI, ai-decisions, analyze AI decisions, AI playing poorly.
---

# AI Decision Log Analysis

Analyze JSONL logs from the MCTS search to answer questions about AI decision-making, search quality, and positioning behavior.

## Step 1: Get the Log

AI decision logs are written to `ai_decisions.jsonl` in the rules engine log directory. Find it:

```bash
find /Users/dthurn/dreamtides -name "ai_decisions.jsonl" -not -path "*/.claude/*" 2>/dev/null | head -5
```

If empty, the dev server must be running with `log_ai_decisions: true` (enabled by default in dev_server.rs).

## Step 2: JSONL Schema

Each line is one JSON object representing a single AI decision point.

### Top-Level Fields

| Field | Type | Description |
|-------|------|-------------|
| `timestamp` | string | ISO 8601 timestamp of the decision |
| `player` | string | "One" or "Two" — which player the AI controls |
| `chosen_action` | string | Debug format of the chosen action (e.g. `PlayCardFromHand(39)`) |
| `chosen_action_short` | string | Short format (e.g. `PCFH39`, `ET`, `BP`, `MCFR39P2`) |
| `chosen_avg_reward` | float | Average reward of the chosen action (-1.0 to 1.0) |
| `game_state` | object | Snapshot of the game state at decision time |
| `budget` | object | Iteration budget details |
| `action_results` | array | Per-action search statistics, sorted by avg_reward descending |

### ActionResult Fields

| Field | Type | Description |
|-------|------|-------------|
| `action` | string | Debug format of the action |
| `action_short` | string | Short format |
| `total_reward` | float | Sum of all rollout rewards |
| `visit_count` | int | Number of MCTS iterations for this action |
| `avg_reward` | float | total_reward / visit_count (-1.0 to 1.0) |
| `wins` | int | Rollouts where the AI won |
| `losses` | int | Rollouts where the AI lost |
| `draws` | int | Rollouts that ended in a draw |
| `tree_node_count` | int | Number of nodes in the search tree |
| `tree_max_depth` | int | Maximum depth reached in the search tree |

### BudgetDetails Fields

| Field | Type | Description |
|-------|------|-------------|
| `iterations_per_action` | int | Iterations allocated per action (after multiplier) |
| `base_iterations` | int | Iterations before multiplier |
| `total_iterations` | int | iterations_per_action * num_actions |
| `num_actions` | int | Number of legal actions considered |
| `multiplier` | float | Budget multiplier applied |
| `multiplier_reason` | string | Why this multiplier: `prompt` (0.5x), `first_main` (1.5x), `main` (1.0x), `other` (0.75x), `override` |
| `num_threads` | int | Rayon thread count |

### GameStateSnapshot Fields

| Field | Type | Description |
|-------|------|-------------|
| `turn_id` | int | Current turn number |
| `active_player` | string | "One" or "Two" |
| `phase` | string | Current phase (Main, Judgment, etc.) |
| `player_one` | object | PlayerSnapshot for player one |
| `player_two` | object | PlayerSnapshot for player two |

### PlayerSnapshot Fields

| Field | Type | Description |
|-------|------|-------------|
| `points` | int | Current victory points |
| `points_to_win` | int | Points needed to win |
| `current_energy` | int | Available energy |
| `produced_energy` | int | Energy produced this turn |
| `hand_size` | int | Cards in hand |
| `battlefield` | object | BattlefieldSnapshot with `front` and `back` arrays |

### BattlefieldSnapshot

`front` and `back` are arrays of 8 elements (columns 0-7). Each element is either `null` (empty slot) or:

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Card display name (e.g. "Glimmer Scout") |
| `spark` | int | Current spark value |
| `id` | int | Internal card ID |

## Step 3: Common Queries

### "Why did the AI pick this action?"

```python
import json
entries = [json.loads(l) for l in open("ai_decisions.jsonl")]
# Show decision at a specific turn
for e in entries:
    if e["game_state"]["turn_id"] == TURN:
        print(f"Chose: {e['chosen_action']} (avg_reward={e['chosen_avg_reward']:.3f})")
        for r in e["action_results"]:
            print(f"  {r['action_short']:12s} avg={r['avg_reward']:+.3f} wins={r['wins']} losses={r['losses']} draws={r['draws']} visits={r['visit_count']}")
```

### "How noisy is the signal?"

Actions where win rate is close to 50% have very noisy signals:

```python
for e in entries:
    for r in e["action_results"]:
        total = r["wins"] + r["losses"] + r["draws"]
        if total > 0:
            win_rate = r["wins"] / total
            if 0.35 < win_rate < 0.65:
                print(f"Turn {e['game_state']['turn_id']} {r['action_short']}: win_rate={win_rate:.2f} ({r['wins']}W/{r['losses']}L/{r['draws']}D)")
```

### "Which decisions were closest?"

Smallest gap between top-2 actions means the AI was most uncertain:

```python
for e in entries:
    results = e["action_results"]
    if len(results) >= 2:
        gap = results[0]["avg_reward"] - results[1]["avg_reward"]
        print(f"Turn {e['game_state']['turn_id']}: gap={gap:.4f} — chose {results[0]['action_short']} over {results[1]['action_short']}")
```

### "What was the board state?"

```python
def show_battlefield(e):
    gs = e["game_state"]
    for pname in ["player_one", "player_two"]:
        p = gs[pname]
        print(f"\n{pname} (pts={p['points']}/{p['points_to_win']} energy={p['current_energy']}/{p['produced_energy']} hand={p['hand_size']})")
        for rank_name in ["front", "back"]:
            chars = []
            for i, slot in enumerate(p["battlefield"][rank_name]):
                if slot:
                    chars.append(f"[{i}]{slot['name']}({slot['spark']})")
            if chars:
                print(f"  {rank_name}: {', '.join(chars)}")
```

### "How deep did search go?"

```python
for e in entries:
    for r in e["action_results"]:
        if r["tree_max_depth"] > 0:
            print(f"Turn {e['game_state']['turn_id']} {r['action_short']}: depth={r['tree_max_depth']} nodes={r['tree_node_count']}")
```

### "How is the iteration budget?"

```python
for e in entries:
    b = e["budget"]
    print(f"Turn {e['game_state']['turn_id']}: {b['iterations_per_action']}iter/action * {b['num_actions']}actions = {b['total_iterations']}total (base={b['base_iterations']} x{b['multiplier']} [{b['multiplier_reason']}])")
```

### "What positioning choices were made?"

```python
positioning_actions = ["BP", "SCFP", "MCFR"]
for e in entries:
    if any(e["chosen_action_short"].startswith(p) for p in positioning_actions):
        print(f"Turn {e['game_state']['turn_id']}: {e['chosen_action']} (avg={e['chosen_avg_reward']:.3f})")
        for r in e["action_results"][:5]:
            print(f"  {r['action_short']:12s} avg={r['avg_reward']:+.3f}")
```

## Step 4: Action Short Code Reference

| Code | Meaning |
|------|---------|
| `PCFH{id}` | PlayCardFromHand |
| `PCFV{id}` | PlayCardFromVoid |
| `AAFC{id}` | ActivateAbilityForCharacter |
| `PP` | PassPriority |
| `ET` | EndTurn |
| `SNT` | StartNextTurn |
| `BP` | BeginPositioning |
| `SCFP{id}` | SelectCharacterForPositioning |
| `MCFR{id}P{col}` | MoveCharacterToFrontRank (character id, column) |
| `MCBR{id}P{col}` | MoveCharacterToBackRank |
| `SCT{id}` | SelectCharacterTarget |
| `SSCT{id}` | SelectStackCardTarget |
| `SVCT{id}` | SelectVoidCardTarget |
| `SHCT{id}` | SelectHandCardTarget |
| `SPC{n}` | SelectPromptChoice |
| `SEAC{n}` | SelectEnergyAdditionalCost |
| `SMEC{n}` | SelectModalEffectChoice |

## Step 5: When Logs Cannot Answer the Question

If you need **rollout-level data** (what happens during random playouts), that requires adding logging inside the `evaluate()` function in `uct_search.rs`. This is expensive and should only be done for targeted debugging:

1. Add a sampling flag (e.g. log 1 in 100 rollouts)
2. Capture rollout length, terminal scores, and key positioning decisions
3. Add to `ActionResult` as an optional `sample_rollouts` array

If you need **tree structure data** (which child nodes were explored), the search tree is available in `ActionSearchResult.graph`. The existing DOT graph logging (`log_ai_search_diagram`) provides a 3-depth visualization of the best action's tree.

### Procedure for Adding New Logging

1. Identify the data gap
2. Add fields to structs in `rules_engine/src/ai_uct/src/decision_log.rs`
3. Collect data in the appropriate function in `rules_engine/src/ai_uct/src/uct_search.rs`
4. Run `just fmt && just review`
5. Update this skill with the new fields
