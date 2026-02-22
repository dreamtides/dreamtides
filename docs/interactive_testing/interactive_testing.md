# Interactive Testing Guide

Guide for AI agents performing interactive playtesting of Dreamtides via Abu.

## Setup

```sh
python3 scripts/abu/abu.py status      # verify Unity is running
python3 scripts/abu/abu.py clear-save  # reset to fresh game state
python3 scripts/abu/abu.py play        # enter play mode (if not active)
sleep 5                                # wait for play mode to stabilize
python3 scripts/abu/abu.py snapshot --compact  # initial game state
```

Always wait a few seconds after entering play mode before taking the first
snapshot. The TCP server needs time to start.

## Core Testing Workflow

1. Take a `--compact` snapshot to see the current game state
2. Identify available actions from the snapshot
3. Perform an action (drag card to Play Zone, click button, etc.)
4. Parse the history and updated snapshot from the response
5. Repeat

Use `snapshot --compact` for routine state checks. Use full `snapshot` (no flag)
when debugging missing UI elements.

## Playing Cards

Cards are played by dragging them to the Play Zone target:

```sh
just abu drag <card_ref> <play_zone_ref>
```

The Play Zone ref changes with every snapshot. Always re-read the play zone ref
from the most recent snapshot.

## Understanding the Snapshot

Key elements in a typical battle snapshot:

- **User > Status**: Energy (current/max), Score, Spark, Turn indicator
- **User > Battlefield**: Characters in play with spark values
- **User > Hand**: Cards with cost, type, abilities, and "can play" tags
- **Opponent > Status**: Same stats for the opponent
- **Actions**: End Turn, Resolve, Submit, Next Turn (context-dependent)
- **Play Zone**: Drop target for playing cards (ref changes each snapshot)
- **Stack**: Cards currently on the stack awaiting resolution
- **Card Order Selector**: Appears during Foresee (reorder/void cards)

### Card Display Format

Cards in hand show:

```
"Name, Type (cost: N, spark: N, can play) -- Rules text"
```

- `can play` indicates the card is currently playable
- Activated abilities appear as separate hand entries
- Reclaim abilities appear as hand entries with "(Reclaimed)" tag
- Modal cards show "cost: " (empty) since cost depends on mode choice

### Identifying Card Types

- **Character**: Has "spark: N" in its description
- **Event**: No spark value, one-time effect
- **Activated Ability**: Shows "Activated Ability" type, references a
  battlefield character
- **Reclaim Ability**: Shows "Reclaim Ability" type, plays from void

## Action Responses and History

Every action (click, drag) returns a fresh snapshot plus a `--- History ---`
section. History entries describe what happened:

- Card zone transitions: "X moved from hand to stack"
- Opponent actions: "Enemy drew a card", "Enemy dreamwell: ..."
- Turn markers: "Your turn begins", "Opponent's turn begins"
- Game events: "Victory", "Defeat"
- Dreamwell activations: "Dreamwell: Name -- effect"

**IMPORTANT**: Refs are invalidated after every action. Always use refs from the
most recent snapshot.

## Responding to Game Prompts

### Targeting (e.g., "Dissolve an enemy")

When a card requires a target, the snapshot shows an interface message like
"Dissolve an enemy." Click the target directly:

```sh
just abu click <target_ref>
```

### Foresee (Card Reordering)

During Foresee, a Card Order Selector appears with Deck Positions and a Void
slot. Drag cards between positions to reorder, or drag to Void to discard. Click
Submit when done. To keep the default order, just click Submit.

### Energy Payment (e.g., Dreamscatter)

For variable-cost effects, use the +1/−1 buttons to adjust the amount, then
click "Spend N●" to confirm.

### Stack Interaction

When cards are on the stack:

- **Resolve** button: Let the top card resolve
- You can play fast cards in response by dragging them to Play Zone
- The opponent may also respond (their plays appear in history)

Stack resolves LIFO (last-in, first-out).

### End of Turn Flow

- **End Turn**: Pass during your main phase
- **Next Turn**: Acknowledge opponent's turn end, advance to your turn
- During opponent's turn, you may still play fast cards

## Common Gotchas

- **Chaining abu commands in bash**: When chaining multiple abu.py calls with
  `&&`, each returns a separate snapshot. Only the last snapshot's refs are
  valid. Prefer sequential single commands.
- **Refs change on every action**: Never reuse refs from a previous snapshot.
  The ref for "End Turn" will be different after each action.
- **"can play" tag**: Only cards marked "can play" can be played right now.
  Missing this tag means insufficient energy, wrong phase, or card-specific
  restrictions.
- **Activated abilities in hand**: When a character with an activated ability is
  on the battlefield, the ability appears as a separate card in your hand. Drag
  it to the Play Zone to activate.
- **Reclaim abilities**: Cards with Reclaim can be played from the void. They
  appear as "Reclaim Ability" entries in your hand after the original goes to
  the void.

## Interpreting History Messages

Some history messages can be misleading:

- **"moved to void"** vs **"moved to banished"**: Reclaimed cards go to banished
  instead of void when they leave play.

## Testing Strategy

1. **Play a full game** to test basic card flow, scoring, and victory
2. **Focus on interactions**: Stack responses, counter-spells, triggered
   abilities
3. **Test edge cases**: Hand size limit (10 cards), character limit (8), energy
   overflow from Dreamwell
4. **Document bugs immediately**: Stop and write a detailed description when
   something seems wrong
5. **Take screenshots** when the snapshot doesn't match expected UI state

## Writing Narrative Files

Write playtest narratives to `/tmp/narrative.md`. Include:


- Game state at key decision points (energy, score, spark, hand)
- Actions taken and their results
- Opponent responses and triggered abilities
- Any bugs or unexpected behavior encountered
