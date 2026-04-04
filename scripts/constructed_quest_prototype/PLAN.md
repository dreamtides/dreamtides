# Constructed Quest Prototype

Collection-based quest mode prototype for Dreamtides. Players select starting
tides, receive a starter card pool, acquire cards through loot packs, shops,
forges, and drafts, then construct decks from their pool to fight 7 battles.

Full design: `notes/pack_quest_design.md`

## Architecture

React 19 + Vite 7 + TypeScript 5.8, Tailwind CSS for styling, Framer Motion
for animations. Pure client-side (no backend). State managed via React Context
(`quest-context.tsx`) with a mutations API. Screen routing via discriminated
union (`Screen` type) rendered by `ScreenRouter.tsx`.

Key directories:

- `src/types/` -- `quest.ts` (state types), `cards.ts` (card data types)
- `src/state/` -- `quest-context.tsx` (state + mutations), `quest-config.ts`
  (URL params)
- `src/screens/` -- one component per screen
- `src/components/` -- shared UI (HUD, DeckEditor, DeckViewer, CardDisplay,
  CardOverlay, ScreenRouter)
- `src/data/` -- card database, synthetic data (dreamcallers, dreamsigns,
  dream journeys, tempting offers, biomes), tide weights
- `src/atlas/` -- atlas graph generation and dreamscape site composition
- `src/pack/` -- loot pack generation with duplicate protection
- `src/shop/` -- card shop and pack shop generation
- `src/forge/` -- forge recipe generation (sacrifice cards of one tide for
  a card of another)
- `src/provisioner/` -- provisioner site catalog (buy extra sites for essence)
- `src/ante/` -- ante wager logic and escalation
- `src/transfiguration/` -- card modification logic

## Quest Flow

1. **Quest Start** -- 3 random tides selected (configurable). Starter deck of
   ~10 cards built from those tides. Player receives starting essence (250).
2. **First Dreamscape** -- Auto-entered immediately (no atlas). Contains loot
   packs, a dreamcaller draft, and the first battle. All non-battle sites must
   be visited before the battle unlocks.
3. **Atlas** -- After the first battle, the Dream Atlas appears. Radial node
   graph with a central Nexus. Completed nodes unlock adjacent ones. Each node
   shows site icons and biome info.
4. **Dreamscapes 2-7** -- Each contains a battle plus a composition of sites
   determined by completion level. Earlier levels have more loot packs; later
   levels add forge, provisioner, transfiguration, and draft sites.
5. **Quest Complete** -- Win all 7 battles. Final summary screen with stats.

Battles are auto-resolved (no card combat). Battle 4 is a miniboss, battle 7
is the final boss.

## Card Acquisition

Cards are acquired into a **pool**. The player constructs a **deck** from the
pool using the Deck Editor. Acquisition sources:

- **Loot Pack** -- Tide-themed pack of cards (default 4). Duplicate protection
  via weighted sampling. Enhanced packs are double size.
- **Card Shop** -- Buy individual cards for essence. Prices vary by rarity.
  Reroll available (cost increases each use).
- **Pack Shop** -- Buy sealed packs (tide packs, alliance packs, removal packs,
  aggro packs, event packs). Special packs appear randomly.
- **Forge** -- Sacrifice N cards of one tide to receive a card of a different
  tide. Recipes generated based on pool composition.
- **Provisioner** -- Pay essence to add a new site (Forge, Transfiguration,
  Duplication, Draft, etc.) to the current dreamscape.
- **Draft Site** -- See N cards, keep a subset (default: see 4, keep 1).
- **Ante** -- Wager pool cards before battle. Win to claim opponent's ante
  cards; lose to forfeit yours.

Other non-acquisition sites: Essence, Dreamsign Offering/Draft, Dream Journey,
Tempting Offer, Transfiguration, Duplication, Reward, Cleanse, Dreamcaller
Draft.

## Deck Editor

Full-screen overlay accessible from the HUD. Two-panel layout:

- **Pool panel** (left) -- All cards not in the active deck. Filterable by
  tide and card type. Sortable by energy cost, name, tide, or card type. Click
  a card to move it to the deck.
- **Deck panel** (right) -- Compact grouped list showing card name, cost, and
  copy count. Click to remove from deck.

Constraints enforced by configuration:

- Minimum deck size (default 25)
- Maximum deck size (default 50)
- Maximum copies of any single card (default 2)
- Bane cards are locked in the deck and cannot be removed

Bulk actions: "Add All" and "Remove All" buttons.

## Ante System

Optional pre-battle wagering system (enabled by default).

1. Before battle, player selects cards from their pool to wager (up to
   `maxAnteCards`, default 2).
2. Opponent antes cards weighted toward the player's dominant tides and higher
   rarity.
3. **Escalation** -- At turn N (default 6), either side can raise the stakes
   by adding more cards. AI has a 50% bluff-escalation chance.
4. **Concession** -- Player may concede to forfeit only their first ante card
   instead of all.
5. On win, player gains opponent's ante cards. On loss (concession), player
   loses their ante cards.

## Configuration

All tuning knobs are exposed as URL search parameters (38 total). Parsed by
`quest-config.ts` with defaults, min/max bounds, and type coercion.

Key parameters and defaults:

| Category | Parameters |
|----------|-----------|
| Tides | `revisedTides` (true), `startingTides` (3), `sequentialTides` (true) |
| Starter | `initialCards` (10), `starterNeutral` (5), `starterLowCost` (4), `starterMidCost` (3), `starterHighCost` (1) |
| Economy | `startingEssence` (250), `battleEssence` (150), `essencePerLevel` (50), `essenceSiteAmount` (200) |
| Packs | `lootPackSize` (4), `dupePenalty2` (50), `dupePenalty3` (90), `packOnTheme` (60), `packAdjacent` (25), `packExplore` (15) |
| Deck | `minimumDeckSize` (25), `maximumDeckSize` (50), `maxCopies` (2) |
| Card Shop | `cardShopSize` (4), `cardPriceMin` (50), `cardPriceMax` (100), `rerollBase` (40), `rerollIncrement` (20) |
| Pack Shop | `packShopSize` (3), `specialPackChance` (20) |
| Ante | `anteEnabled` (true), `escalationTurn` (6), `maxAnteCards` (2) |
| Forge | `forgeRecipes` (3), `forgeCost` (4) |
| Draft | `draftSiteTotal` (4), `draftSiteKeep` (1) |
| Other | `provisionerOptions` (3), `dreamcallerChoices` (3), `opponentPreviewCards` (3), `showTideSymbols` (true) |

## Running

From `scripts/constructed_quest_prototype/`:

```
npm install
npm run dev          # Dev server (runs setup-assets first)
npm run test         # Run tests via vitest
npm run typecheck    # TypeScript type checking
npm run build        # Production build
npm run setup-assets # Symlink card images from local cache
```

## Constraints

- No persistence -- state resets on page load.
- No backend or network requests.
- Card images require the local image cache at
  `~/Library/Caches/io.github.dreamtides.tv/image_cache/`.
- Battles are auto-win; no card combat implemented.
- Special-rarity cards excluded from all card pools.
