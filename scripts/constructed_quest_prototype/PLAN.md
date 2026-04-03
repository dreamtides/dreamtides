# Technical Design: Dreamtides Quest Mode Web Prototype

## Goal

Build an interactive web prototype of Dreamtides Quest Mode for playtesting.
The prototype implements the full quest loop -- navigating a Dream Atlas,
visiting dreamscape sites, drafting cards via a cube draft system, managing
essence economy, and progressing through 7 battles. Dark fantasy visual theme
with animations. No backend; all state lives in memory and resets on page load.

## Background

Dreamtides is a roguelike deckbuilding game. A "quest" is a single run: the
player starts with 250 essence and no cards, navigates dreamscapes on a Dream
Atlas map, visits sites to draft cards and gain resources, and fights 7 battles
(battle 4 is a miniboss, battle 7 is the final boss). Victory requires winning
all 7 battles.

The card pool is ~485 cards defined in
`client/Assets/StreamingAssets/Tabula/rendered-cards.toml`. Each card has:
name, id (UUID), card-number (1-503), card-type (Character or Event),
subtype (24 types plus empty and wildcard), rarity (Common/Uncommon/Rare/
Legendary/Special), energy-cost (0-13), spark (0-10 for characters, empty
string for events), is-fast (bool), tide (one of Bloom/Arc/Ignite/Pact/Umbra/
Rime/Surge/Wild), tide-cost (1-3), rendered-text (with special symbols),
image-number (Shutterstock ID), art-owned (bool).

Special symbols in rendered-text: `●` (energy), `⍏` (spark), `▸` (trigger
prefix), `↯` (fast), `•` (bullet), `—` (em dash).

Card distribution: 333 Characters, 152 Events. 157 Common, 160 Uncommon,
162 Rare, 4 Legendary, 2 Special. Tide distribution is roughly even (56-65
cards per tide, plus 65 Wild).

Card art is cached as WebP files (462x280 or 390x280px, no file extension) at
`~/Library/Caches/io.github.dreamtides.tv/image_cache/`. The cache key for
each card is the SHA-256 hex digest of the URL string
`https://www.shutterstock.com/image-illustration/-260nw-{image_number}.jpg`
(UTF-8 bytes). There are 510 cached files. The Rust implementation at
`rules_engine/src/tv/src-tauri/src/images/image_cache.rs` (line 71-75) shows
the exact hashing: `Sha256::new()`, update with `url.as_bytes()`, format as
lowercase hex.

Tide icons are 7 PNG files (299x304 RGBA) at
`client/Assets/ThirdParty/GameAssets/Tides/`: Arc.png, Bloom.png, Ignite.png,
Pact.png, Rime.png, Surge.png, Umbra.png.

An existing Python draft simulator at `scripts/draft_simulator_v2/` provides
reference implementations for cube management, AI bot drafting, and pack
generation. The quest prototype must reimplement this logic in TypeScript.

The TV app (`rules_engine/src/tv/`) uses Vite 7 + React 19 + TypeScript 5.8
and can be referenced for project configuration patterns. Its `package.json`,
`tsconfig.json`, and `vite.config.ts` show the current conventions.

## Design

### Technology Stack

Vite + React + TypeScript, Tailwind CSS for styling, Framer Motion for
animations. Located at `scripts/quest_prototype/`. No Tauri, no backend -- this
is a pure client-side web app.

The project must use `"type": "module"` in package.json and target ES2020.
TypeScript should use strict mode with the same lint flags as the TV app's
tsconfig.json (strict, noImplicitAny, strictNullChecks, noImplicitReturns,
noUnusedLocals, noUnusedParameters).

### Visual Theme

Dark fantasy aesthetic throughout. Background colors should be deep
purple-black (#0a0612 range). Accent colors: purple (#7c3aed / #a855f7 range)
for primary actions, gold (#d4a017 / #fbbf24 range) for highlights and essence.
Card borders and UI chrome should use subtle gradients. Text is light
(off-white, #e2e8f0 range).

Framer Motion provides enter/exit animations on all screens: cards fan in on
draft, nodes pulse on the atlas, essence particles animate to the counter.
Transitions between screens use crossfade or slide animations (300-500ms).

### Image Setup Script

A build-time Node.js script (run as part of the dev/build process) creates
symlinks from human-readable names to the cached WebP files. The script:

1. Parses `rendered-cards.toml` to extract each card's `image-number`.
2. Computes SHA-256 of the URL
   `https://www.shutterstock.com/image-illustration/-260nw-{image_number}.jpg`.
3. Creates a symlink (or copy) in `scripts/quest_prototype/public/cards/` from
   `{card_number}.webp` to the cache file at
   `~/Library/Caches/io.github.dreamtides.tv/image_cache/{hash}`.
4. Copies tide PNGs from `client/Assets/ThirdParty/GameAssets/Tides/` into
   `scripts/quest_prototype/public/tides/`.

This script should be runnable via `npm run setup-assets` and must be idempotent.
It should report how many card images were found vs. missing from cache. The
TOML parsing can use a lightweight npm package (e.g., `smol-toml`) or the
built-in Node.js TOML support if available -- the script runs at build time, not
in the browser.

At runtime, card images load from `/cards/{card_number}.webp` and tide icons
from `/tides/{Tide}.png`. Missing images should show a styled placeholder
with the card name.

### Data Layer

Parse `rendered-cards.toml` at build time and emit a JSON file that the app
imports. This avoids shipping a TOML parser to the browser. The build script
(can be the same asset setup script or a separate one) should read the TOML
and write `public/card-data.json` containing the array of card objects.

The app loads this JSON at startup and builds an in-memory card database keyed
by card-number. Each card record contains all fields from the TOML.

The 7 tides are: Bloom, Arc, Ignite, Pact, Umbra, Rime, Surge. Wild is a
neutral pseudo-tide. Each tide should have an associated color for UI theming:

| Tide   | Color concept           |
|--------|-------------------------|
| Bloom  | Green / emerald         |
| Arc    | Yellow / amber          |
| Ignite | Red / crimson           |
| Pact   | Pink / magenta          |
| Umbra  | Purple / deep violet    |
| Rime   | Blue / ice blue         |
| Surge  | Cyan / teal             |
| Wild   | Gray / silver           |

### Synthetic Data

The prototype needs synthetic data for game elements that have no card data
source. Each category needs 8-12 entries with hand-authored names, descriptions,
and mechanical effects. These live as static data files within the prototype
source.

**Dreamcallers** (8-12): Each has a name, tide, ability description, essence
bonus (50-150), and a tide crystal grant. Distribute roughly evenly across
tides. Example shape: "Vyra the Tidebinder" -- Surge, "Characters you
materialize cost 1 less energy", 100 essence bonus.

**Dreamsigns** (8-12): Each has a name, tide, and effect description. Effects
should span battle modifiers, quest map modifiers, and economy modifiers.
Example: "Sigil of Abundance" -- Bloom, "Gain 50 bonus essence after each
battle."

**Dream Journeys** (8-12): Each is a circular card with a name and description
of a dramatic random effect. Effects should be splashy and deck-altering.
Example: "The Dreamer's Gambit" -- "Remove 5 random cards from your deck,
then add 3 rare cards."

**Tempting Offers** (8-12): Each has a journey (benefit) and a cost. Benefits
are powerful; costs are meaningful. Example: Benefit = "Add 3 copies of the
strongest card in your deck", Cost = "Gain 2 bane cards."

### Core State

The application maintains a single `QuestState` object in React context. This
state resets on page load (no persistence). Key fields:

- **essence**: number (starts at 250)
- **deck**: array of card-numbers (the player's drafted cards)
- **dreamcaller**: selected dreamcaller or null
- **dreamsigns**: array of acquired dreamsigns (max 12)
- **tideCrystals**: record of tide to count of permanent crystals
- **completionLevel**: number of battles won (0-7)
- **atlas**: the Dream Atlas graph structure
- **currentDreamscape**: which dreamscape is active (or null if on atlas)
- **visitedSites**: set of site IDs visited in current dreamscape
- **draftState**: the persistent cube draft state (pool, bots, round/pick
  counters)

The current screen is derived from state: atlas view when no dreamscape is
active, site view when interacting with a site, etc.

### Dream Atlas

The Dream Atlas is a radial node graph with the Nexus at the center. Nodes
are circular, connected by animated dotted lines.

**Node states**: Completed (dimmed, checkmark overlay), Available (glowing
pulse animation, bright border), Unavailable (dark, low opacity). Only nodes
connected to the Nexus or to a Completed node are Available.

**Layout**: Use a force-directed or radial placement algorithm. The Nexus is
always centered. Initial dreamscapes connect directly to the Nexus. New nodes
are placed adjacent to their parent completed node, spreading outward. The
graph should be pannable and zoomable (or auto-fit to viewport).

**Generation**: At quest start, generate 2-3 dreamscapes connected to the
Nexus. Each time a dreamscape is completed, generate 2-4 new Unavailable nodes
connected to the newly Completed node and to other nearby nodes. Each node
stores its dreamscape definition (sites, biome).

**Interaction**: Hover/click a node to preview its biome and available site
icons. Click an Available node to enter that dreamscape (transition animation).
The atlas shows the player's essence, deck size, dreamcaller, and dreamsign
count as a persistent HUD.

**Site icons on nodes**: Each dreamscape preview shows 2-3 non-draft,
non-battle site icons. Use simple SVG icons or emoji for each site type:
Battle = sword, Draft = card stack, Dreamcaller = crown, Shop = storefront,
Specialty Shop = star-store, Dreamsign Offering = sparkle, Dreamsign Draft =
sparkles, Dream Journey = moon, Tempting Offer = scale, Purge = flame,
Essence = diamond, Transfiguration = flask, Duplication = copy,
Reward = treasure chest, Cleanse = snowflake.

### Dreamscape Generation

When a dreamscape becomes available, its sites are generated:

1. Add draft sites per completion level: 2 at levels 0-1, 1 at levels 2-3,
   0 at level 4+.
2. Add exactly 1 battle site (always last to visit).
3. Add the dreamcaller draft site to the first dreamscape only.
4. Draw 1-3 additional sites from a weighted pool. Earlier completion levels
   favor shops, essence, and dreamsign offerings. Later levels add
   transfiguration, purge, and duplication.
5. Each dreamscape has a randomly assigned biome. The biome determines which
   site type (if present) becomes "enhanced" with stronger effects.

Total sites per dreamscape: 3-6 (including battle and draft sites).

### Dreamscape Site Interaction

Within a dreamscape, sites appear as a vertical list or a grid of clickable
icons with labels. All non-battle sites must be visited before the battle
site unlocks. Visited sites show a checkmark and become non-interactive.
Clicking a site transitions to that site's full-screen UI.

After visiting all sites and completing the battle, the view transitions back
to the Dream Atlas with the completed dreamscape marked.

### Card Rendering

Cards are the core visual element. Each card component displays:

- Card art (from `/cards/{card_number}.webp`), filling the top portion
- Card name (styled text, with tide-colored accent)
- Energy cost (top-left badge with `●` symbol)
- Spark value (bottom badge with `⍏` symbol, characters only)
- Tide icon (small tide PNG, colored border)
- Tide cost (small number next to tide icon)
- Rarity indicator (border glow: white=Common, green=Uncommon, blue=Rare,
  purple=Legendary)
- Card type and subtype line
- Rules text (rendered-text with special symbol substitution)
- Fast indicator (`↯` badge if is-fast is true)

**Compact mode** (used in draft grids, deck browser): Shows art, name, cost,
and spark only. Aspect ratio ~2:3.

**Expanded mode** (hover or click to expand): Full card with all fields
including rules text. Should appear as an overlay/modal.

Special symbols in rules text should render as styled inline icons or colored
Unicode characters. The `●` symbol should show in gold, `⍏` in a star shape,
`▸` as a colored arrow prefix, `↯` as a lightning bolt.

### Cube Draft System

The draft system is the most complex subsystem. It must faithfully implement
the 10-person cube draft described in the quest design.

**Pool initialization**: At quest start, create a pool of all ~483 non-Special
cards (exclude rarity "Special"). Each card appears exactly once. Assign
unique instance IDs.

**Pack dealing**: Deal 10 packs of 15 cards each from the pool (without
replacement). Each pack goes to one of the 10 seats (seat 0 = player, seats
1-9 = AI bots).

**Pick and pass**: Each seat picks 1 card from their pack, then packs rotate
to the next seat (fixed direction, always left). After 10 picks, all packs
have been through all seats and 5 cards remain per pack -- these are discarded.

**Player's draft site experience**: Each Draft site provides 5 picks. The first
draft site in a dreamscape covers picks from packs with 15, 14, 13, 12, 11
cards. The second covers packs with 10, 9, 8, 7, 6 cards.

**Rounds**: After 10 player picks (completing one set of circulating packs),
deal 10 new packs. After 3 rounds (30 total picks), discard the remaining pool
and create a fresh pool from all cards.

**AI Bot Drafters**: 9 persistent bots. Each bot scores cards using a weighted
blend:

- Archetype alignment (60%): dot product of card's tide fitness with the bot's
  learned preference vector.
- Raw power (20%): approximated by rarity value (Common=0.0, Uncommon=0.33,
  Rare=0.67).
- Openness (20%): estimated availability based on what cards are being passed.

Bots commit to a tide after 5 picks. They pick randomly 20% of the time.
After each pick, bots update their preference vector using a learning rate.

The Python reference uses an 8-archetype fitness vector, but rendered-cards.toml
no longer has per-archetype boolean fields. The prototype should use the card's
`tide` field to derive fitness: a card with tide X has fitness 1.0 for tide X
and 0.0 for all others. Wild cards have equal low fitness (e.g., 0.15) across
all tides.

The draft state (pool, bot states, round counter, pick counter) persists
across dreamscapes throughout the entire quest.

**Draft site UI**: Cards available for the current pick are shown in a
responsive 5x3 grid (5 columns, 3 rows, filling left-to-right top-to-bottom).
Cards are in compact mode. Hovering a card shows an expanded overlay with full
details. Clicking a card selects it -- the card animates to a "drafted" area
and the remaining cards animate away as the next pick's cards animate in.
After all 5 picks, show a summary of drafted cards and transition back to
the dreamscape. Cards in the draft grid have an orange selection outline.

### Battle System

Battles are auto-resolved in the prototype (no actual card combat). Clicking
"Start Battle" shows a brief animation (e.g., a clash effect lasting 1-2
seconds) and then the Victory screen.

**Pre-battle screen**: Shows the enemy dreamcaller name and ability, enemy
dreamsign count, and a "Start Battle" button. The enemy dreamcaller is
synthetic data (use a random name and generic ability text per completion
level).

**Victory screen**: Displays "Victory!" with an animated entrance, then shows:

- Essence reward: 100 + (completionLevel * 50) essence
- Rare card draft: 4 rare cards weighted to match player's deck tides, pick 1.
  This draft pick is mandatory (cannot be skipped).

Battle 4 is labeled "Miniboss" and battle 7 is labeled "Final Boss" with
distinct visual treatment (e.g., red-tinted pre-battle screen for miniboss,
gold for final boss).

Winning battle 7 shows a "Quest Complete!" celebration screen with final
stats (total cards drafted, essence spent, dreamscapes visited).

### Shop System

**Regular Shop**: Display 6 items in a 2x3 grid. Items are drawn from:

- Cards (majority): Weighted toward the player's most-drafted tides. Price by
  rarity: Common 50, Uncommon 100, Rare 200, Legendary 400.
- Dreamsigns: Occasionally (1 in 6 chance per slot). Price 150.
- Tide crystals: Occasionally (1 in 6 chance per slot). Price 200.

One slot may be a "Reroll" option (cost: 50 essence, increases by 25 each
reroll). Rerolling regenerates all 6 items with a staggered scale animation.

Random discounts: 1-2 items per shop get a 30-90% discount, shown with
a crossed-out original price and a highlighted new price.

Each item has a purple "Buy" button showing the essence cost. Buying animates
the item toward the deck/dreamsign area. Items remain in place after purchase
(leaving gaps). A "Leave Shop" button closes the site.

**Enhanced shop**: The reroll option is free (0 cost for all rerolls).

**Specialty Shop**: Shows 4 rare cards (tide-weighted). Same purchase UI as
regular shop but with different visual framing. Enhanced version: player may
take all offered cards for free.

### Dreamcaller Draft

Shows 3 dreamcaller cards side-by-side. Each card displays: name, tide icon,
ability text, essence bonus amount. A "Select" button below each. Selecting
a dreamcaller animates it to the HUD area and grants the essence bonus and
1 tide crystal.

If the player has already drafted cards when visiting this site, the offered
dreamcallers are weighted toward the player's most-drafted tides.
Otherwise, random selection.

The dreamcaller draft appears only in the first dreamscape.

### Dreamsign Sites

**Dreamsign Offering**: Shows 1 dreamsign card with its name, tide, and effect.
Accept (purple button) or Reject (gray button). Accepting adds it to the
dreamsign collection (max 12; if at cap, must purge one first).

**Dreamsign Draft**: Shows 3 dreamsigns in a row with accept buttons below
each and a close/skip button. Pick 1 or skip.

Enhanced dreamsign offering becomes a draft (3 options). Enhanced dreamsign
draft shows 4 options instead of 3.

### Dream Journey

Shows 2 circular journey cards side-by-side with descriptions. A close
button allows skipping. Selecting a journey shows its effect description with
an animation, then applies the effect (e.g., adds/removes cards from deck,
grants essence, modifies dreamsigns). Enhanced version shows 3 options.

### Tempting Offer

Shows 2 (enhanced: 3) pairs of journey + cost cards. Each pair is a row:
journey card on the left, cost card on the right, accept button below the pair.
Close button to skip. Accepting applies both the benefit and the cost.

### Purge

Opens a deck browser showing all cards in the player's deck. Player selects
up to 3 cards (enhanced: 6) by clicking them (selected cards get a red outline).
A "Purge N cards" button at the bottom confirms removal. A close button
allows purging 0 cards.

### Essence Site

Grants 200-300 essence (random). Enhanced: doubled (400-600). No full-screen
transition -- the atlas node plays a particle-like animation and the essence
counter increments with a count-up animation.

### Transfiguration

Shows 3 random cards from the player's deck. Each shows a "transfigured"
version with one of these modifications applied (randomly chosen per card):

- Viridian: Energy cost halved (rounded nearest). Not for 0-cost cards.
- Golden: A number in the rules text +/- 1.
- Scarlet: Spark doubled (or set to 1 if 0). Characters only.
- Azure: "Draw a card" appended. Events only.
- Bronze: "Reclaim" appended. Events only.

The card name and modified text display in the transfiguration's color. Accept
button per card, close button to skip. In the prototype, transfiguration is
visual only -- the card in the deck is marked with a transfiguration badge
and the display text is modified. Enhanced: player picks any card from their
deck to transfigure.

### Duplication

Shows 3 random cards from the player's deck, each with a random copy count
(1-4). A "Duplicate xN" button below each. Selecting one adds N copies of
that card to the deck. Close button to skip. Enhanced: player picks any card
from their deck.

### Reward Site

Shows a fixed reward (a specific card, dreamsign, or set of items) with
accept/decline buttons. The reward is visible on the atlas node preview before
entering the dreamscape.

### Cleanse

Shows up to 3 randomly selected "bane" items from the player's deck or
dreamsigns. In the prototype, banes are cards or dreamsigns that were acquired
through tempting offers' cost effects. Accept removes them; decline keeps them.
If no banes exist, the site displays "Nothing to cleanse" and auto-completes.

### Deck Viewer

A persistent overlay accessible from the HUD (click the deck icon). Shows all
cards in the player's deck in a scrollable grid. Features:

- Filter by tide (toggle buttons for each tide)
- Sort by: energy cost, name, tide, card type, acquisition order
- Card count display
- Click a card to see expanded view

Also shows dreamcaller (if selected) and dreamsigns in a sidebar or tabs.

### HUD (Heads-Up Display)

Persistent across all screens. Contains:

- Essence counter (gold diamond icon + number, count-up/down animations)
- Deck size (card stack icon + number)
- Dreamcaller portrait (small square, or empty frame if not selected)
- Dreamsign count (sparkle icon + number)
- Completion level / battle counter ("Battle 3/7")
- Deck viewer button

The HUD anchors to the bottom of the screen. On narrow viewports, it collapses
to icons only with numbers.

### Screen Flow

1. **Quest Start**: Brief intro screen with "Begin Quest" button. Initializes
   state with 250 essence, empty deck, draft pool.
2. **Dream Atlas**: Navigate and select a dreamscape.
3. **Dreamscape View**: List of sites to visit. Visit all non-battle sites,
   then battle.
4. **Site Screens**: Each site type has its own full-screen UI (described
   above). On completion, return to dreamscape view.
5. **Battle**: Pre-battle screen, auto-resolve, victory screen with rewards.
6. **Back to Atlas**: Dreamscape marked complete, new nodes generated.
7. **Repeat** until 7 battles won.
8. **Quest Complete**: Final summary screen.

### Responsive Layout

The prototype should work on desktop (1280+ wide) and tablet-sized viewports
(768+). Mobile is not required but the layout should not break below 768px.
Use Tailwind responsive breakpoints.

Draft grids adapt: 5 columns on desktop, 3 on tablet. Shop grids: 3 columns
on desktop, 2 on tablet.

## Constraints

- No automated tests. Manual QA with visual inspection at each stage.
- No persistence. All state resets on page load.
- No backend or network requests. All data is local (card data JSON, images
  from public/).
- Card images depend on the local image cache existing at
  `~/Library/Caches/io.github.dreamtides.tv/image_cache/`. The asset setup
  script must handle missing images gracefully.
- The prototype lives entirely within `scripts/quest_prototype/` and must not
  modify any other files in the repository.
- Battles are auto-win; no card combat is implemented.
- Special-rarity cards (2 total) are excluded from the draft pool.

## Non-Goals

- Actual battle gameplay (card combat, turns, AI opponents).
- Sound effects or music.
- Mobile-first design (tablet and desktop only).
- Network multiplayer or any server communication.
- Saving/loading quest state across sessions.
- Production-quality balance tuning of economy numbers.
- 3D rendering or WebGL (all 2D/DOM-based).
- Animation of NPC characters (the prototype omits NPCs entirely; sites show
  their content directly without NPC framing).

## Open Questions

- Exact tide-to-color mapping may need tuning during implementation based on
  contrast and readability against the dark background. The table above is a
  starting point.
- The card fitness derivation from tide (rather than 8-archetype booleans) is a
  simplification. If bot drafting behavior feels degenerate during playtesting,
  the fitness model may need enrichment (e.g., subtype-based secondary fitness).

## References

- Quest design document: `docs/plans/quests/quests.md`
- Card data: `client/Assets/StreamingAssets/Tabula/rendered-cards.toml`
- Image cache implementation: `rules_engine/src/tv/src-tauri/src/images/image_cache.rs`
- Tide icons: `client/Assets/ThirdParty/GameAssets/Tides/*.png`
- Draft simulator (Python reference): `scripts/draft_simulator_v2/`
  - Draft runner: `scripts/draft_simulator_v2/draft_runner.py`
  - Cube manager: `scripts/draft_simulator_v2/cube_manager.py`
  - AI agents: `scripts/draft_simulator_v2/agents.py`
  - Card loading: `scripts/draft_simulator_v2/card_generator.py`
  - Config/defaults: `scripts/draft_simulator_v2/config.py`
  - Data models: `scripts/draft_simulator_v2/draft_models.py`
- TV app (Vite/React/TS reference): `rules_engine/src/tv/`
  - package.json: `rules_engine/src/tv/package.json`
  - tsconfig.json: `rules_engine/src/tv/tsconfig.json`
  - vite.config.ts: `rules_engine/src/tv/vite.config.ts`
