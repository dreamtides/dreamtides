# Combat Model Redesign

## Overview

Revise the Dreamtides battle system to use a new positional combat model with
column-based attacker/blocker mechanics, delayed attacks, and post-judgment
repositioning. Changes affect the Rust rules engine and the web battle
prototype. No changes to the C# Unity client.

## Battlefield Layout

- 8 columns, each player has a front rank and back rank (8 slots each).
- A player can have at most **8 total characters** on the battlefield across
  both ranks. If a player has 8 characters, character cards are unplayable.
- Characters enter play in the back rank and cannot move to the front rank on
  the turn they were played (summoning sickness — unchanged from current).

## Repositioning

- During the main phase only, the active player may freely reposition their
  characters: between front and back ranks, and between columns within a rank.
- Characters cannot be repositioned during the Ending or Judgment phases.
- Drag-and-drop in the prototype UI (unchanged mechanism).

## Attacker/Blocker Classification

Determined per-column at the moment Judgment begins (not locked in advance):

- A front-rank character is an **attacker** if the opposing front-rank space in
  its column is empty.
- A front-rank character is a **blocker** if there is an opposing front-rank
  character directly across from it in the same column.
- Back-rank characters never participate in Judgment.

## Phase Ordering

**New order:** Dreamwell → Draw → Dawn → Main → Ending → Judgment → EndOfTurn

The Ending phase (non-active player's fast-action window) moves to before
Judgment. This gives the opponent a chance to play fast-speed cards (e.g.
dissolve an attacker) before combat resolves. Active player can only respond to
opponent cards during Ending (unchanged rule).

No action window exists after Judgment — the turn ends immediately.

## Judgment Phase Resolution

During the active player's Judgment phase, the **non-active player's**
front-rank characters are evaluated as attackers. For each column 0–7:

1. **Non-active player has a front-rank character, active player does not:**
   Unblocked attacker. Scores victory points equal to its spark.

2. **Both players have a front-rank character:** Spark comparison (judgment).
   - Lower spark is dissolved (sent to Void).
   - Equal spark: both dissolved.
   - The attacker does **not** score points even if it survives.

3. **Only the active player has a front-rank character:** Nothing happens. No
   one is attacking in this column.

## Post-Judgment

- **Dissolved characters** go to Void as normal.
- **All surviving participants** (both attackers and blockers that won their
  spark comparison) return to the **back rank**.
- **Non-participants** (front-rank characters in columns where no judgment
  occurred) remain in the front rank.
- When returning to back rank, a character slots into the back-rank position
  directly behind its column (same column index). Existing back-rank characters
  shift to make room. This is a visual/positional convention for clarity; it
  does not affect gameplay.
- Given the 8-character limit and dissolves freeing slots, the back rank will
  always have room.

## Timing of Attacks

Attacks are delayed by one turn:

1. Player A places characters in front rank during their main phase.
2. Player A's turn ends → Ending phase (Player B may play fast cards) →
   Judgment phase evaluates Player B's front-rank characters as attackers.
3. Player B's turn → Player B positions blockers → Player B's turn ends →
   Ending phase (Player A may play fast cards) → Judgment evaluates Player A's
   front-rank characters as attackers.

Characters placed in the front rank during your turn will not attack until the
end of your opponent's next turn.

## Character Limit

- Maximum 8 characters per player on the battlefield (both ranks combined).
- If a player has 8 characters, character cards in hand are not playable (no
  play action offered, same mechanism as insufficient energy).

## Spark and Card Effects

- Spark may be modified by card effects before Judgment begins.
- Once Judgment begins, no new cards can be played in response.
- Existing spark mechanics (gain, base value, modifications) are unchanged.

## Victory Condition

Unchanged: first player to reach `points_to_win` (currently 25) wins.

## Rules Engine Changes

1. **Judgment phase resolution** (`judgment_phase.rs`): Flip attacker evaluation
   to non-active player. Add post-judgment return-to-back-rank logic for
   surviving participants.

2. **Phase ordering** (`battle_turn_phase.rs` and phase transition logic): Swap
   Ending and Judgment in the phase sequence.

3. **Character limit** (card playability logic): Add check that player has < 8
   battlefield characters before allowing a character card to be played.

4. **Back-rank slot assignment**: When returning characters to back rank after
   judgment, prefer the slot behind the character's current column. Shift
   existing back-rank characters as needed.

## Battle Prototype UI Changes

1. **Judgment event generation** (`battle-context.tsx`): Update `generateEvents()`
   to describe non-active player's characters as attackers with appropriate log
   messages.

2. **Post-judgment positioning**: Surviving participants visually move to back
   rank (same column). Back-rank characters shift to accommodate.

3. **Judgment pause display**: Existing modal continues to work with updated
   event text.

4. **Character limit**: No specific UI needed — unplayable cards are handled by
   the rules engine not offering play actions.

5. **Phase display**: UI reflects Ending before Judgment (server-driven via
   polling).

## QA Plan

Manual QA via subagents using the `qa` skill against the battle prototype:

- Verify attacker/blocker classification matches column positions
- Verify non-active player's characters attack during active player's Judgment
- Verify spark comparisons resolve correctly (higher wins, ties both dissolve)
- Verify unblocked attackers score points equal to spark
- Verify surviving participants return to back rank after Judgment
- Verify non-participants remain in front rank
- Verify 8-character limit prevents playing a 9th
- Verify summoning sickness prevents front-rank movement on play turn
- Verify Ending phase occurs before Judgment (fast cards can remove attackers)
- Verify repositioning only works during main phase
