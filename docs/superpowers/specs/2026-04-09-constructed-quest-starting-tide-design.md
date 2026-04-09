# Constructed Quest Starting Tide Redesign - Design Spec

Redesign `scripts/constructed_quest_prototype` so each quest run begins with
an explicit tide choice. The player picks one of 3 random named tides, receives
a 30-card starting deck anchored to that tide, and then sees dreamcallers,
packs, shops, and rewards that bias toward the chosen tide and its neighbors.

This spec applies to `scripts/constructed_quest_prototype/`. It adapts the
design intent from the `quest_prototype` starting tide spec
(`docs/superpowers/specs/2026-04-08-quest-starting-tide-design.md`) to the
constructed prototype's different architecture — no finite draft pool, no
consumed-card tracking, simpler weighting infrastructure.

## Goals

- Make quest start an explicit gameplay choice instead of "Begin Quest".
- Start each run with a 30-card deck that can battle immediately.
- Treat `Starter` cards as loadout-only basics, never offered in packs, shops,
  drafts, or rewards.
- Preserve the revised tide philosophy: a tide is an anchor, not a complete
  mono-tide lane.
- Convert dreamcallers to two-tide pairs for archetype-defining early choices.
- Mix occasional neutral cards into tide packs for deck variety.
- Validate via extensive manual QA with `agent-browser`.

## Non-Goals

- Do not create a finite draft pool or consumed-card tracking system.
- Do not create a full quest-tide-profile module — keep the existing
  `tideWeight()` / `countDeckTides()` / `weightedSample()` infrastructure.
- Do not redesign quest map site counts, battle simulation, economy numbers, or
  card rendering.
- Do not write final dreamcaller abilities or roster content.

## Quest Start Flow

`QuestStartScreen` becomes a starting tide selection screen.

On first render, generate 3 distinct options from the 7 named tides:
Bloom, Arc, Ignite, Pact, Umbra, Rime, Surge. Neutral is never offered.

Selecting a tide starts the quest:

1. Store `startingTide: NamedTide` on QuestState, replacing the current
   `startingTides: Tide[]` array.
2. Grant 1 permanent tide crystal of `startingTide`.
3. Build the starting deck (see below).
4. Generate the initial atlas.
5. Transition to the atlas screen.

## Starting Deck

The starting deck contains exactly 30 cards:

1. The 10 cards with `rarity = "Starter"` from `rendered-cards.toml`. These are
   all Neutral tide with basic effects.
2. 10 random cards with `tide = startingTide`, excluding `Starter`, `Special`,
   and `Legendary` rarity.
3. 10 random Neutral cards, excluding `Starter`, `Special`, and `Legendary`
   rarity.

No cards are consumed from any pool. All sampling is infinite (with
replacement across quests, without replacement within a single starting deck
generation).

## Rarity Type Change

Add `Starter` to the `Rarity` type in `types/cards.ts`. Add a display color
for Starter in `RARITY_COLORS`. The existing `setup-assets.mjs` already
includes Starter cards in `card-data.json` (it only filters `Special`), so no
asset pipeline changes are needed.

## Starter Card Exclusion

All card sampling throughout the quest excludes `rarity === "Starter"`:

- Loot packs
- Card shop inventory
- Pack shop packs (tide, alliance, removal, aggro, events)
- Draft sites
- Forge recipes
- Rare battle rewards
- Reward sites
- Any other system that generates random card offers

Starter cards exist only in the fixed starting loadout.

## Neutral Cards in Tide Packs

Currently tide packs and loot packs filter strictly to `c.tide === packTide`.

Change this so there is approximately a 50% chance to include one Neutral card
in a tide pack. When triggered, one of the tide-specific slots is replaced with
a random non-Legendary, non-Starter Neutral card.

Result: a 4-card tide pack contains 3-4 tide cards + 0-1 neutral cards (total
pack size unchanged). For enhanced loot packs (8 cards), allow 0-2 neutrals
using the same per-slot probability.

Apply this to both `generateLootPack` and `generateTidePackCards` in the pack
shop.

## Dreamcallers

### Type Change

Convert dreamcallers from single-tide to two-tide pairs:

```ts
interface Dreamcaller {
  name: string;
  tides: [NamedTide, NamedTide];
  abilityDescription: string;
  essenceBonus: number;
  tideCrystalGrant: NamedTide;
}
```

Remove the Neutral dreamcaller. All 10 dreamcallers use named-tide neighbor
pairs from the tide circle. Cover all 7 neighbor pairs at least once:

```
Bloom/Arc, Arc/Ignite, Ignite/Pact, Pact/Umbra,
Umbra/Rime, Rime/Surge, Surge/Bloom
```

With 10 callers and 7 pairs, 3 pairs get a second caller.

`tideCrystalGrant` must be one of the caller's two tides.

### Dreamcaller Draft Offer Logic

The dreamcaller draft offers 3 choices:

1. **Left fork:** A dreamcaller whose pair is `[startingTide, leftNeighbor]`.
2. **Right fork:** A dreamcaller whose pair is `[startingTide, rightNeighbor]`.
3. **Adaptive:** Weighted sample from remaining callers based on current deck
   composition using existing `tideWeight()`.

This gives the player an archetype-defining fork early in the run. For example,
starting Bloom offers Bloom/Surge (storm direction) vs Bloom/Arc (ramp
direction), plus a wildcard.

**Fallback order** when a desired pair has no matching caller:

1. Any caller whose pair contains the starting tide.
2. Any caller whose pair contains a neighbor of the starting tide.
3. Any named-tide caller weighted by deck tides.

No duplicate callers across the 3 slots.

### UI

Display both tide icons on each dreamcaller card. Selection logs both tides.

## Downstream Tide Weighting

Keep the existing `tideWeight()`, `countDeckTides()`, `weightedSample()`, and
`selectPackTide()` infrastructure. Adapt it to work with the single starting
tide:

- Where code currently seeds from `startingTides: Tide[]` (e.g., shop
  generators, atlas loot pack tide selection), seed from `startingTide` + its
  two circle neighbors instead. This maintains the early-game bias toward the
  starting tide's archetype space.
- `selectPackTide()` continues to derive dominant tides from pool composition.
  Early on the pool is starting-tide + neutral cards, so it naturally biases
  toward the starting tide and adjacent tides.
- All other weighting logic (duplicate penalties, on-theme/adjacent/explore
  distribution, pack shop special packs) remains unchanged.

## QuestState Changes

Replace:
```ts
startingTides: Tide[];
```

With:
```ts
startingTide: NamedTide | null;
```

All code that currently reads `state.startingTides` must be updated to derive
the equivalent from `state.startingTide` and `adjacentTides()`.

## Deck Viewer

Add a "Quest Origin: [Tide]" display in the deck viewer when `startingTide` is
set.

No other UI changes. The HUD already displays tide crystals, and the starting
crystal grant makes the chosen tide visible there.

## Automated Verification

Keep existing tests passing with minimal changes:

- Update `Rarity` type and `RARITY_COLORS` for Starter.
- Update `synthetic-data.test.ts` for two-tide dreamcaller shape.
- Update any tests that reference `startingTides` array to use `startingTide`.
- Run typecheck, lint, and build.

## Manual QA

Use `agent-browser` for extensive manual QA. Required scenarios:

1. Start quest — verify 3 named tide options displayed, no Neutral.
2. Select a tide — verify deck is 30 cards, tide crystal granted.
3. Open deck viewer — verify origin displayed, deck contains expected
   starter/tide/neutral groups.
4. Visit dreamcaller draft — verify two-tide icons, left-fork/right-fork
   behavior for the chosen starting tide.
5. Visit loot pack — verify no Starter cards, tide packs occasionally include
   a neutral card.
6. Visit card shop — verify no Starter cards offered.
7. Visit pack shop — verify no Starter cards in packs.
8. Win a battle — verify rare rewards exclude Starter.
9. Reload — verify no state leaks between runs.

Any of the following is a bug:

- A Starter card appears in any random offer (draft, shop, pack, reward, forge,
  rare reward).
- Neutral or no tide options appear in the starting choice.
- Starting deck is not exactly 30 cards.
- Legendary cards appear in the starting tide or neutral starting packages.
- Dreamcaller shows only one tide or offers a Neutral caller.
- Starting tide is not visible in deck viewer.

## Acceptance

- The prototype has a visible starting-tide choice out of 3 named-tide options.
- Selecting a starting tide starts a quest with 30 cards and 1 matching crystal.
- Starter cards are loadout-only and never appear in random offers.
- Dreamcallers have two tides and the draft offers archetype forks.
- Tide packs occasionally include a neutral card.
- All downstream systems (shops, packs, rewards, forge, draft) exclude Starter
  cards and bias toward the starting tide.
- Automated tests pass. Browser QA documents the above scenarios.
