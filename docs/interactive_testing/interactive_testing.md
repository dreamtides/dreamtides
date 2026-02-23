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
- **"X removed"**: History entries like "Sundown Surfer removed" appear when
  triggered abilities modify a card (e.g., adding spark). The card is NOT being
  removed from play — this message represents an internal state update. Track
  the actual spark values in the snapshot to see what changed.
- **"X removed" during reclaim**: When a Reclaim Ability is used, the original
  card moves from void to stack and the reclaim entry is "removed" from hand.
  This is normal.

## Known ABU Limitations

### Modal Cards

Modal cards (e.g., "Break the Sequence — Choose One: ...") show "cost: " with an
empty cost in hand. When played, choice cards appear in the Browser group. Click
the desired choice to select it.

### Opponent Responses and Sundown Surfer Triggers

When you play cards during your turn, the AI opponent may respond with fast
events (Guiding Light, Dreamscatter, Ripple of Defiance, Abolish, Immolate).
**Each opponent response triggers Sundown Surfer's "When you play a card during
the opponent's turn" ability**, adding +1 spark per Surfer per response. This
can dramatically escalate the opponent's spark total. Strategy: avoid playing
non-essential cards when the opponent has Sundown Surfers and energy to respond.

### Discard Selection

When an effect requires discarding from hand (e.g., Astral Interface's "Draw a
card. Discard a card."), click the card you want to discard. The snapshot won't
visually indicate the selection, but a Submit button appears. Click Submit to
confirm. The hand count stays at 10 until after you submit.

### Ripple of Defiance Payment

When Ripple of Defiance targets your event, you'll see "Spend 2●" and "Decline"
buttons. Click "Spend 2●" to pay the tax and keep your event, or "Decline" to
let it be prevented. The opponent can chain multiple Ripples, each requiring a
separate 2 energy payment.

## Testing Strategy

1. **Play a full game** to test basic card flow, scoring, and victory
2. **Focus on interactions**: Stack responses, counter-spells, triggered
   abilities
3. **Test edge cases**: Hand size limit (10 cards), character limit (8), energy
   overflow from Dreamwell
4. **Document bugs immediately**: Stop and write a detailed description when
   something seems wrong
5. **Take screenshots** when the snapshot doesn't match expected UI state

## Setup: Restarting a Fresh Game

After `clear-save`, you need to exit and re-enter play mode to start a new game.
The `cycle` command may fail with "Assets > Refresh menu item not found".
Workaround:

```sh
python3 scripts/abu/abu.py clear-save
python3 scripts/abu/abu.py play          # exit play mode (if active)
sleep 3
python3 scripts/abu/abu.py play          # re-enter play mode
sleep 5
python3 scripts/abu/abu.py snapshot --compact
```

Check `status` between steps to confirm play mode toggled correctly.

## Game Strategy Notes

### General Principles

- **Hold fast cards until the ending phase.** A common mistake is playing cards
  as soon as possible during your main phase. Fast cards (events and abilities
  marked fast) can be played during the ending phase after you click End Turn.
  Playing them then denies the opponent a chance to respond with their own fast
  cards before you pass. If you play fast events during your main phase, the
  opponent gets priority and can respond — potentially triggering Sundown
  Surfers or disrupting your plan.
- **Minimize stack interactions when the opponent has energy.** Every card you
  put on the stack during your main phase gives the opponent a window to respond
  with fast cards. If the opponent has Sundown Surfers, each response grows
  their spark. Play characters and essential non-fast cards, then save fast
  cards for the ending phase.
- **Activated abilities are safe.** They don't go to the stack and don't give
  the opponent priority. Use these freely even when the opponent has energy and
  Surfers.

### Early Game (Turns 1-3)

- Play characters with high spark first (e.g., Minstrel of Falling Light at
  spark 2 is better than Sundown Surfer at spark 1)
- Guiding Light (cost 1, Foresee + draw) is excellent early tempo
- Establish board presence before using activated abilities
- Don't waste Abolish or other counter-spells early — save them for high-value
  targets later

### Mid Game (Turns 4-6)

- Hold counter-spells (Abolish, Cragfall, Ripple of Defiance) for the opponent's
  key plays. These are fast and reactive — they can only be played in response
  to cards on the stack
- Watch for the opponent deploying Sundown Surfers. If they have even one
  Surfer, be cautious about how many cards you play during your main phase
- Start tracking the spark gap and planning around Judgment scoring

### Late Game (Turns 7+)

- Scoring accelerates as spark totals grow. A single turn with a large spark gap
  can end the game
- Immolate becomes the highest-value card — dissolving a 5+ spark Surfer swings
  the gap by 5+ in your favor
- Archive of the Forgotten (cost 4, returns up to 2 events from void) is the key
  recovery tool for getting Immolates back after they've been used
- If behind on spark, avoid ending your turn unless you can close the gap first
  — the opponent scores the full difference at Judgment

### Scoring

- Points are scored during Judgment (start of each turn) by the active player if
  their spark exceeds the opponent's. Points = difference.
- Dreamwell cards can also grant points directly (e.g., Autumn Glade)
- Victory at 12 points. Monitor both scores relative to the threshold.
- A 6-spark lead sustained for 2 turns = 12 points = instant victory. Prevent
  large spark gaps from persisting across turns.

### The Sundown Surfer Spiral

Sundown Surfer is the most dangerous card in the Core11 mirror match. It gains
+1 spark each time its owner plays a card during the opponent's turn. With
multiple Surfers on the battlefield, each fast card the opponent plays during
your turn triggers ALL of them.

**How the spiral works:**

1. Opponent has 2-3 Sundown Surfers on the battlefield
2. During your turn, you play a card on the stack
3. Opponent responds with a fast card (Guiding Light, Break the Sequence, Ripple
   of Defiance, Cragfall, etc.)
4. Each Surfer triggers, gaining +1 spark per Surfer per card played
5. When you resolve the opponent's response, they may play another fast card,
   triggering all Surfers again
6. This repeats until the opponent runs out of energy or fast cards

In a real game, an opponent with 3 Surfers playing 5 fast cards during your turn
gains +15 spark from triggers alone — easily enough to score 12+ points at
Judgment and win immediately.

**Counter-strategies:**

- **Dissolve Surfers early with Immolate.** This is the #1 priority. Don't waste
  Immolate on Minstrels when Surfers are on the board.
- **Don't put cards on the stack during your main phase** when the opponent has
  Surfers + energy. Each response triggers the spiral. Use activated abilities
  instead (they don't use the stack).
- **Save fast cards for the ending phase.** After you click End Turn, you can
  still play fast cards. The opponent can respond, but you've already committed
  to ending — minimizing the window for responses.
- **Your own plays during your turn are safe.** Only the opponent's plays during
  your turn trigger their Surfers. Abolish and other cards you play do NOT
  trigger them.
- **Preserve Abolish for the opponent's Surfer plays.** If the opponent tries to
  play a Surfer, counter it before it reaches the battlefield.

### Key Card Interactions

- **Sundown Surfer**: Gains +1 spark each time its owner plays a card during the
  opponent's turn. This compounds rapidly. See "The Sundown Surfer Spiral"
  above.
- **Break the Sequence**: Returns an enemy character to hand. The opponent loves
  using this during your turn to bounce your characters while triggering
  Surfers. History does not log the target's zone transition (known bug).
- **Reclaim abilities**: Appear in hand as separate entries. Cost is the Reclaim
  cost, not the original card cost. Reclaimed cards go to banished (not void)
  when they leave play.
- **Activated abilities**: Cost energy but don't go to the stack. They appear as
  hand entries with the ability cost. Safe to use when the opponent has Surfers.
- **Ripple of Defiance**: Forces the opponent to pay 2 energy or have their
  event prevented. When targeting your event, you see "Spend 2●" and "Decline"
  buttons.
- **Archive of the Forgotten**: Returns up to 2 events from void to hand.
  Critical for recovering Immolates. Plan the Archive → Immolate sequence when
  the opponent has high-spark Surfers.

## Known Bugs and History Gaps

### Break the Sequence Missing History Entries

When Break the Sequence resolves and returns an enemy character to hand, the
target character's zone transition does NOT appear in history. Only "Break the
Sequence moved from hand to stack" and "Break the Sequence moved to void" are
logged. The affected character silently disappears from the battlefield. To
detect this, compare battlefield character counts before and after the action.
The void count will be unchanged since the character goes to its owner's hand,
not to the void.

### Opponent Can Chain Fast Cards During Your Stack Resolution

When you play a card and the opponent responds, resolving their response may
trigger further opponent plays before your original card resolves. This can lead
to deeply nested stacks. Be patient and keep clicking Resolve until the stack
fully empties. Always check the stack in the snapshot — if a Stack group exists,
cards are still pending.

## Abu Usage Tips for Agents

### Activated Abilities and Virtual Hand Entries

The hand count displayed in the snapshot (e.g., "Hand (13 cards)") includes
virtual entries for activated abilities and reclaim abilities. These do NOT
count toward the 10-card hand limit. To estimate real hand cards, subtract the
number of activated ability entries (one per battlefield character with an
ability) and reclaim ability entries (one per void card with Reclaim).

### Discard Selection

When selecting cards to discard, clicking virtual entries (Activated Ability,
Reclaim Ability) does NOT produce a Submit button since they are not valid
discard targets. Only click real cards (Character, Event) when discarding.

### Foresee Interactions

When Foresee triggers during dreamwell resolution (e.g., Skypath), the Card
Order Selector appears and you must Submit before the draw phase proceeds. Just
click Submit to keep the default order if you want to leave cards where they
are.

### Archive of the Forgotten Browser Selection

When Archive resolves, a Browser group appears showing void contents. Click 1-2
events to select them, then click Submit. Note: if the opponent responds to the
Archive being on the stack, the browser may appear before the Archive actually
resolves — the Archive still needs to go through stack resolution. Be prepared
for multiple resolve cycles.

### Dreamscatter Energy Requirement

Dreamscatter costs 2 energy to play PLUS requires paying at least 1 additional
energy for the draw effect. So the minimum total cost is 3 energy. The "can
play" tag only appears when you have enough for both the play cost and minimum
payment.

### Abolish Timing

Abolish only shows "can play" when there is a valid target on the stack (a
played card to prevent). It cannot be played proactively. It activates during
the opponent's turn when they play cards.

## Writing Narrative Files

Write playtest narratives to `/tmp/narrative.md`. Include:

- Game state at key decision points (energy, score, spark, hand)
- Actions taken and their results
- Opponent responses and triggered abilities
- Any bugs or unexpected behavior encountered

## Turn Sequencing Advice

A recommended turn structure for playing well:

1. **Assess the board**: Check spark totals, opponent energy, and opponent
   Surfer count before making any plays
2. **Play characters first**: These must go on the stack, so get them out while
   you still have energy to respond if the opponent plays fast cards
3. **Use activated abilities**: These are safe (no stack, no opponent priority).
   Draw cards, filter your deck
4. **Click End Turn**: This enters the ending phase
5. **Play fast cards during the ending phase**: Guiding Light, removal spells,
   and other fast events are best played here. The opponent can still respond,
   but the turn is already ending — limiting their window
6. **Hold counter-spells**: Keep Abolish and Together Against the Tide in hand
   for the opponent's turn. They'll show "can play" when the opponent puts
   something on the stack
