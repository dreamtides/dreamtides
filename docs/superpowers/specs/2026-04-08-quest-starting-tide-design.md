# Quest Starting Tide Redesign - Design Spec

Redesign the quest prototype start so each run begins from a visible tide
anchor. The player chooses one of 3 random named tides, receives a playable
30-card starting deck, and then sees drafts, dreamcallers, shops, dreamsigns,
and rewards that begin near that tide but can pivot toward the deck they build.

This spec applies to the React quest prototype in `scripts/quest_prototype/`.

## Goals

- Make quest start an explicit gameplay choice instead of "Begin Quest".
- Start each run with a 30-card deck that can battle immediately.
- Treat `Starter` cards as loadout-only basics, never random offer cards.
- Preserve the revised tide philosophy: a tide is an anchor, not a complete
  mono-tide lane.
- Make later generated offers use one shared tide-profile model instead of
  ad-hoc deck-count weighting.
- Log enough structured data to audit tide choice, starting deck, offer bias,
  and deck evolution from downloaded quest logs.

## Non-Goals

- Do not implement the production 10-person cube draft from `quests.md`.
- Do not write final dreamcaller abilities or final dreamcaller roster content.
- Do not redesign quest map site counts, battle simulation, economy numbers, or
  card rendering.
- Do not include `Starter` cards in random generation to make tests pass.

## Quest Start Flow

`QuestStartScreen` becomes a starting tide selection screen.

On first render, generate 3 distinct options from the 7 named tides:
Bloom, Arc, Ignite, Pact, Umbra, Rime, Surge. Neutral is never offered.

Selecting a tide starts the quest:

1. Store `startingTide` on quest state.
2. Grant 1 permanent tide crystal of `startingTide`.
3. Build the starting deck.
4. Record the random starting grant card numbers as consumed cards.
5. Generate the initial atlas.
6. Transition to the atlas.

The normal start flow uses no default excluded tides. Keep URL/debug support for
excluded tides, but the default quest should have `excludedTideCount = 0`.

If a debug excluded-tide config is active, the starting-tide offer must not
include excluded tides.

## Starting Deck

The starting deck contains exactly 30 cards:

1. The 10 cards with `rarity = "Starter"` from
   `client/Assets/StreamingAssets/Tabula/rendered-cards.toml`.
2. 10 random cards with `tide = startingTide`, excluding `Starter` and
   `Special`.
3. 10 random Neutral cards, excluding `Starter`, `Special`, and `Legendary`.

The 20 random tide/neutral grants are consumed from the finite quest draft pool.
The 10 fixed Starter cards are not consumed because they are never in that pool.

The prototype asset pipeline must load Starter cards into `card-data.json` so
the starter package can be found and rendered. `Starter` becomes a valid
TypeScript `Rarity`.

## Offerable Card Pool

Use a common eligibility helper for generated cards.

By default, random card offers exclude:

- `Special`
- `Starter`
- debug-excluded tides
- card numbers already consumed by starting random grants, for finite draft pool
  only

Neutral starting grants additionally exclude `Legendary`.

Named-tide starting grants may include Legendary cards. That is intentionally
different from the Neutral grant rule.

## Starting Tide Visibility

The selected starting tide must be visible after quest start.

Minimum UI:

- HUD shows a compact starting-tide/origin badge and the granted crystal is
  reflected in the existing crystal count.
- Deck viewer sidebar has a "Quest Origin" or "Starting Tide" section.
- Deck viewer distribution includes the full deck, including starters, but
  weighting logic does not treat 10 starters as a strong Neutral commitment.

## Dreamcallers

Dreamcallers change from one tide to exactly two named tides. Neutral
dreamcallers are removed from the normal offer pool for this prototype pass.

Example model:

```ts
interface Dreamcaller {
  name: string;
  tides: [NamedTide, NamedTide];
  abilityDescription: string;
  essenceBonus: number;
  tideCrystalGrant: NamedTide;
}
```

The Dreamcaller Draft keeps 3 offer slots.

For the first dreamcaller draft:

1. Offer one dreamcaller containing the starting tide and its left neighbor if
   available.
2. Offer one dreamcaller containing the starting tide and its right neighbor if
   available.
3. Fill the third slot from the adaptive quest tide profile.

Fallback order when a desired pair has no content:

1. Any dreamcaller whose pair contains the starting tide.
2. Any dreamcaller whose pair contains a neighbor of the starting tide.
3. Any named-tide dreamcaller sampled from the adaptive profile.

If the player visits drafts, shops, or other deck-changing sites before the
dreamcaller draft, the third slot and all fallback samples use the latest quest
tide profile.

The Dreamcaller Draft UI displays both tide icons. Selection logs both tides.

## Quest Tide Profile

Add a shared profile function for offer weighting. It returns numeric weights
for the 7 named tides plus a low Neutral baseline.

Inputs:

- starting tide, as an early-run prior
- the two named neighbors of the starting tide
- current deck, excluding Starter cards and discounting Neutral cards heavily
- currently selected dreamcaller's two tides
- permanent tide crystals
- recent draft picks, newest first

Intended behavior:

- Before any deck changes, starting tide has the strongest named-tide weight and
  its two neighbors have meaningful secondary weight.
- After dreamcaller selection, both dreamcaller tides become major weights.
- After many acquired non-neutral cards, the actual deck distribution can beat
  the starting-tide prior.
- Neutral provides a baseline/glue option, not a dominant profile because the
  starter package is Neutral.

Log profile summaries at major generation points. Every profile log includes
the ordered total tide weights and the separate start/deck/dreamcaller/crystal
contribution maps.

## Draft Sites

Initialize the draft state from the offerable finite pool:

- no `Special`
- no `Starter`
- no debug-excluded tides
- no consumed starting random grants

The pack generator uses the quest tide profile as its initial affinity. Keep the
current Tide Current behavior so recent draft picks increase focus and allow
pivots during the run.

Draft pick logs should continue to include picked card name/tide and should add
enough generation context to tell whether the pack was profile-biased.

## Shops, Dreamsigns, and Rewards

Regular shops and specialty shops sample card items from the shared profile
instead of raw deck-count weighting. They never offer `Starter` or `Special`
cards.

Shop tide crystals prefer profile tides, but can occasionally offer a named
tide with low or zero existing crystal count.

Dreamsign offers, dreamsign drafts, dreamsign shop slots, and dreamsign reward
sites sample dreamsigns using the profile.

Battle rare rewards use profile weighting and remain rare-card rewards.

Reward sites roll their concrete reward when the player enters the site instead
of pre-rolling card/dreamsign rewards during atlas generation. This gives reward
sites access to the freshest deck, dreamcaller, crystal, and starting-tide
signals. Essence-only reward data may remain static.

## Logging

Add structured events for the new flow:

- `starting_tide_options_generated`: the 3 offered named tides
- `starting_tide_selected`: selected tide, granted crystal
- `starting_deck_initialized`: starter card numbers, starting-tide card
  numbers, neutral card numbers, total deck size
- `quest_tide_profile_computed`: generation context, ordered tide weights
- `dreamcaller_offers_generated`: offered names and tide pairs
- `shop_inventory_generated`: slot item types and card/dreamsign/crystal tides
- `reward_generated`: reward type and tide/card information when relevant

Update existing events whose shape changes, especially `quest_started`,
`dreamcaller_selected`, and `draft_pool_initialized`.

## Automated Verification

Add or update TypeScript tests for:

- `Starter` rarity transforms from TOML into `card-data.json`.
- Starting tide option generation returns 3 distinct named tides.
- Starting deck generation returns exactly 10 Starter, 10 selected-tide, and 10
  eligible Neutral entries.
- Starter/Special cards are excluded from offerable random pools.
- Neutral Legendary cards are excluded from Neutral starting grants.
- Random starting grants are removed from draft pool initialization.
- Default config has zero excluded tides.
- Dreamcaller offer generation attempts the left-pair and right-pair
  starting-tide forks.
- Quest tide profile has the expected dominant tides before deck changes and can
  pivot after non-neutral deck additions.
- Shop, specialty shop, rare reward, and dreamsign sampling never emit Starter
  or Special cards.

Run the quest prototype tests, typecheck, lint, and build before browser QA.

For this work, use web/prototype review gates only. The repo-level `just review`
gate currently includes broken Unity work and should not block implementation
of this prototype redesign.

## Manual QA

Use `agent-browser` for adversarial manual QA after implementation.

Required scenarios:

1. Start 3 fresh quests and screenshot the 3 starting tide options each time.
2. Select at least 2 different starting tides across QA runs.
3. Immediately open deck viewer. Verify deck size is 30, origin is visible,
   crystal is visible, and the deck contains the expected starter/tide/neutral
   groups.
4. Enter Draft, record visible pack tides for all 5 picks, choose cards, and
   verify deck size increases by exactly 5.
5. Enter Dreamcaller Draft. Verify both-tide icon display and starting-tide
   fork behavior where content exists.
6. Enter Shop or Specialty Shop. Verify item purchases mutate essence/deck by
   the displayed amounts and no Starter card is offered.
7. Enter a Reward site. Verify card/dreamsign reward is generated at entry and
   reflects current profile.
8. Win a battle and choose a rare reward. Verify rare reward is not Starter or
   Special and is logged.
9. Download the quest log and run a log-analysis script that reconstructs:
   starting options, selected tide, starting deck groups, draft pool size,
   dreamcaller offer tides, shop offer tides, reward offer tides, final deck
   size, and card additions/removals in order.
10. Reset/reload and verify no previous quest origin, deck, draft pool, or log
    state leaks into the next run.

Any of the following is a feature bug:

- Normal quest start silently excludes named tides.
- A Starter card appears in draft, shop, specialty shop, rare reward, reward
  site, or random starting tide/neutral package.
- Neutral Legendary appears in the Neutral starting package.
- A random starting grant appears in the finite draft pool.
- Starting tide, starting crystal, dreamcaller pair, or generated offer tides
  are not visible or not logged.
- Initial deck count is not exactly 30.

## Acceptance

- The prototype has a visible starting-tide choice out of 3 named-tide options.
- Selecting a starting tide starts a quest with 30 cards and 1 matching crystal.
- Starter cards are loadout-only and visible in the starting deck.
- Drafts, shops, dreamsigns, dreamcallers, reward sites, and battle rare rewards
  use one shared quest tide profile.
- The run can begin near the chosen tide, fork through a two-tide dreamcaller,
  and pivot through later deck choices.
- Automated tests pass.
- Browser QA is documented with screenshots and a downloaded log.
- Log analysis confirms the generated run matches this spec.
