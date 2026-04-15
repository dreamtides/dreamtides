# Quest Prototype Battle Support Spec

Date: 2026-04-15

## Summary
Add a playable battle mode to `scripts/quest_prototype/` that stays fully in
TypeScript, does not call `rules_engine/`, and does not implement automatic
card-text resolution. It should support a real battle loop with turn flow,
energy, Judgment, scoring, victory, defeat, draw, and reward handoff.

Card text is handled through trusted debug tooling that lets the user edit
battle state directly. The prototype should automate bookkeeping, not legality.
It should not try to prevent impossible moves, illegal timing, or zone changes.
The debug surface is the primary interaction model for playable battle, not a
hidden fallback for exceptional cases.

The current auto-resolve battle flow must remain available. The playable mode
must activate only behind a runtime URL flag.

## Scope
In scope:
- opening hands, draw, turn flow, main phase, and ending
- simplified energy refresh and spending
- playing characters and events from hand
- reserve and deployed battlefield positions in a staggered grid
- Judgment combat and point scoring
- victory, defeat, draw, and forced-result transitions
- very simple opponent heuristics
- strong manual debug tools for simulating card effects
- reuse of current battle reward and quest progression behavior on victory

Out of scope:
- any Rust integration
- automatic event resolution
- automatic Dreamcaller, Dreamsign, or Dreamwell effects
- stack, priority, or fast-response timing
- a generic effect engine
- full AI

## Delivery Cut
Phase 1 is the implementation target for playable battle support.

Phase 1 must include:
- runtime toggle and routing
- battle-entry initialization from quest state
- battle state, turn flow, Judgment, scoring, and result detection
- playable battle UI with hand, board, status strips, and result surfaces
- prominent skip-to-victory shortcut that jumps straight to reward selection
- simple AI that obeys energy costs
- shared battle-entry and victory-completion seams that preserve current reward
  generation, reward application, and atlas progression behavior
- inspectable defeat and draw flow with explicit reset action
- typed debug actions for the minimum useful set:
  zone moves, position moves, numeric edits, force result, undo, redo,
  opponent-hand reveal controls, and banish moves
- deck, hand, void, and banished browsers
- single-card deck top and deck bottom moves from debug and browser surfaces
- logging and automated tests

Phase 2 follow-up work:
- create-copy and figment creation tooling
- Foresee overlay and reusable multi-card deck-order tooling
- temporary note system and note expiry UI
- prevent/copied markers and related debug affordances
- compact battle-log drawer polish
- additional debug shortcuts beyond the Phase 1 minimum

## Recommendation
Build battle as an isolated module under `src/battle/` and integrate it into
quest mode through a thin battle-site adapter. Do not evolve the current
`BattleScreen` into a full battle engine.

Recommended split:
- `AutoBattleScreen`: current fake-battle animation and reward flow
- `PlayableBattleScreen`: new battle UI and local battle state
- `BattleScreen`: wrapper that chooses mode from runtime config

Recommended integration helpers:
- `src/battle/integration/runtime-config.ts`
- `src/battle/integration/create-battle-init.ts`
- `src/battle/integration/complete-battle-site.ts`

## Runtime Toggle
Use a runtime URL flag parsed once near app startup:
- `?battle=playable`: use the new playable battle screen
- missing or any other value: keep current auto-resolve behavior

Do not store this in `QuestState`. Expose it through a small runtime-config
helper.

## Current Integration Seam
Today, battle sites route to a `BattleScreen` that:
- shows a pre-battle splash
- plays a fake battle animation
- grants essence and a reward card
- increments completion and advances the atlas

That reward and progression logic should be extracted into a shared contract
used by both battle modes.

Recommended victory completion bridge input:
- `siteId`
- `dreamscapeId`
- `completionLevelAtBattleStart`
- `atlasSnapshot`
- `selectedRewardCard`
- `essenceReward`
- `isMiniboss`
- `isFinalBoss`
- `playerHasBanes`

Required victory completion bridge behavior, in order:
1. grant essence
2. add the chosen reward card
3. mark the battle site visited
4. increment completion level
5. log battle and site completion
6. if not final boss, transition to atlas
7. if a dreamscape is active, generate replacement nodes, update atlas, log
   dreamscape completion, and clear `currentDreamscape`

The playable battle module must not reimplement this sequence itself.

Quest-level logging ownership:
- the completion bridge emits the persistent quest-progression events such as
  `battle_won`, `site_completed`, `dreamscape_completed`, and
  `screen_transition`
- the battle module emits only `battle_proto_*` events for session-local
  actions and state changes

## Session Stability And Trust Model
The battle session only needs to be stable after entry, but battle
initialization should still be deterministic for a given `BattleInit.seed`.

At battle entry, create and freeze a session snapshot:
- initialization seed
- enemy descriptor
- player deck order
- enemy deck definition and order
- reward card options
- any other battle-only metadata needed for the session

Required guarantees:
- reward options are generated once at battle entry and reused for the full
  battle session
- the same `BattleInit.seed` produces the same enemy descriptor, player deck
  order, enemy deck definition, and reward options
- rerenders, undo/redo, and result overlays must not reroll enemy flavor, deck
  order, or reward options
- the playable battle inspector is trusted developer tooling, not a safe player
  sandbox
- forced results and manual state edits are allowed to change the live quest
  run
- if the user forces or edits the battle into victory and then completes reward
  selection, the normal completion bridge runs

## Visual Reference
The battle UI should borrow the battlefield presentation from
`~/dreamtides-staggered`, especially:
- `scripts/battle_prototype/src/components/BattleScreen.tsx`
- `scripts/battle_prototype/src/components/BattlefieldZone.tsx`
- `scripts/battle_prototype/src/components/PlayerStatus.tsx`
- `docs/superpowers/specs/2026-04-07-staggered-grid-combat-design.md`
- `docs/superpowers/specs/2026-04-08-start-of-turn-judgment-design.md`

Borrow the visual language, not the Rust-facing legality model.

Preserve these cues:
- enemy zones above, player zones below
- separate reserve and deployed rows
- deployed row inset so the stagger is legible
- visible Judgment divider
- compact status bars for score, energy, deck, void, and hand counts
- ownership cues by side color
- compact battlefield cards and always-visible empty-slot frames
- modal overlays for hidden-zone browsing

When `?battle=playable` is active, the screen should look like a tactical
surface, not a reward animation.

## Battle Rules Slice
Supported rules:
- both players draw 5 cards
- turns proceed in fixed order
- the player always takes the first turn
- the first player skips the draw step on turn 1
- Judgment happens at the start of turn
- players gain and spend energy
- characters enter reserve when played by default
- events played from hand go to void by default
- Judgment resolves each deployed lane independently
- unblocked attackers score points equal to spark
- paired attackers and blockers compare spark and dissolve the weaker card
- tied spark dissolves both cards
- dissolved cards move to void
- surviving deployed characters stay in place
- first player to 25 points wins
- battle draws after 50 full turns
- drawing from an empty deck is a no-op

Use a 25-point threshold intentionally. With no automated card text,
Dreamcaller effects, or Dreamwell effects, a 12-point game would end too
quickly to exercise the debug surface and basic turn loop.

Explicit simplifications:
- no Dreamwell cards
- no stack
- no fast-response timing
- no automatic triggered abilities
- no automatic support-based spark calculations
- no legality enforcement for moves or timing

Canonical phase timeline:
- `startOfTurn`
- `judgment`
- `draw`
- `main`
- `endOfTurn`

Turn boundary rules:
- `Judgment` always resolves during `judgment`
- the first player's first turn skips `draw` but still passes through the phase
- normal player interaction happens during `main`
- `End Turn` is the only standard way to advance from `main` to the next turn

## Energy Model
Because Dreamwell is out of scope, use a fixed prototype ramp:
- each player starts with `maxEnergy = 0`
- at the start of that player's turn, `maxEnergy += 1`
- `currentEnergy` resets to `maxEnergy`
- default cap is 10

This is intentionally fake but predictable. The debug suite covers edge cases
that need bonus energy, reduced energy, or maximum-energy changes.

## Spark Model
Use one canonical spark value per card instance:
- `printedSpark` comes from card data, with `null` treated as `0`
- each card instance tracks `sparkDelta`, default `0`
- `effectiveSpark = max(0, printedSpark + sparkDelta)`
- only `effectiveSpark` is used for Judgment, scoring, UI summaries, and AI
  combat evaluation
- support relationships do not modify spark automatically in Phase 1
- `Kindle N` is a convenience wrapper around spark editing
- if no friendly character is selected, `Kindle N` targets the leftmost
  friendly battlefield character by slot order `D0-D3`, then `R0-R4`
- `sparkDelta` stays attached to the card instance until changed or removed

## Card Play Rules
- characters played from hand spend energy, leave hand, and default to reserve
- events played from hand spend energy and default to void
- fast cards still show their fast badge, but it is informational only
- player card play is never blocked for affordability or legality
- player energy may go negative
- AI card play must obey current energy

This keeps the loop honest without pretending card text is half-implemented.

## Battlefield Model
Each side has:
- deployed lanes `D0-D3`
- reserve slots `R0-R4`

The UI must clearly teach the staggered support map:
- `D0` is supported by `R0` and `R1`
- `D1` is supported by `R1` and `R2`
- `D2` is supported by `R2` and `R3`
- `D3` is supported by `R3` and `R4`

Support is visual only in Phase 1.

Positioning is permissive:
- characters may move freely between reserve and deployed slots
- a freshly played character may be placed directly into a deployed lane
- moving onto an occupied battlefield slot swaps the cards

Recommended row order:
- enemy reserve
- enemy deployed
- Judgment divider
- player deployed
- player reserve

Recommended presentation:
- always-on subtle labels for `R0-R4` and `D0-D3`
- support relationship highlighting on selection
- compact card body with cost, name, subtype, spark, and minimal text
- clear selected state distinct from hover
- temporary badge area for later notes or markers

## Battle Screen Anatomy
Top to bottom:
1. global battle status bar
2. enemy status strip
3. enemy reserve row
4. enemy deployed row
5. Judgment divider
6. player deployed row
7. player reserve row
8. player status strip
9. player hand
10. primary action bar

Persistent overlays:
- desktop debug inspector rail
- full-screen zone browser modal
- result modal or result overlay

### Global Battle Status Bar
Must show:
- turn number
- active side
- current phase
- score summary
- AI label if present

Judgment should have a visually strong phase treatment.

### Player Status Strips
Each side should show:
- score
- current energy
- maximum energy
- total visible deployed spark
- deck count
- void count
- hand count
- banished count if non-zero

Deck, void, hand, and banished counts should open the corresponding browser.

### Hand Area
The player hand stays as a dedicated horizontal strip near the bottom. It must
support:
- click to select and inspect
- primary action button or double click to play
- contextual actions for move, discard, banish, and reveal

Opponent hand is hidden by default and shown only through explicit debug
controls.

### Action Bar
Keep the main action bar minimal:
- End Turn
- Undo
- Redo
- Skip To Rewards
- Toggle Battle Log
- Toggle Debug Inspector on narrow screens

The battle UI should expose its debug affordances plainly. Inspector actions,
selection-driven actions, and zone-browser actions are primary controls, not
hidden developer shortcuts.

`Skip To Rewards` should be visually prominent and available directly on the
playable battle surface so testers can bypass the live battle loop once battle
testing is done.

Control grouping:
- `Battle Actions`: play, move, end turn, inspect hidden zones
- `Debug Actions`: numeric edits, zone overrides, reveal and hide controls,
  force result, and `Skip To Rewards`

Actions that violate the rules slice must always appear under a visible
`Debug` label with distinct visual treatment from normal battle actions.

## Debug UX Strategy
The debug suite is the core design problem. It must be fast enough to manually
simulate card text without turning into a giant admin form.
It is also the primary intended interaction model for this prototype battle
screen.

Recommended structure:
- persistent inspector rail on desktop
- context-sensitive quick actions for the selected card, slot, or zone
- full-screen browsers for deck, hand, void, and banished
- visible action history with undo and redo

Selection model:
- selectable battlefield cards
- selectable hand cards
- selectable empty battlefield slots
- selectable player and opponent summaries
- selectable zone headers

Important consequence: operations should usually be enabled by default. Prefer
"do the move and log it" over "block the move and explain why."
Do not hide core debug actions behind secret gestures, dev-only hotkeys, or
deeply nested menus.

## Interaction Model
Do not require explicit surface modes such as `Play`, `Move`, or `Debug`.
The user should remain on one overtly debug-capable surface throughout battle.

Phase 1 required path:
- single click to select a card, zone, or slot
- visible context actions in the inspector or action tray for play, move,
  reveal, numeric edit, and force-result operations
- browser footer or side-rail actions for hidden-zone moves
- this path must work on desktop and tablet without relying on drag-and-drop

Recommended hierarchy:
- single click: select
- double click or primary action: default action
- drag and drop: optional enhancement for visible moves
- inspector action: non-positional state edit
- browser action: hidden-zone move

Selection rules:
- only one primary target is selected at a time
- selection updates the inspector immediately
- selection is sticky across minor state edits when the target still exists
- if the target changes zone, selection follows the same card instance

Default actions:
- selected hand card: play
- battlefield card: select only
- empty battlefield slot with a selected card: move the selected card here
- deck or void header: open browser

### Responsive Requirements
Phase 1 must define explicit narrow-screen behavior:
- on tablet and narrow desktop, the inspector becomes an overlay drawer instead
  of a persistent rail
- the hand remains visible while the inspector is closed
- zone browsers use full-screen modal layout
- the action bar remains docked and reachable without scrolling past the hand
- the battlefield rows must remain readable without horizontal clipping

Drag-and-drop is optional on tablet. The click-select path is the supported
interaction model on all devices.

## Debug Operations
Phase 1 required operations:
- deploy and reserve characters
- add and remove current energy
- add and remove maximum energy
- add and remove points
- draw and discard cards
- move cards between all zones used in Phase 1
- freely reposition characters between battlefield slots
- change spark
- kindle
- inspect deck, hand, void, and banished
- move a selected card to deck top or deck bottom
- banish from battlefield, hand, or void
- edit both player and opponent state
- hide and reveal opponent hand cards
- force victory, defeat, or draw

Phase 2 follow-up operations:
- create a copy of an existing card
- create a generic figment token with chosen subtype and spark
- reveal the top card of deck and allow play from top of deck
- mark a card or action as prevented or copied
- add temporary notes
- trigger an extra Judgment
- grant an extra turn

## Debug Operation Model
Define debug actions as a typed command system, not ad hoc component callbacks.

Recommended categories:
- battle-flow operations
- zone-move operations
- battlefield-position operations
- numeric-state operations
- card-instance operations
- visibility operations
- result operations

Recommended operation shape:
- `id`
- `kind`
- `actor`
- `timestamp`
- `targets`
- `payload`
- `undoPayload`
- `sourceSurface`

Examples by category:
- battle-flow: end turn, force result
- zone-move: hand to reserve, void to hand, battlefield to void,
  battlefield to banished, any zone to deck top, any zone to deck bottom
- numeric-state: change energy, change max energy, change points, set spark,
  kindle
- visibility: reveal card, hide card, reveal opponent hand

Recommended command ids:
- `END_TURN`
- `PLAY_CARD`
- `MOVE_CARD`
- `DEBUG_EDIT`
- `FORCE_RESULT`
- `SKIP_TO_REWARDS`

## Undo Semantics
Undo must be exact, not simulated. Every operation must record enough state to
reverse itself without consulting derived game rules.

Requirements:
- one history entry per user action
- composite entries when one gesture causes several state changes
- undo follows exact card-instance ids
- redo reapplies recorded post-state, not a fresh recalculation
- automatic turn-start processing, Judgment resolution, AI main phase, and
  result entry each commit as one composite history step
- `Skip To Rewards` records one composite battle-history step that moves
  directly into the victory reward surface
- quest-level reward application after the user selects a reward is outside the
  battle undo domain

Required composite boundary:
- `END_TURN` is one history step that may include ending the active side's
  `main` phase, switching active side, incrementing turn count when needed,
  refreshing energy, resolving `Judgment`, processing draw, and running the AI
  side until control returns to the player or a result is reached

This boundary is authoritative for implementation and tests. Do not split
automatic start-of-turn processing or the AI main phase into separately undone
history entries.

Composite examples:
- "Play to D2" may spend energy, remove from hand, and place on battlefield
- "Force victory" may set result state and open the result surface
- "Kindle 2" may touch one card's spark and history

## Zone Browser Design
Required browsers:
- player deck
- player hand
- player void
- player banished
- opponent hand
- opponent deck
- opponent void
- opponent banished

Browser capabilities:
- search by card name
- sort by current order, acquisition order, cost, spark, and name
- optional type filter
- strong single-select support
- move selected card to any supported destination
- move selected cards to deck top or deck bottom one at a time in Phase 1
- hidden-card placeholder state for opponent hand
- reveal individual opponent-hand cards without globally revealing all

For deck and void browsing, show current order explicitly when order matters.

## AI Detail
Core rules:
- AI obeys energy costs and never intentionally spends energy it does not have
- AI uses the same permissive position model as the player
- AI never uses debug-only operations
- AI evaluates main-phase actions in this order:
  character play, battlefield reposition, single event play, end turn

Recommended play-order tiebreakers:
1. highest energy cost
2. character before event
3. higher printed spark
4. stable hand order

Recommended lane-choice tiebreakers:
1. open lane that creates immediate future scoring pressure
2. favorable trade
3. equal trade
4. leftmost lane

Recommended loop guard:
- cap AI main-phase actions per turn
- if no board-improving action exists, end turn
- if only no-op or circular repositioning remains, end turn

Event-play heuristic:
- if no affordable character play or reposition improves the board and at least
  one affordable event exists, play exactly one event this turn
- after playing one event, reevaluate once; if nothing improved, end turn

The goal is predictability, not strength.

## Logging
Use the existing `logEvent()` pipeline with a `battle_proto_*` prefix.

Ownership rule:
- the battle module logs only `battle_proto_*` events
- the shared completion bridge and quest state mutations continue to emit the
  existing quest-level events

Recommended common fields:
- `battleId`
- `turnNumber`
- `phase`
- `activeSide`
- `sourceSurface`
- `selectedCardId` when relevant

Additional fields by event family:
- movement: `cardId`, `from`, `to`
- numeric edits: `target`, `oldValue`, `newValue`, `delta`
- battle result: `winner`, `playerScore`, `enemyScore`, `reason`

At minimum log:
- battle initialized
- opening hands drawn
- phase changed
- energy changed
- card played
- card moved between zones
- position changed
- Judgment resolved per lane
- points changed
- AI action chosen
- debug action applied
- undo and redo
- battle won, lost, or drawn

## State Model
Battle state should live inside the battle module, not in global `QuestState`.

Recommended top-level concepts:
- metadata
- active player
- turn number
- phase
- per-side battle state
- battle history
- result state

Recommended metadata fields:
- `battleId`
- `siteId`
- `dreamscapeId`
- `completionLevelAtStart`
- `enemyDescriptor`
- `playerDreamcallerSummary`
- `playerDreamsignSummaries`

Each side should track:
- deck
- hand
- void
- banished
- battlefield positions
- current and maximum energy
- points
- pending extra turns
- visibility flags needed by the debug UI

Each card instance needs a stable battle-instance id so movement, spark edits,
history, and undo target the correct object.

Recommended initialization contract:
- `seed`: generated once at battle entry and used for all battle-only
  randomness
- `questDeckEntries`: copied from `QuestState.deck`, preserving `entryId`,
  `cardNumber`, `transfiguration`, and `isBane`
- `startingSide`: always `"player"` for Phase 1
- `battleCardId`: a new stable id for each in-battle instance, including cards
  created later by Phase 2 tooling
- `enemyDescriptor`: generated once at battle entry and then treated as session
  data
- `playerDeckOrder`: shuffled once at battle start and then frozen for the
  session
- `enemyDeckDefinition`: generated once at battle entry and then frozen for the
  session
- `rewardOptions`: the four battle reward cards generated once at battle entry
  and frozen for the session

Recommended Phase 1 enemy deck note:
- use real quest-prototype cards from `cardDatabase`, not synthetic placeholder
  cards
- derive one enemy accent tide from the generated enemy flavor
- prefer cards whose tide presentation matches the enemy accent tide
- treat `Neutral` as a display accent only, not as an assumed source of real
  candidate cards
- exclude cards with `energyCost = null`
- partition candidates into characters and events, then into simple cost bands
  `0-2`, `3-4`, and `5+`
- choose a 12-card enemy deck with this target mix:
  `8` characters and `4` events
  characters target `3` cheap, `3` mid, `2` expensive
  events target `2` cheap-or-mid and `2` remaining best-fit cards
- when a bucket is short, backfill in this order:
  matching-accent candidates, any non-starter numeric-cost card, then
  duplicates of already chosen candidates if still required
- shuffle final enemy deck order once for the session

Transfigurations and bane flags are visual only in battle mode.

## Quest Integration
Entering battle in playable mode:
- read the player's current quest deck into a `BattleInit` payload
- generate a battle seed once and include it in `BattleInit`
- preserve quest deck metadata needed for display and debug
- build a shuffled player battle deck
- build an enemy descriptor and enemy deck from real quest-prototype card data
- generate the four battle reward options once at entry and freeze them
- seed local battle state

Player Dreamsigns and Dreamcaller text should be displayed in side panels, but
remain informational unless the user manually applies their effects.

Leaving battle:
- on victory, present the frozen reward options from battle entry and then run
  the shared completion bridge
- `Skip To Rewards` should use that same frozen reward surface and completion
  bridge, skipping only the remaining battle simulation
- on defeat or draw, enter an inspectable result overlay on top of the live
  battle screen
- defeat and draw remain editable
- undo, redo, zone moves, and numeric edits stay available on defeat/draw
  result surfaces
- if a state-changing edit removes the defeat or draw condition, clear
  `resultState` and return to normal interaction
- resetting the run must be an explicit user action
- after reset confirmation, compute the failure summary, transition to a
  dedicated `questFailed` screen, and defer `resetQuest()` until the user
  starts a new run from that screen

Victory is intentionally not an inspectable post-result surface once the user
starts reward selection. After reward selection begins, battle undo is over.

Recommended `questFailed` behavior:
- explicit screen variant in the quest router
- screen payload:
  `result`, `turnNumber`, `playerScore`, `enemyScore`, `reason`, `battleId`
- the screen renders from its own payload, not from live battle state
- `resetQuest()` is called only from the `questFailed` primary action
- one primary action to start a new run

## File Boundaries
Recommended structure:
- `src/battle/types.ts`
- `src/battle/state/`
- `src/battle/engine/`
- `src/battle/ai/`
- `src/battle/debug/`
- `src/battle/components/`
- `src/battle/integration/`

Ownership:
- `battle/state`: reducer, initialization, and pure updates
- `battle/engine`: turn flow, Judgment, scoring, win checks
- `battle/ai`: greedy move selection
- `battle/debug`: typed debug actions, undo, and visibility control
- `battle/components`: board, hands, HUD, inspector, browsers, result UI
- `battle/integration`: runtime flag, battle init adapter, shared completion
  bridge, and failure-route adapter

## Automated Tests
Add pure battle-module tests first, then a small set of quest integration
tests.

Unit tests:
- opening-hand and deck-order stability after initialization
- reward-option stability after initialization
- energy spending behavior
- Judgment lane resolution
- dissolved cards move to void
- empty-deck draw is a no-op
- win and draw detection
- AI move selection in obvious board states
- permissive zone and position moves
- debug operation round trips
- undo and redo round trips

Integration tests:
- auto mode still grants rewards
- playable mode routes to the new surface
- playable mode freezes reward options for the full session
- victory updates quest progression identically to auto mode
- `Skip To Rewards` opens the same reward surface and uses the same completion
  bridge as normal victory
- debug-forced victory uses the same completion bridge and reward flow
- defeat and draw preserve an inspectable result surface before reset
- defeat and draw reach the game-over flow after explicit confirmation
- URL toggle switches cleanly between modes

## Manual QA
Manual QA should use `agent-browser` against the local Vite app.

Baseline regression:
- open `/`
- confirm battle still uses the current auto-resolve path
- confirm non-battle quest flow is unchanged
- confirm no new battle UI appears without the URL flag

Playable battle smoke path:
- open `/?battle=playable`
- start a run and enter battle
- verify opening hands, points, energy, deck counts, and turn indicator

Core QA:
- first player skips the first-turn draw
- energy resets to max at turn start
- maximum energy increments each turn
- Judgment happens before draw and main actions
- player character play goes to reserve by default
- player event play goes to void by default
- unaffordable player plays still go through
- reserve and deployed moves work
- occupied-position swaps work
- unblocked attackers score spark
- higher spark beats lower spark
- equal spark dissolves both and sends both to void
- drawing from an empty deck does nothing
- AI never takes an unaffordable play
- AI ends turn instead of looping
- every Phase 1 debug operation works once for both sides
- deck, void, banished, and both hands are browsable
- top and bottom deck moves work
- victory reward flow still works
- `Skip To Rewards` is visible and jumps directly to reward selection
- reward selection after `Skip To Rewards` grants the same quest rewards and
  progression as normal victory
- defeat and draw remain inspectable before reset
- reset only happens after explicit confirmation
- desktop and tablet layouts remain usable
- battle log entries are present and ordered
- no runtime errors appear in the dev console

## Implementation Sequence
Phase 1 sequence:
1. Extract the shared victory completion bridge and define the failure-route
   contract.
2. Add runtime mode selection and preserve auto mode.
3. Build battle-entry initialization from quest state.
4. Build pure battle state, turn flow, Judgment logic, and result detection.
5. Build the playable battle screen and board layout.
6. Add simple opponent AI.
7. Add typed debug actions for Phase 1 plus undo and redo.
8. Add zone browsers and opponent-hand visibility controls.
9. Add victory, defeat, and draw result surfaces.
10. Add automated tests.
11. Run the full manual QA sweep.

Phase 2 sequence:
1. Add figment and copy creation tools.
2. Add Foresee and deck-order tooling.
3. Add temporary note UI and richer debug shortcuts.
4. Add battle-log drawer polish and remaining debug affordances.

## Acceptance Criteria
- `?battle=playable` enables a playable battle instead of the fake animation
- no Rust battle code is called
- no card text is automatically executed
- the basic battle loop works end to end
- the Phase 1 debug suite can manually simulate common card effects efficiently
- auto mode remains available without the URL flag
- reward options are generated once per battle entry and stay stable for that
  session
- a prominent `Skip To Rewards` button is available in playable mode and uses
  the normal victory reward flow
- victory still grants the correct quest rewards and progression
- defeat and draw are inspectable before reset and have explicit game-over
  handling afterward
- manual QA confirms desktop and tablet usability
