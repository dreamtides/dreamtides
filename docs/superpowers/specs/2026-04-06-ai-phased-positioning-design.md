# AI Phased Positioning Design

## Context

After introducing the column-based judgment combat model, the AI's MCTS
branching factor during the main phase exploded to 18-100+ actions (card plays +
repositioning combined). This destroys search quality because iteration budget
is fixed — more actions means fewer iterations per action. The goal is to reduce
branching to ~6 per node by splitting the AI's turn into sequential sub-phases
with low branching at each step.

## Design

### AI Turn Sub-Phases

The AI's main phase is decomposed into two sequential sub-phases tracked via new
state in `TurnData`. Human players are unaffected — they continue to see the
full interleaved action set.

**Phase 1 — Card Play.** The AI plays cards and activates abilities.

Legal actions:

- `PlayCardFromHand(id)` for each affordable card
- `PlayCardFromVoid(id)` for each playable void card
- `ActivateAbilityForCharacter(id)` for each activatable ability
- `BeginPositioning` — irreversibly transition to Phase 2
- `EndTurn` — skip positioning entirely

Typical branching: 5-10, shrinks as cards are played.

**Phase 2 — Positioning.** A two-step loop for assigning back-rank characters:

**Step A — Select Character.** Pick which back-rank character to move forward.

- One action per eligible back-rank character (no summoning sickness, not
  already assigned this turn)
- `EndTurn` — done positioning

Typical branching: 2-5, shrinks as characters are assigned.

**Step B — Assign Column.** Pick where the selected character goes.

- One action per column containing an opponent front-rank character (blocking)
- One "attack" action if there exists an empty column where neither player has a
  front-rank character (all such columns are equivalent)

Typical branching: 2-5 (opponent front chars + 1 attack option).

After Step B completes, return to Step A with the assigned character removed.

### Invariants

- `BeginPositioning` is irreversible — no card plays after entering Phase 2.
- Character column assignments are irreversible — once placed, the character
  stays in that front-rank column for the rest of the turn.
- Front-rank characters from previous turns are committed — no retreating in V1.
- Characters not selected in Step A stay in back rank automatically.
- When there are no eligible back-rank characters, Step A offers only `EndTurn`.

### State Tracking

New fields in `TurnData`:

```rust
/// True once the AI has entered the positioning sub-phase. Irreversible
/// within a turn. Only used when computing legal actions for Agent players.
pub ai_positioning_started: bool,

/// The character selected in positioning Step A, awaiting column assignment
/// in Step B. Cleared after the assignment is applied.
pub ai_positioning_character: Option<CharacterId>,
```

Both fields are cleared on turn change (existing `TurnData` reset logic).

### New BattleAction Variants

```rust
/// AI transitions from card-play sub-phase to positioning sub-phase.
BeginPositioning,

/// AI selects a back-rank character to move forward. The character awaits
/// column assignment via a subsequent MoveCharacterToFrontRank action.
SelectCharacterForPositioning(CharacterId),
```

`BeginPositioning` sets `turn.ai_positioning_started = true`.

`SelectCharacterForPositioning(id)` sets
`turn.ai_positioning_character = Some(id)`.

`MoveCharacterToFrontRank(id, col)` during Step B: moves the character, clears
`ai_positioning_character`, adds to `moved_this_turn`.

### New LegalActions Variants

```rust
/// Positioning Step A: select which back-rank character to move forward.
/// EndTurn is always available (done positioning).
AiSelectPositioningCharacter {
    eligible: CardSet<CharacterId>,
},

/// Positioning Step B: assign the selected character to a column.
AiAssignColumn {
    character: CharacterId,
    /// Columns containing opponent front-rank characters (blocking targets).
    block_targets: Vec<u8>,
    /// First empty column with no character from either player, if any.
    attack_column: Option<u8>,
},
```

### Legal Actions Routing

In `legal_actions::compute()`, for AI players during the main phase:

```
if !ai_positioning_started:
    → Standard actions (cards, abilities, EndTurn) + BeginPositioning
    → reposition_to_front and reposition_to_back are empty
elif ai_positioning_character.is_none():
    → AiSelectPositioningCharacter { eligible back-rank chars }
    → EndTurn always available
elif ai_positioning_character.is_some():
    → AiAssignColumn { character, block_targets, attack_column }
```

### Simulation / Rollout Behavior

During MCTS rollouts, BOTH players use the phased positioning system. This is
detected via `action_history.is_none()` (simulations always have `None` action
history). The random policy selects uniformly from available actions at each
step, producing reasonable behavior for both players:

- Phase 1: randomly plays some cards, eventually picks `BeginPositioning`
- Step A: randomly picks a character (or ends turn)
- Step B: randomly picks a column

No player-type differentiation during simulations — both players behave
identically with respect to action generation.

### Files to Modify

| File                                                            | Change                                                                                                                                        |
| --------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| `battle_state/src/battle/turn_data.rs`                          | Add `ai_positioning_started`, `ai_positioning_character` fields                                                                               |
| `battle_state/src/actions/battle_actions.rs`                    | Add `BeginPositioning`, `SelectCharacterForPositioning` variants                                                                              |
| `battle_queries/src/legal_action_queries/legal_actions.rs`      | Route AI main-phase actions through sub-phase state machine                                                                                   |
| `battle_queries/src/legal_action_queries/legal_actions_data.rs` | Add `AiSelectPositioningCharacter`, `AiAssignColumn` variants + implement `all()`, `len()`, `find_missing()`, `random_action()`, `contains()` |
| `battle_mutations/src/actions/apply_battle_action.rs`           | Handle `BeginPositioning` and `SelectCharacterForPositioning`                                                                                 |
| `ai_agents/src/agent_search.rs`                                 | Remove old forced-repositioning code (already done)                                                                                           |

### Branching Factor Analysis

| Decision Point            | Typical Actions | Max Actions                  |
| ------------------------- | --------------- | ---------------------------- |
| Phase 1 (Card Play)       | 5-10            | ~15 (full hand + abilities)  |
| Step A (Select Character) | 2-4             | 9 (8 chars + EndTurn)        |
| Step B (Assign Column)    | 2-4             | 9 (8 opponent cols + attack) |
| Priority/Stack            | 1-5             | ~10 (fast cards only)        |
| Prompts                   | 2-4             | varies                       |

Average across all decision points: ~5-7. The main-phase explosion from 18-100+
is eliminated.

### Verification

1. `just fmt && just review` passes
2. All existing battle tests pass (new action types don't affect human player
   flow)
3. Manual play: AI makes intelligent blocking decisions (strong blockers vs
   strong attackers)
4. AI-vs-AI matchup shows improved win rate over random baseline
