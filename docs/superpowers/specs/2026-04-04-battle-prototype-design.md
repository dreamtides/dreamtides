# Battle Prototype Web Client — Design Spec

A web-based battle client for prototyping Dreamtides battles against AI, built
with React/Vite/TypeScript/Tailwind. Connects to the existing Rust dev server
via HTTP, reusing the same API the Unity client uses. No animations, no sound —
just a functional, playable battle UI with card art and rules text.

## Architecture

### Communication Model

The web client talks to the existing dev server (port 26598) via HTTP. The Vite
dev server proxies `/connect`, `/perform_action`, `/poll`, and `/log` to avoid
CORS issues (the dev server has no CORS middleware).

**Flow:**

1. On load, send `ConnectRequest` to `GET /connect` with `DebugConfiguration`
   (specifying deck and AI opponent) as query parameters or JSON body.
2. Parse `CommandSequence` from the response. Extract the last `UpdateBattle`
   command and use its `BattleView` to render the initial state.
3. When the player clicks a card or button, POST `PerformActionRequest` with the
   `GameAction` embedded in that card/button.
4. Poll `/poll` every 200ms until `PollResponseType::Final`. On each
   `Incremental` response, update the `BattleView` (so AI moves are visible as
   they happen).
5. On `Final`, update the `BattleView` and re-enable interactions.

All animation/VFX commands (`FireProjectile`, `DissolveCard`, `PlayAudioClip`,
`Wait`, etc.) in the `CommandSequence` are ignored. Only `UpdateBattle` commands
are processed.

### Version Tracking

Each response includes a `response_version` UUID. The client sends
`last_response_version` on each `PerformActionRequest` to avoid stale state.

### State Management

A single React context holds the current `BattleView` (or null before connect).
No derived state, no local game logic — the server is the single source of
truth.

During polling (after sending an action), all interactions are disabled (cards
greyed out, buttons disabled) until the `Final` response arrives.

## Project Structure

Lives at `scripts/battle_prototype/`, mirroring the quest prototype's structure:

```
scripts/battle_prototype/
├── package.json
├── vite.config.ts          # Vite config with proxy to dev server
├── tsconfig.json
├── index.html
├── scripts/
│   └── setup-assets.mjs    # Symlink card art from image cache
├── public/
│   ├── cards/              # Symlinked .webp card art
│   └── tides/              # Tide icon PNGs
└── src/
    ├── main.tsx
    ├── App.tsx
    ├── index.css
    ├── api/                # Server communication
    │   └── client.ts       # connect, performAction, poll functions
    ├── state/
    │   └── battle-context.tsx  # React context for BattleView
    ├── types/
    │   └── battle.ts       # TypeScript types mirroring Rust display_data
    ├── components/
    │   ├── BattleScreen.tsx     # Main layout container
    │   ├── PlayerStatus.tsx     # Score, energy, spark, deck/void counts
    │   ├── BattlefieldZone.tsx  # Row of up to 8 characters
    │   ├── StackZone.tsx        # Center column stack display
    │   ├── HandZone.tsx         # Bottom hand of cards
    │   ├── CardDisplay.tsx      # Single card rendering
    │   ├── ActionBar.tsx        # InterfaceView buttons
    │   ├── OverlayPrompt.tsx    # FlexNode text/button extraction
    │   └── DebugPanel.tsx       # Debug controls
    └── util/
        ├── command-parser.ts    # Extract BattleView from CommandSequence
        └── flex-node-parser.ts  # Extract text/buttons from FlexNode tree
```

## Asset Pipeline

**Card images:** A setup script (based on the quest prototype's
`setup-assets.mjs`) symlinks card art from
`~/Library/Caches/io.github.dreamtides.tv/image_cache/` into
`public/cards/{cardNumber}.webp`. The script parses `rendered-cards.toml` to
build a card-number-to-image mapping, output as `public/card-data.json`.

**Tide icons:** Copied from `client/Assets/ThirdParty/GameAssets/Tides/` into
`public/tides/`.

**Card metadata (cost, spark, rules text, etc.) is NOT sourced from the asset
pipeline.** It comes from the server's `RevealedCardView` in each `BattleView`
response. The asset pipeline only provides images.

**Image mapping:** The server sends
`DisplayImage::Sprite(SpriteAddress { sprite })` for card art. The `sprite`
string must be mapped to a card number to find the correct `.webp` file. The
`card-data.json` file (generated from `rendered-cards.toml`) provides this
mapping.

## Layout

Classic TCG vertical layout:

```
+-------------------------------------+
|         Enemy Status Bar            |
|   Score | Energy | Spark | Deck/Void|
+-------------------------------------+
|        Enemy Battlefield            |
|   [Card] [Card] [Card] ...  (max 8)|
+-------------------------------------+
|     +--- THE STACK ---+             |
|     | [Card] [Card]   |             |
|     | (newest first)  |             |
|     +-----------------+             |
+-------------------------------------+
|        Your Battlefield             |
|   [Card] [Card] [Card] ...  (max 8)|
+-------------------------------------+
|         Your Status Bar             |
|   Score | Energy | Spark | Deck/Void|
+-------------------------------------+
|           Your Hand                 |
|  [Card] [Card] [Card] [Card] [Card]|
+-------------------------------------+
|          Action Bar                 |
|   [Primary] [Secondary] [Undo]     |
+-------------------------------------+
```

**Stack zone:** Centered between the two battlefields, gold-bordered. Cards
displayed horizontally, newest highlighted. Collapses to a thin line or
disappears when empty.

**Card positions are determined by `CardView.position`:**

- `Position::InHand(DisplayPlayer::User)` → hand zone
- `Position::OnBattlefield(DisplayPlayer::User)` → your battlefield
- `Position::OnBattlefield(DisplayPlayer::Enemy)` → enemy battlefield
- `Position::OnStack(...)` → stack zone
- `Position::InDeck(...)` → deck count only
- `Position::InVoid(...)` → void count only
- `Position::InBanished(...)` → banished count only
- Other positions → not displayed

## Card Rendering

Each card shows:

- **Art image** — from the symlinked `.webp` file, matched via sprite address
- **Name** — from `RevealedCardView.name`
- **Energy cost** — from `RevealedCardView.cost` (pre-formatted string)
- **Spark** — from `RevealedCardView.spark` (pre-formatted string, characters
  only)
- **Card type** — from `RevealedCardView.card_type`
- **Rules text** — from `RevealedCardView.rules_text` (HTML-formatted)
- **Fast badge** — if `RevealedCardView.is_fast` is true
- **Outline color** — from `RevealedCardView.outline_color`, indicates
  playability (green = playable, white/yellow = valid target)

**Face-down cards** (`CardView.card_facing == FaceDown` or `revealed` is None)
render as card backs showing only the card prefab type.

**Card interactions:**

- Cards with `actions.can_play` set → clickable, sends the embedded `GameAction`
  on click
- Cards with `actions.on_click` set → clickable (for targeting), sends the
  embedded `GameAction`
- Cards with neither → not interactive

## Interface Controls

From `InterfaceView`:

- `primary_action_button` → main action (Pass Priority, End Turn, Start Next
  Turn, Submit, confirm choices)
- `secondary_action_button` → alternative choice
- `undo_button` → undo last action
- `dev_button` → toggle debug panel
- `increment_button` / `decrement_button` → for energy cost selection prompts
- `browser` → card browser modal (for viewing void, etc.)
- `card_order_selector` → Foresee card ordering UI

Each `ButtonView` has a `label` and optional `GameAction`. If `action` is None,
the button renders as disabled.

## FlexNode Overlay Handling

`InterfaceView.screen_overlay` is an optional `FlexNode` tree used for prompt
messages and panel overlays (including buttons).

Rather than building a full flex layout renderer, we recursively walk the
`FlexNode` tree and extract:

1. **Text content** — any node with `node_type: Text(TextNode { label })` →
   rendered as text
2. **Buttons** — any node with `event_handlers.on_click` set → rendered as a
   clickable button, with the label extracted from its child `TextNode`

Extracted content renders as a simple overlay: semi-transparent backdrop, prompt
text on top, buttons in a row below. Styling, positioning, images, animations,
drag-and-drop, sliders, and text fields from the FlexNode tree are ignored.

## Prompts and Targeting

The server handles all prompt logic. The client just needs to:

1. **Render card highlights** — cards with `actions.on_click` get a colored
   border matching `outline_color`
2. **Render buttons** — `InterfaceView` buttons for confirm/cancel/choices
3. **Render overlay text** — from `screen_overlay` FlexNode extraction
4. **Send actions on click** — clicking a highlighted card or button sends its
   embedded `GameAction`

Specific prompt types handled:

- **Character targeting** — battlefield cards highlighted, click to select
- **Stack card targeting** — stack cards highlighted, click to select
- **Void card selection** — void browser opens, cards selectable
- **Hand card selection** — hand cards highlighted, click to select
- **Choice prompts** — primary/secondary buttons show choice labels
- **Modal effect choices** — buttons for each option
- **Energy cost selection** — increment/decrement buttons
- **Card ordering (Foresee)** — card order selector UI with drag or click
  reordering
- **Mulligan** — hand cards selectable, submit button

## Debug Panel

A collapsible panel (toggled via `dev_button`) with controls mapping to existing
`DebugAction` and `DebugBattleAction` values:

- **Restart with deck** — dropdown of `TestDeckName` values (Vanilla,
  StartingFive, Benchmark1, Core11), sends `DebugAction::RestartBattleWithDecks`
- **Set energy to 99** — sends `DebugBattleAction::SetEnergy`
- **Draw card** — sends `DebugBattleAction::DrawCard`
- **Add enemy character** — sends `DebugBattleAction::AddCardToBattlefield`
- **Opponent continue** — sends `DebugBattleAction::OpponentContinue`
- **Set deck to 1 card** — sends `DebugBattleAction::SetCardsRemainingInDeck`

## Visual Style

Dark theme, functional but not flashy:

- Background: `#0a0612` (matches quest prototype)
- Card borders: tide-colored when applicable, green for playable, gold for stack
- Text: light grey/white on dark backgrounds
- Status bars: compact, clear typography
- No animations, no hover effects, no transitions
- Desktop-only (no mobile/responsive considerations)

## Tech Stack

- **Vite 7.x** — bundler/dev server with proxy config
- **React 19.x** — UI framework
- **TypeScript 5.x** — strict mode
- **Tailwind CSS 4.x** — styling

## Out of Scope

- Animations, particle effects, sound
- Card hover zoom/preview
- Multiplayer (two humans)
- Deck building UI
- Mobile/responsive layout
- Full FlexNode layout rendering (only text and button extraction)
- Dreamcaller/Dreamsign special rendering

## Implementation Milestones

Each milestone must be verified with extensive `agent-browser` manual QA before
proceeding to the next. No milestone is considered complete without visual
evidence that it works correctly. See the QA Requirements section at the bottom
for the full protocol — it applies to every milestone and every subagent.

### Milestone 1: Scaffold & Connect

Set up the Vite project, proxy config, and successfully call `/connect`. Log the
`BattleView` to the console.

**QA:**

1. Use `agent-browser` to open the app URL. Take a screenshot.
2. Verify the page loads without errors (no white screen, no console errors).
3. Render the raw `BattleView` JSON on screen (or a summary: player names, card
   counts per zone, turn number, phase).
4. Take a screenshot and confirm the BattleView contains expected data: cards in
   hand, cards in deck, turn 1, etc.
5. Reload the page and verify the connection succeeds again (not a fluke).
6. Take a screenshot after reload and compare — state should be consistent.

### Milestone 2: Static Zone Rendering

Parse `BattleView` and render all zones with placeholder card rectangles showing
card name, cost, and spark values. Render player status bars with score, energy,
spark, and deck/void counts.

**QA:**

01. Use `agent-browser` to take a screenshot of the full layout.
02. Count cards in hand zone — must match the number of `InHand(User)` cards in
    the BattleView. Log the expected count and the rendered count.
03. Count cards on your battlefield — must match `OnBattlefield(User)` cards.
04. Count cards on enemy battlefield — must match `OnBattlefield(Enemy)` cards.
05. Verify deck count number matches the number of `InDeck(User)` cards.
06. Verify void count number matches the number of `InVoid(User)` cards.
07. Verify player score, energy, and spark values match `PlayerView` data.
08. Verify enemy score, energy, and spark values match enemy `PlayerView` data.
09. Verify turn number is displayed and correct.
10. Take a final screenshot confirming all zones are visually distinct and no
    cards are missing or duplicated.

### Milestone 3: Card Art & Details

Wire up card art from the image cache. Display rules text, cost, spark, type,
and fast badge on each card using data from `RevealedCardView`.

**QA:**

1. Use `agent-browser` to take a screenshot of the hand zone.
2. For each card in hand: verify the image loads (no broken image icon), the
   name is displayed and readable, the energy cost is shown, spark is shown for
   characters.
3. Take a screenshot of the battlefield zone. Verify character cards show spark
   values and rules text.
4. Verify at least one card's rules text contains ability keywords and is
   properly formatted (not raw HTML tags visible).
5. If any fast cards are present, verify the fast badge appears.
6. Verify face-down cards (enemy hand, deck) render as card backs, not
   broken/empty.
7. Restart the battle (via direct API call or page reload) to get a different
   hand. Take another screenshot and verify the new cards also render correctly
   — this catches hardcoded or cached rendering bugs.
8. Check for broken images across all zones. Every revealed card must have art
   or a clear fallback.

### Milestone 4: Basic Interactions

Wire click handlers to `GameAction` values on cards (`can_play`, `on_click`) and
`InterfaceView` buttons. Send actions to the server, poll for response,
re-render with updated `BattleView`.

**QA — Play a full turn sequence:**

01. Take a screenshot of the initial state. Note hand size, energy, and
    battlefield state.
02. Identify a playable card (green outline / `can_play` set). Click it.
03. Take a screenshot. Verify: the card moved to the stack or battlefield, hand
    size decreased by 1, energy decreased by the card's cost.
04. If the card is on the stack, click Pass Priority (or verify it auto-
    resolves). Take a screenshot showing the card resolved to the battlefield.
05. Play a second card if energy permits. Take a screenshot after each action.
06. Click End Turn. Take a screenshot. Verify: it is now the AI's turn, the UI
    shows a disabled/waiting state.
07. Wait for the AI turn to complete (poll returns Final). Take a screenshot.
    Verify: the AI has played cards (battlefield changed), it is now your turn
    again, energy has reset.
08. Play through at least 3 full turns (yours + AI). Take screenshots each turn.
    Verify the game state progresses: turn number increases, energy production
    increases (dreamwell), cards accumulate on battlefields.
09. Verify the judgment phase: after enough turns, one player should have more
    spark. Verify score changes are reflected in the status bar.
10. If the game reaches a point where no cards are playable, verify Pass
    Priority and End Turn still work correctly.

### Milestone 5: Prompts & Targeting

Handle all prompt types: character/stack/void/hand targeting, choice prompts,
modal effects, energy cost selection, card ordering (Foresee), mulligan.

**QA — Test each prompt type:**

1. **Mulligan:** On game start, if a mulligan prompt appears, take a screenshot.
   Click cards to select/deselect for mulligan. Take a screenshot showing
   selected cards are highlighted. Click Submit. Verify the hand changes. If no
   mulligan prompt exists by default, use debug actions to restart and trigger
   one.
2. **Character targeting:** Use debug panel to set energy to 99, then play a
   card that targets a character (e.g., a dissolve effect). Take a screenshot
   showing valid targets are highlighted on the battlefield. Click a target.
   Take a screenshot showing the target was dissolved (removed from battlefield,
   appears in void count). Verify spark totals updated.
3. **Stack targeting:** If any prevent effects exist in the test decks, play one
   in response to an opponent's card on the stack. Take a screenshot showing the
   stack card is targetable. If not testable with available cards, note this as
   a gap.
4. **Choice prompt:** Play a card with a "Choose one:" modal. Take a screenshot
   showing the choice buttons. Click one. Verify the chosen effect resolves
   correctly.
5. **Energy cost selection:** If any cards have variable energy costs, test the
   increment/decrement buttons. Take screenshots showing the value changes.
6. **Card ordering (Foresee):** Play a Foresee card. Take a screenshot showing
   the card order selector. Reorder cards and submit. Verify the deck state
   changed (this may only be verifiable indirectly by drawing and seeing the
   expected card).
7. **Void card selection:** If any reclaim effects are available, test selecting
   cards from the void. Take a screenshot showing void cards are browsable and
   selectable.
8. After testing individual prompt types, play through a full game (at least 5
   turns) using a deck that triggers multiple prompt types. Take screenshots
   throughout. Verify no prompts get stuck or cause the UI to become
   unresponsive.

### Milestone 6: FlexNode Overlay

Extract text and buttons from `screen_overlay` FlexNode trees. Render as a
prompt banner with clickable buttons.

**QA:**

1. Trigger a game state that produces a screen overlay. Take a screenshot.
   Verify: overlay text is readable, the backdrop dims the game behind it.
2. If the overlay contains buttons, verify each button is visible with its
   label. Click a button. Take a screenshot showing the overlay resolved and the
   correct action was taken.
3. Open the developer panel (via dev_button). Take a screenshot. Verify all
   debug buttons are visible and labeled correctly.
4. Click each debug button in the developer panel and verify it works:
   - "Restart Battle" → game resets, take screenshot
   - "Draw Card" → hand size increases by 1, take screenshot
   - "99 Energy" → energy display updates, take screenshot
5. Close the developer panel. Verify the overlay disappears and the game is
   interactive again.
6. Play through 2 full turns after interacting with overlays to verify the
   overlay system doesn't leave the game in a broken state.

### Milestone 7: Debug Panel

Implement the collapsible debug panel with restart-with-deck-selection, set
energy, draw card, and other debug controls.

**QA:**

1. Open the debug panel. Take a screenshot showing all controls.
2. **Restart with Benchmark1 vs Benchmark1:** Click the restart button. Take a
   screenshot of the new game. Note the cards in hand.
3. **Restart with Core11 vs Core11:** Click the restart button with a different
   deck. Take a screenshot. Verify the hand contains different cards than the
   Benchmark1 game.
4. **Set energy to 99:** Click the button. Take a screenshot. Verify energy
   display shows 99. Play an expensive card to confirm the energy is actually
   usable.
5. **Draw card:** Click multiple times. Take a screenshot after each. Verify
   hand size increases each time (up to the 10-card hand limit).
6. **Add enemy character:** Click the button. Take a screenshot. Verify an enemy
   character appeared on the enemy battlefield.
7. **Opponent continue:** Trigger a state where the opponent needs to act, then
   click this button. Verify the opponent takes an action.
8. **Set deck to 1:** Click the button. Draw a card. Verify the deck is nearly
   empty. Draw again — verify the hand-full-draw-gives-energy behavior or empty
   deck behavior works.
9. After using debug controls, play through 3 full turns to verify the game is
   still in a valid, playable state.

### Milestone 8: Polish Pass

Dark theme styling, status bar refinement, stack collapse when empty, disabled
state visual feedback during polling, general visual cleanup.

**QA:**

1. Take a screenshot of the initial game state. Verify: dark background
   (#0a0612), readable text contrast, card borders are visible.
2. Play a card and take a screenshot during the polling/disabled state. Verify:
   cards are visually greyed out or otherwise clearly non-interactive, buttons
   show a disabled state.
3. Verify stack zone when empty: take a screenshot showing no stack. It should
   collapse or show a minimal indicator, not a large empty box.
4. Play a card to put it on the stack. Take a screenshot showing the stack zone
   expanded with the gold border and card details.
5. Play enough turns to fill the battlefield to 8 characters. Take a screenshot.
   Verify all 8 fit without overlapping or breaking layout.
6. Play a 9th character and verify the abandon mechanic fires (lowest spark
   character removed, spark bonus added to status).
7. Continue playing until judgment phase scores points. Take a screenshot
   showing the score changed.
8. Play a full game to completion (one player reaches 12 points). Take a
   screenshot of the end state. Verify the game-over state is clear.

### Milestone 9: Extended Playtesting & Bug Fixing

This milestone consists of 10 sequential playtest-and-fix rounds. Each round
involves playing a complete battle from start to finish using `agent-browser`,
identifying every bug or visual issue encountered, fixing all issues found, and
then verifying the fixes before starting the next round.

**Each round follows this exact protocol:**

1. Start a new battle (alternate between deck configurations each round:
   Benchmark1 vs Benchmark1, Core11 vs Core11, Vanilla vs Vanilla, StartingFive
   vs StartingFive, and mixed matchups).
2. Play the entire game from turn 1 to completion (one player reaches 12 points
   or 50 turns pass). Do not skip turns or rush — play every card you can,
   activate every ability available, trigger every prompt type that comes up
   naturally.
3. Take a screenshot at least once per turn. Take additional screenshots
   whenever anything looks wrong, unexpected, or unclear.
4. After each screenshot, analyze it carefully. Check for:
   - Cards in wrong zones or missing from display
   - Incorrect card counts (hand size, deck count, void count)
   - Wrong energy/spark/score values
   - Broken card images or unreadable text
   - Buttons that should be present but aren't (or vice versa)
   - Prompts that don't resolve or get stuck
   - UI elements overlapping or misaligned
   - Disabled state not showing during AI turns
   - Stack zone not collapsing/expanding properly
   - Any crash, freeze, or unresponsive state
5. Log every bug found with: a description, which turn it occurred on, and the
   screenshot showing the issue.
6. After the game ends (or if a blocking bug prevents completion), fix ALL bugs
   found in that round.
7. After fixing, replay the specific scenarios that triggered each bug to verify
   the fixes. Take screenshots confirming each fix.
8. Only then proceed to the next round.

**Round-specific focus areas (in addition to general play):**

- **Rounds 1-2:** Focus on basic flow — playing cards, passing priority, ending
  turns, AI turns completing. These rounds catch fundamental interaction bugs.
- **Rounds 3-4:** Focus on prompts and targeting — deliberately play cards that
  require targets, trigger choice prompts, use Foresee if available. Use debug
  panel to set up specific board states if needed.
- **Rounds 5-6:** Focus on edge cases — fill battlefield to 8 characters (test
  abandon), empty your deck (test empty deck behavior), play with 0 energy, have
  the stack contain multiple cards simultaneously.
- **Rounds 7-8:** Focus on the full game arc — play from start to finish without
  using debug tools. Verify the game ends correctly, scores are accurate, and
  the end state is clear. Try to reach high turn counts.
- **Rounds 9-10:** Stress testing — use debug panel aggressively (99 energy,
  draw many cards, add enemy characters, restart mid-game). Verify the client
  handles all debug actions gracefully without breaking the game state.

**Completion criteria for Milestone 9:** The final two rounds (9 and 10) must
complete with ZERO bugs found. If bugs are found in rounds 9 or 10, fix them and
add additional rounds until two consecutive rounds are clean.

## QA Requirements (applies to ALL milestones)

**This section is mandatory and non-negotiable. Every subagent prompt MUST copy
this section verbatim or link to it explicitly.**

Every implementation step — not just milestones but every meaningful code change
within a milestone — must be verified with `agent-browser` before proceeding.
Specifically:

1. **Take a screenshot** after each change using `agent-browser`.
2. **Analyze the screenshot carefully.** Do not glance at it — examine every
   visible element. Check card counts, stat values, zone contents, button
   labels, text readability, layout integrity.
3. **Compare against expected state.** If rendering server data, log the
   expected values (from the BattleView JSON) and compare against what is
   displayed on screen. Mismatches are bugs.
4. **Do not proceed** to the next step if the screenshot shows ANY errors,
   missing content, or incorrect rendering. Fix issues first, re-screenshot, and
   re-verify.
5. **Subagents must follow this same protocol.** Every subagent prompt must
   explicitly include these instructions:
   - "You MUST use `agent-browser` to take a screenshot and verify your work
     after every meaningful code change."
   - "You MUST NOT claim completion without screenshot evidence showing the
     feature works correctly."
   - "If a screenshot shows any issue, you MUST fix it before proceeding."
   - "When verifying game state, log the expected values from the server
     response and compare them against what is visible on screen."
6. **Volume of QA matters.** A single screenshot is not sufficient to verify a
   milestone. Each milestone should involve multiple screenshots across
   different game states. Milestones involving interactions (4+) should include
   screenshots from playing through multiple full turns.
7. **Subagents must play real games.** After implementing any interactive
   feature, the subagent must play at least 2 full turns of a real battle (not
   just click one button and call it done). For Milestone 9, full games from
   start to finish are required.
8. **Screenshot before AND after.** For any action (clicking a card, pressing a
   button), take a screenshot before the action and after, to verify the state
   transition was correct.
