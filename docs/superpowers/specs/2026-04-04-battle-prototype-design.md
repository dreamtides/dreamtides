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

**Tide icons:** Copied from
`client/Assets/ThirdParty/GameAssets/Tides/` into `public/tides/`.

**Card metadata (cost, spark, rules text, etc.) is NOT sourced from the asset
pipeline.** It comes from the server's `RevealedCardView` in each `BattleView`
response. The asset pipeline only provides images.

**Image mapping:** The server sends `DisplayImage::Sprite(SpriteAddress {
sprite })` for card art. The `sprite` string must be mapped to a card number to
find the correct `.webp` file. The `card-data.json` file (generated from
`rendered-cards.toml`) provides this mapping.

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
- Cards with `actions.can_play` set → clickable, sends the embedded
  `GameAction` on click
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
  StartingFive, Benchmark1, Core11), sends
  `DebugAction::RestartBattleWithDecks`
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

Each milestone must be verified with `agent-browser` screenshot QA before
proceeding to the next. No milestone is considered complete without visual
evidence that it works correctly.

### Milestone 1: Scaffold & Connect

Set up the Vite project, proxy config, and successfully call `/connect`. Log
the `BattleView` to the console.

**QA:** Use `agent-browser` to open the app URL. Verify the page loads without
errors. Check browser console (or render the raw BattleView JSON on screen) to
confirm the dev server connection works and a valid BattleView was received.
Take a screenshot and verify.

### Milestone 2: Static Zone Rendering

Parse `BattleView` and render all zones with placeholder card rectangles
showing card name, cost, and spark values. Render player status bars with
score, energy, spark, and deck/void counts.

**QA:** Use `agent-browser` to take a screenshot. Verify: correct number of
cards in each zone matches the BattleView data, player stats (score, energy,
spark) are displayed and match server state, all zones are visually
distinguishable. Compare card counts against the raw BattleView to confirm
accuracy.

### Milestone 3: Card Art & Details

Wire up card art from the image cache. Display rules text, cost, spark, type,
and fast badge on each card using data from `RevealedCardView`.

**QA:** Use `agent-browser` to take a screenshot. Verify: card images load
correctly (no broken images), rules text is readable, cost and spark values
are displayed, fast cards show the fast badge. Check at least 3 different
cards across different zones.

### Milestone 4: Basic Interactions

Wire click handlers to `GameAction` values on cards (`can_play`, `on_click`)
and `InterfaceView` buttons. Send actions to the server, poll for response,
re-render with updated `BattleView`.

**QA:** Use `agent-browser` to play a card from hand. Take screenshots before
and after. Verify: the card leaves the hand, appears on the stack or
battlefield (depending on timing), energy decreases, the BattleView updates.
Test Pass Priority and End Turn buttons. Verify the AI takes its turn and
the board state changes.

### Milestone 5: Prompts & Targeting

Handle all prompt types: character/stack/void/hand targeting, choice prompts,
modal effects, energy cost selection, card ordering (Foresee), mulligan.

**QA:** Use `agent-browser` to play a card that requires targeting (e.g., a
dissolve effect). Take screenshots showing: the prompt appears, valid targets
are highlighted, clicking a target resolves the effect. Test at least one
choice prompt and one targeting prompt. Verify each with screenshots.

### Milestone 6: FlexNode Overlay

Extract text and buttons from `screen_overlay` FlexNode trees. Render as a
prompt banner with clickable buttons.

**QA:** Use `agent-browser` to trigger a game state that produces an overlay
(e.g., a panel or prompt message). Take a screenshot. Verify: overlay text
is readable, overlay buttons are visible and clickable, clicking a button
sends the correct action and the overlay resolves. Test with the developer
panel overlay specifically.

### Milestone 7: Debug Panel

Implement the collapsible debug panel with restart-with-deck-selection, set
energy, draw card, and other debug controls.

**QA:** Use `agent-browser` to open the debug panel, restart a battle with a
different deck (e.g., switch from Benchmark1 to Core11). Take screenshots
before and after restart. Verify: new game starts, hand contains different
cards, game state resets. Test set-energy-to-99 and draw-card buttons.

### Milestone 8: Polish Pass

Dark theme styling, status bar refinement, stack collapse when empty, disabled
state visual feedback during polling, general visual cleanup.

**QA:** Use `agent-browser` to play through several turns of a complete game.
Take screenshots at key moments: empty stack, full battlefield (8
characters), judgment phase scoring, game end. Verify: all states render
cleanly, no visual glitches, disabled state is visually clear during AI
turns.

## QA Requirements (applies to ALL milestones)

Every implementation step — not just milestones but every meaningful code
change within a milestone — must be verified with `agent-browser` before
proceeding. Specifically:

1. **Take a screenshot** after each change using `agent-browser`.
2. **Analyze the screenshot** to confirm the change works as expected.
3. **Compare against expected state** — if rendering server data, verify the
   displayed values match what the server sent.
4. **Do not proceed** to the next step if the screenshot shows errors, missing
   content, or incorrect rendering. Fix issues first.
5. **Subagents must follow this same protocol.** Every subagent prompt must
   explicitly state that `agent-browser` screenshot verification is required
   after each code change, and that the subagent must not claim completion
   without screenshot evidence.
