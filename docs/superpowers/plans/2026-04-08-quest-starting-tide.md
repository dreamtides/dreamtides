# Quest Starting Tide Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redesign `scripts/quest_prototype` so a quest starts by choosing a named tide, builds a 30-card starting deck, and routes later offers through a shared adaptive tide profile.

**Architecture:** Add small pure TypeScript generation modules for card-pool eligibility, quest-start deck construction, quest tide profiling, and dreamcaller offers. Extend quest context with `startingTide` and `consumedStartingCardNumbers`, then wire existing draft/shop/reward/dreamcaller screens through those pure helpers. Keep UI changes local to quest start, HUD, deck viewer, and dreamcaller cards.

**Tech Stack:** React 19, Vite 7, TypeScript 5, Vitest, ESLint, agent-browser manual QA, downloaded JSONL quest logs

**Design spec:** `docs/superpowers/specs/2026-04-08-quest-starting-tide-design.md`

---

## Execution Rules

For this prototype work, run web/prototype gates only. Do not use `just review`;
the repo-level Unity review gate is known broken for this work.

Run commands from `scripts/quest_prototype` unless a step explicitly says to run
from repo root.

Primary verification commands:

```bash
npm run setup-assets
npm test
npm run typecheck
npm run lint
npm run build
```

Manual QA tool:

```bash
/Users/dthurn/Library/pnpm/agent-browser
```

## QA Requirements For Worker Prompts

Copy this block into implementation-worker prompts for UI-affecting tasks:

> Use the `qa` skill for browser verification and use `agent-browser` to take screenshots. Before each click, state the expected deck size, screen, visible origin tide, essence, crystals, and selected-card effect. After each click, take a screenshot and compare actual visible values plus any JS-evaluated state you use. Keep screenshots under `/tmp/quest-starting-tide-qa/`. Do not claim completion if deck size, HUD origin, crystal count, dreamcaller pair icons, shop item state, reward state, or logs differ from the expected behavior.

Copy this block into implementation-worker prompts for logging / run-analysis tasks:

> Use the `qs-analyze` skill after downloading a quest log. Analyze JSONL events; do not infer missing data. If the log cannot answer starting options, selected tide, starter/tide/neutral card groups, dreamcaller offer pairs, draft pool exclusions, shop offer tides, reward tides, or final deck size, add explicit logging and retest.

## File Map

**Create:**

| File | Responsibility |
|------|----------------|
| `scripts/quest_prototype/src/data/card-pools.ts` | Pure card eligibility, sampling, starter lookup, consumed-card filtering |
| `scripts/quest_prototype/src/data/card-pools.test.ts` | Tests for Starter/Special/Neutral/consumed-card rules |
| `scripts/quest_prototype/src/quest-start/quest-start-generator.ts` | Starting tide options and 30-card starting deck generation |
| `scripts/quest_prototype/src/quest-start/quest-start-generator.test.ts` | Quest-start generation tests |
| `scripts/quest_prototype/src/data/tide-circle.ts` | Named-tide type, circle neighbors, pair helpers |
| `scripts/quest_prototype/src/data/tide-circle.test.ts` | Neighbor and pair-key tests |
| `scripts/quest_prototype/src/data/quest-tide-profile.ts` | Shared adaptive tide weighting profile |
| `scripts/quest_prototype/src/data/quest-tide-profile.test.ts` | Profile dominance, pivot, logging tests |
| `scripts/quest_prototype/src/data/dreamcaller-offers.ts` | Three-slot dreamcaller offer selection |
| `scripts/quest_prototype/src/data/dreamcaller-offers.test.ts` | Starting-tide pair and fallback tests |
| `scripts/quest_prototype/src/qa/analyze-quest-log.mjs` | Local JSONL invariant checker for manual QA logs |

**Modify:**

| File | Responsibility |
|------|----------------|
| `scripts/quest_prototype/scripts/setup-assets.mjs` | Keep Starter cards in generated `card-data.json`; continue excluding Special |
| `scripts/quest_prototype/src/types/cards.ts` | Add `Starter` rarity; add `NamedTide` type alias |
| `scripts/quest_prototype/src/types/quest.ts` | Store `startingTide`, consumed starting cards, and two-tide Dreamcaller |
| `scripts/quest_prototype/src/types/draft.ts` | Allow profile seed on draft pack strategy or context |
| `scripts/quest_prototype/src/state/quest-config.ts` | Change default excluded tide count to 0 |
| `scripts/quest_prototype/src/state/quest-context.tsx` | Add quest-start mutation; reset/log new state; adapt dreamcaller logging |
| `scripts/quest_prototype/src/screens/QuestStartScreen.tsx` | Replace begin button with 3 starting-tide choice cards |
| `scripts/quest_prototype/src/draft/draft-engine.ts` | Exclude consumed/Starter cards; seed affinity from profile |
| `scripts/quest_prototype/src/screens/DraftSiteScreen.tsx` | Pass consumed cards and profile into draft initialization / pack entry |
| `scripts/quest_prototype/src/data/dreamcallers.ts` | Convert synthetic callers to two named tides |
| `scripts/quest_prototype/src/screens/DreamcallerDraftScreen.tsx` | Use dreamcaller offer helper; display two tide icons |
| `scripts/quest_prototype/src/data/tide-weights.ts` | Replace rare rewards with profile-aware rare reward helper or delegate |
| `scripts/quest_prototype/src/screens/BattleScreen.tsx` | Use profile-aware enemy / rare reward path |
| `scripts/quest_prototype/src/shop/shop-generator.ts` | Use profile-aware card, dreamsign, crystal selection; exclude Starter/Special |
| `scripts/quest_prototype/src/screens/ShopScreen.tsx` | Pass quest tide profile into shop inventory generation |
| `scripts/quest_prototype/src/screens/SpecialtyShopScreen.tsx` | Pass quest tide profile into specialty shop generation |
| `scripts/quest_prototype/src/atlas/atlas-generator.ts` | Stop pre-rolling card/dreamsign reward data for Reward sites |
| `scripts/quest_prototype/src/screens/RewardSiteScreen.tsx` | Roll card/dreamsign reward at site entry from current profile |
| `scripts/quest_prototype/src/components/HUD.tsx` | Show compact starting tide/origin badge |
| `scripts/quest_prototype/src/components/DeckViewer.tsx` | Show Quest Origin and two-tide dreamcaller metadata |
| `scripts/quest_prototype/src/components/CardDisplay.tsx` | Ensure Starter rarity renders with a stable color |
| `scripts/quest_prototype/src/data/card-database.ts` | Add Starter rarity color |
| Existing tests beside each modified module | Update expectations for Starter and two-tide dreamcallers |

---

## Task 1: Asset Data And Card Types

**Files:**
- Modify: `scripts/quest_prototype/scripts/setup-assets.mjs`
- Modify: `scripts/quest_prototype/src/types/cards.ts`
- Modify: `scripts/quest_prototype/src/data/card-database.ts`
- Modify: `scripts/quest_prototype/src/data/card-database.test.ts`
- Modify: `scripts/quest_prototype/src/data/synthetic-data.test.ts`
- Modify: `scripts/quest_prototype/src/components/CardDisplay.tsx`

- [ ] **Step 1: Write failing tests for Starter loading**

In `scripts/quest_prototype/src/data/card-database.test.ts`, update rarity expectations:

```ts
describe("RARITY_COLORS", () => {
  it("maps all 5 offer/display rarities to hex colors", () => {
    const expected: Record<Rarity, string> = {
      Common: "#ffffff",
      Uncommon: "#10b981",
      Rare: "#3b82f6",
      Legendary: "#a855f7",
      Starter: "#d4a017",
    };
    for (const [rarity, color] of Object.entries(expected)) {
      expect(RARITY_COLORS[rarity as Rarity]).toBe(color);
    }
  });
});
```

In the real-data integration test, change the size test:

```ts
it("loads 592 non-Special cards and indexes them by cardNumber", async () => {
  const raw = await readCardDataJson();
  if (raw === null) return;

  vi.stubGlobal(
    "fetch",
    vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(raw),
    }),
  );

  const db = await loadCardDatabase();
  expect(db.size).toBe(592);
});
```

In the required fields integration test, include Starter:

```ts
const validRarities = new Set([
  "Common",
  "Uncommon",
  "Rare",
  "Legendary",
  "Starter",
]);
```

Add this integration test:

```ts
it("includes the 10 Starter loadout cards", async () => {
  const raw = await readCardDataJson();
  if (raw === null) return;

  const starters = raw.filter((card) => card.rarity === "Starter");
  expect(starters.map((card) => card.cardNumber).sort((a, b) => a - b)).toEqual([
    711, 712, 713, 714, 715, 716, 717, 718, 719, 720,
  ]);
});
```

- [ ] **Step 2: Run the focused test and verify it fails before setup-assets changes**

Run:

```bash
npm test -- src/data/card-database.test.ts
```

Expected: FAIL. Acceptable failures are `db.size` still being old and the real data containing 0 Starter cards.

- [ ] **Step 3: Extend Rarity type**

In `scripts/quest_prototype/src/types/cards.ts`, replace the `Rarity` export with:

```ts
/** Card rarity levels. Special is excluded from normal prototype card-data. */
export type Rarity =
  | "Common"
  | "Uncommon"
  | "Rare"
  | "Legendary"
  | "Starter";
```

- [ ] **Step 4: Add Starter display color**

In `scripts/quest_prototype/src/data/card-database.ts`, replace `RARITY_COLORS` with:

```ts
/** Display color hex value for each rarity. */
export const RARITY_COLORS: Readonly<Record<Rarity, string>> = {
  Common: "#ffffff",
  Uncommon: "#10b981",
  Rare: "#3b82f6",
  Legendary: "#a855f7",
  Starter: "#d4a017",
};
```

- [ ] **Step 5: Keep Starter cards in asset output**

In `scripts/quest_prototype/scripts/setup-assets.mjs`, keep this filter as-is:

```js
const cards = allCards.filter((c) => c.rarity !== "Special");
```

If it was changed locally to exclude Starter, revert only that local exclusion.

- [ ] **Step 6: Update synthetic-data test rarity set**

In `scripts/quest_prototype/src/data/synthetic-data.test.ts`, use:

```ts
const ALL_RARITIES: Rarity[] = [
  "Common",
  "Uncommon",
  "Rare",
  "Legendary",
  "Starter",
];
```

- [ ] **Step 7: Verify CardDisplay has a fallback for all RARITY_COLORS**

If `CardDisplay` has a hard-coded rarity color switch, replace it with:

```ts
import { RARITY_COLORS } from "../data/card-database";
```

and read `RARITY_COLORS[card.rarity]` instead of a local four-rarity map.

- [ ] **Step 8: Regenerate prototype assets**

Run:

```bash
npm run setup-assets
```

Expected output includes:

```text
Filtered to 592 non-Special cards
Wrote 592 cards to card-data.json
```

- [ ] **Step 9: Run focused tests**

Run:

```bash
npm test -- src/data/card-database.test.ts src/data/synthetic-data.test.ts
npm run typecheck
```

Expected: PASS.

- [ ] **Step 10: Commit**

```bash
git status --short
git add scripts/quest_prototype/scripts/setup-assets.mjs scripts/quest_prototype/src/types/cards.ts scripts/quest_prototype/src/data/card-database.ts scripts/quest_prototype/src/data/card-database.test.ts scripts/quest_prototype/src/data/synthetic-data.test.ts scripts/quest_prototype/src/components/CardDisplay.tsx
git commit -m "feat(quest): load Starter cards in prototype data"
```

Do not stage `scripts/quest_prototype/public/card-data.json` unless it is already tracked.

---

## Task 2: Card Pool Helpers

**Files:**
- Create: `scripts/quest_prototype/src/data/card-pools.ts`
- Create: `scripts/quest_prototype/src/data/card-pools.test.ts`

- [ ] **Step 1: Write card-pool tests**

Create `scripts/quest_prototype/src/data/card-pools.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import type { CardData, Rarity, Tide } from "../types/cards";
import {
  findStarterCards,
  offerableCards,
  neutralStartingCandidates,
  randomStartingTideCandidates,
  draftPoolCards,
} from "./card-pools";

function makeCard(
  cardNumber: number,
  tide: Tide,
  rarity: Rarity = "Common",
): CardData {
  return {
    name: `Card ${String(cardNumber)}`,
    id: `card-${String(cardNumber)}`,
    cardNumber,
    cardType: "Character",
    subtype: "",
    rarity,
    energyCost: 1,
    spark: 1,
    isFast: false,
    tide,
    tideCost: tide === "Neutral" ? 0 : 1,
    renderedText: "",
    imageNumber: cardNumber,
    artOwned: false,
  };
}

describe("card-pools", () => {
  const db = new Map<number, CardData>([
    [1, makeCard(1, "Bloom", "Common")],
    [2, makeCard(2, "Bloom", "Starter")],
    [3, makeCard(3, "Bloom", "Legendary")],
    [4, makeCard(4, "Neutral", "Common")],
    [5, makeCard(5, "Neutral", "Legendary")],
    [6, makeCard(6, "Arc", "Rare")],
  ]);

  it("finds starter cards in card-number order", () => {
    expect(findStarterCards(db).map((card) => card.cardNumber)).toEqual([2]);
  });

  it("offerableCards excludes Starter and debug-excluded tides", () => {
    expect(
      offerableCards(db, { excludedTides: ["Arc"] }).map((card) => card.cardNumber),
    ).toEqual([1, 3, 4, 5]);
  });

  it("randomStartingTideCandidates excludes Starter and Special-equivalent offer bans", () => {
    expect(
      randomStartingTideCandidates(db, "Bloom").map((card) => card.cardNumber),
    ).toEqual([1, 3]);
  });

  it("neutralStartingCandidates excludes Neutral Legendary", () => {
    expect(neutralStartingCandidates(db).map((card) => card.cardNumber)).toEqual([4]);
  });

  it("draftPoolCards excludes consumed starting grants", () => {
    expect(
      draftPoolCards(db, {
        excludedTides: [],
        consumedCardNumbers: new Set([1, 4]),
      }).map((card) => card.cardNumber),
    ).toEqual([3, 5, 6]);
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
npm test -- src/data/card-pools.test.ts
```

Expected: FAIL with an import error for `./card-pools`.

- [ ] **Step 3: Implement card-pools.ts**

Create `scripts/quest_prototype/src/data/card-pools.ts`:

```ts
import type { CardData, Tide } from "../types/cards";

interface PoolOptions {
  excludedTides?: readonly Tide[];
  consumedCardNumbers?: ReadonlySet<number>;
}

function sorted(cards: CardData[]): CardData[] {
  return [...cards].sort((a, b) => a.cardNumber - b.cardNumber);
}

function allCards(cardDatabase: ReadonlyMap<number, CardData>): CardData[] {
  return sorted(Array.from(cardDatabase.values()));
}

function isExcludedTide(card: CardData, excludedTides: readonly Tide[]): boolean {
  return excludedTides.includes(card.tide);
}

/** Fixed loadout cards; never random offer cards. */
export function findStarterCards(
  cardDatabase: ReadonlyMap<number, CardData>,
): CardData[] {
  return allCards(cardDatabase).filter((card) => card.rarity === "Starter");
}

/** Infinite offer pool for shops/rewards; excludes loadout-only cards. */
export function offerableCards(
  cardDatabase: ReadonlyMap<number, CardData>,
  options: PoolOptions = {},
): CardData[] {
  const excludedTides = options.excludedTides ?? [];
  return allCards(cardDatabase).filter(
    (card) => card.rarity !== "Starter" && !isExcludedTide(card, excludedTides),
  );
}

/** Named-tide candidates for the 10 random cards granted at quest start. */
export function randomStartingTideCandidates(
  cardDatabase: ReadonlyMap<number, CardData>,
  tide: Exclude<Tide, "Neutral">,
): CardData[] {
  return offerableCards(cardDatabase).filter((card) => card.tide === tide);
}

/** Neutral candidates for quest start. Neutral legendaries are excluded. */
export function neutralStartingCandidates(
  cardDatabase: ReadonlyMap<number, CardData>,
): CardData[] {
  return offerableCards(cardDatabase).filter(
    (card) => card.tide === "Neutral" && card.rarity !== "Legendary",
  );
}

/** Finite draft pool. Random starting grants are removed from this pool. */
export function draftPoolCards(
  cardDatabase: ReadonlyMap<number, CardData>,
  options: PoolOptions = {},
): CardData[] {
  const consumed = options.consumedCardNumbers ?? new Set<number>();
  return offerableCards(cardDatabase, options).filter(
    (card) => !consumed.has(card.cardNumber),
  );
}
```

- [ ] **Step 4: Run focused test**

Run:

```bash
npm test -- src/data/card-pools.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git status --short
git add scripts/quest_prototype/src/data/card-pools.ts scripts/quest_prototype/src/data/card-pools.test.ts
git commit -m "feat(quest): add prototype card pool eligibility helpers"
```

---

## Task 3: Tide Circle And Starting Deck Generation

**Files:**
- Create: `scripts/quest_prototype/src/data/tide-circle.ts`
- Create: `scripts/quest_prototype/src/data/tide-circle.test.ts`
- Create: `scripts/quest_prototype/src/quest-start/quest-start-generator.ts`
- Create: `scripts/quest_prototype/src/quest-start/quest-start-generator.test.ts`
- Modify: `scripts/quest_prototype/src/types/cards.ts`

- [ ] **Step 1: Add NamedTide type**

In `scripts/quest_prototype/src/types/cards.ts`, add below `Tide`:

```ts
/** The 7 non-neutral tides on the tide circle. */
export type NamedTide = Exclude<Tide, "Neutral">;
```

- [ ] **Step 2: Write tide-circle tests**

Create `scripts/quest_prototype/src/data/tide-circle.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import {
  leftNeighbor,
  rightNeighbor,
  normalizedPairKey,
  pairContainsTide,
} from "./tide-circle";

describe("tide-circle", () => {
  it("finds wraparound neighbors", () => {
    expect(leftNeighbor("Bloom")).toBe("Surge");
    expect(rightNeighbor("Bloom")).toBe("Arc");
    expect(leftNeighbor("Surge")).toBe("Rime");
    expect(rightNeighbor("Surge")).toBe("Bloom");
  });

  it("normalizes unordered tide-pair keys", () => {
    expect(normalizedPairKey(["Bloom", "Arc"])).toBe("Bloom/Arc");
    expect(normalizedPairKey(["Arc", "Bloom"])).toBe("Bloom/Arc");
    expect(normalizedPairKey(["Surge", "Bloom"])).toBe("Bloom/Surge");
  });

  it("checks pair membership", () => {
    expect(pairContainsTide(["Bloom", "Arc"], "Bloom")).toBe(true);
    expect(pairContainsTide(["Bloom", "Arc"], "Pact")).toBe(false);
  });
});
```

- [ ] **Step 3: Implement tide-circle.ts**

Create `scripts/quest_prototype/src/data/tide-circle.ts`:

```ts
import { NAMED_TIDES } from "./card-database";
import type { NamedTide } from "../types/cards";

function indexOfNamed(tide: NamedTide): number {
  return NAMED_TIDES.indexOf(tide);
}

/** Named tide immediately counter-clockwise on the revised tide circle. */
export function leftNeighbor(tide: NamedTide): NamedTide {
  const index = indexOfNamed(tide);
  return NAMED_TIDES[(index + NAMED_TIDES.length - 1) % NAMED_TIDES.length];
}

/** Named tide immediately clockwise on the revised tide circle. */
export function rightNeighbor(tide: NamedTide): NamedTide {
  const index = indexOfNamed(tide);
  return NAMED_TIDES[(index + 1) % NAMED_TIDES.length];
}

/** Stable key for an unordered two-tide dreamcaller pair. */
export function normalizedPairKey(pair: readonly [NamedTide, NamedTide]): string {
  const [a, b] = pair;
  return indexOfNamed(a) <= indexOfNamed(b) ? `${a}/${b}` : `${b}/${a}`;
}

/** Returns true when a two-tide pair contains the given named tide. */
export function pairContainsTide(
  pair: readonly [NamedTide, NamedTide],
  tide: NamedTide,
): boolean {
  return pair[0] === tide || pair[1] === tide;
}
```

- [ ] **Step 4: Write quest-start generation tests**

Create `scripts/quest_prototype/src/quest-start/quest-start-generator.test.ts`:

```ts
import { describe, expect, it, vi } from "vitest";
import type { CardData, NamedTide, Rarity, Tide } from "../types/cards";
import {
  buildStartingDeckPlan,
  selectStartingTideOptions,
} from "./quest-start-generator";

function makeCard(
  cardNumber: number,
  tide: Tide,
  rarity: Rarity = "Common",
): CardData {
  return {
    name: `Card ${String(cardNumber)}`,
    id: `card-${String(cardNumber)}`,
    cardNumber,
    cardType: "Character",
    subtype: "",
    rarity,
    energyCost: 1,
    spark: 1,
    isFast: false,
    tide,
    tideCost: tide === "Neutral" ? 0 : 1,
    renderedText: "",
    imageNumber: cardNumber,
    artOwned: false,
  };
}

function makeDb(): Map<number, CardData> {
  const cards: CardData[] = [];
  for (let i = 0; i < 10; i++) cards.push(makeCard(711 + i, "Neutral", "Starter"));
  for (let i = 0; i < 12; i++) cards.push(makeCard(100 + i, "Bloom", i === 0 ? "Legendary" : "Common"));
  for (let i = 0; i < 11; i++) cards.push(makeCard(200 + i, "Neutral", i === 0 ? "Legendary" : "Common"));
  return new Map(cards.map((card) => [card.cardNumber, card]));
}

describe("selectStartingTideOptions", () => {
  it("returns 3 distinct named tides", () => {
    const options = selectStartingTideOptions([]);
    expect(options).toHaveLength(3);
    expect(new Set(options).size).toBe(3);
    expect(options).not.toContain("Neutral");
  });

  it("does not return debug-excluded tides", () => {
    const options = selectStartingTideOptions(["Bloom", "Arc", "Ignite", "Pact"] as NamedTide[]);
    expect(options).toHaveLength(3);
    expect(options).not.toContain("Bloom");
    expect(options).not.toContain("Arc");
    expect(options).not.toContain("Ignite");
    expect(options).not.toContain("Pact");
  });
});

describe("buildStartingDeckPlan", () => {
  it("creates 10 starter, 10 selected-tide, and 10 neutral cards", () => {
    vi.spyOn(Math, "random").mockReturnValue(0);
    const plan = buildStartingDeckPlan(makeDb(), "Bloom");
    expect(plan.starterCardNumbers).toHaveLength(10);
    expect(plan.tideCardNumbers).toHaveLength(10);
    expect(plan.neutralCardNumbers).toHaveLength(10);
    expect(plan.deckCardNumbers).toHaveLength(30);
    expect(plan.consumedRandomCardNumbers).toHaveLength(20);
  });

  it("does not include Neutral Legendary in the neutral package", () => {
    vi.spyOn(Math, "random").mockReturnValue(0);
    const plan = buildStartingDeckPlan(makeDb(), "Bloom");
    expect(plan.neutralCardNumbers).not.toContain(200);
  });

  it("does include named-tide Legendary when randomly sampled", () => {
    vi.spyOn(Math, "random").mockReturnValue(0);
    const plan = buildStartingDeckPlan(makeDb(), "Bloom");
    expect(plan.tideCardNumbers).toContain(100);
  });
});
```

- [ ] **Step 5: Run new tests to verify they fail**

Run:

```bash
npm test -- src/data/tide-circle.test.ts src/quest-start/quest-start-generator.test.ts
```

Expected: FAIL with import errors for the new modules.

- [ ] **Step 6: Implement quest-start-generator.ts**

Create `scripts/quest_prototype/src/quest-start/quest-start-generator.ts`:

```ts
import { NAMED_TIDES } from "../data/card-database";
import {
  findStarterCards,
  neutralStartingCandidates,
  randomStartingTideCandidates,
} from "../data/card-pools";
import type { CardData, NamedTide } from "../types/cards";

/** Card-number groups created when the quest origin is selected. */
export interface StartingDeckPlan {
  starterCardNumbers: number[];
  tideCardNumbers: number[];
  neutralCardNumbers: number[];
  deckCardNumbers: number[];
  consumedRandomCardNumbers: number[];
}

function shuffled<T>(items: readonly T[]): T[] {
  const result = [...items];
  for (let i = result.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [result[i], result[j]] = [result[j], result[i]];
  }
  return result;
}

function sampleCardNumbers(cards: CardData[], count: number, label: string): number[] {
  if (cards.length < count) {
    throw new Error(`${label} requires ${String(count)} cards; found ${String(cards.length)}`);
  }
  return shuffled(cards).slice(0, count).map((card) => card.cardNumber);
}

/** Generate the 3 starting tide choices for a quest. */
export function selectStartingTideOptions(
  excludedTides: readonly NamedTide[],
): NamedTide[] {
  const excluded = new Set(excludedTides);
  const options = NAMED_TIDES.filter((tide) => !excluded.has(tide));
  return shuffled(options).slice(0, 3);
}

/** Build the 30-card loadout for the selected starting tide. */
export function buildStartingDeckPlan(
  cardDatabase: ReadonlyMap<number, CardData>,
  startingTide: NamedTide,
): StartingDeckPlan {
  const starterCardNumbers = findStarterCards(cardDatabase).map(
    (card) => card.cardNumber,
  );
  if (starterCardNumbers.length !== 10) {
    throw new Error(`Expected 10 Starter cards; found ${String(starterCardNumbers.length)}`);
  }

  const tideCardNumbers = sampleCardNumbers(
    randomStartingTideCandidates(cardDatabase, startingTide),
    10,
    `${startingTide} starting package`,
  );
  const neutralCardNumbers = sampleCardNumbers(
    neutralStartingCandidates(cardDatabase),
    10,
    "Neutral starting package",
  );

  return {
    starterCardNumbers,
    tideCardNumbers,
    neutralCardNumbers,
    deckCardNumbers: [
      ...starterCardNumbers,
      ...tideCardNumbers,
      ...neutralCardNumbers,
    ],
    consumedRandomCardNumbers: [...tideCardNumbers, ...neutralCardNumbers],
  };
}
```

- [ ] **Step 7: Run focused tests**

Run:

```bash
npm test -- src/data/tide-circle.test.ts src/quest-start/quest-start-generator.test.ts src/data/card-pools.test.ts
```

Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git status --short
git add scripts/quest_prototype/src/types/cards.ts scripts/quest_prototype/src/data/tide-circle.ts scripts/quest_prototype/src/data/tide-circle.test.ts scripts/quest_prototype/src/quest-start/quest-start-generator.ts scripts/quest_prototype/src/quest-start/quest-start-generator.test.ts
git commit -m "feat(quest): generate starting tide choices and loadout"
```

---

## Task 4: Quest State And Start Screen

**Files:**
- Modify: `scripts/quest_prototype/src/types/quest.ts`
- Modify: `scripts/quest_prototype/src/state/quest-context.tsx`
- Modify: `scripts/quest_prototype/src/state/quest-config.ts`
- Modify: `scripts/quest_prototype/src/screens/QuestStartScreen.tsx`

- [ ] **Step 1: Change default excluded tide config**

In `scripts/quest_prototype/src/state/quest-config.ts`:

```ts
const DEFAULT_EXCLUDED_TIDE_COUNT = 0;
```

- [ ] **Step 2: Extend QuestState**

In `scripts/quest_prototype/src/types/quest.ts`, import `NamedTide`:

```ts
import type { NamedTide, Tide } from "./cards";
```

Add these fields to `QuestState`:

```ts
  /** Named tide selected at quest start. Null until the player chooses. */
  startingTide: NamedTide | null;
  /** Random starting grants removed from the finite draft pool. */
  consumedStartingCardNumbers: number[];
```

- [ ] **Step 3: Add quest-start mutation to context interface**

In `QuestMutations`:

```ts
  chooseStartingTide: (
    startingTide: NamedTide,
    starterCardNumbers: number[],
    tideCardNumbers: number[],
    neutralCardNumbers: number[],
    consumedRandomCardNumbers: number[],
  ) => void;
```

Also update the imports:

```ts
import type { CardData, NamedTide, Tide } from "../types/cards";
```

- [ ] **Step 4: Add default state fields**

In `createDefaultState()`:

```ts
    startingTide: null,
    consumedStartingCardNumbers: [],
```

- [ ] **Step 5: Implement chooseStartingTide mutation**

In `quest-context.tsx`, add:

```ts
  const chooseStartingTide = useCallback(
    (
      startingTide: NamedTide,
      starterCardNumbers: number[],
      tideCardNumbers: number[],
      neutralCardNumbers: number[],
      consumedRandomCardNumbers: number[],
    ) => {
      logEvent("starting_tide_selected", {
        startingTide,
        grantedCrystal: startingTide,
      });
      logEvent("starting_deck_initialized", {
        startingTide,
        starterCardNumbers,
        tideCardNumbers,
        neutralCardNumbers,
        totalDeckSize:
          starterCardNumbers.length + tideCardNumbers.length + neutralCardNumbers.length,
      });

      setState((prev) => {
        const allCardNumbers = [
          ...starterCardNumbers,
          ...tideCardNumbers,
          ...neutralCardNumbers,
        ];
        const addedDeckEntries: DeckEntry[] = allCardNumbers.map((cardNumber) => {
          const entry: DeckEntry = {
            entryId: nextEntryId(),
            cardNumber,
            transfiguration: null,
            isBane: false,
          };
          return entry;
        });

        return {
          ...prev,
          startingTide,
          consumedStartingCardNumbers: consumedRandomCardNumbers,
          deck: [...prev.deck, ...addedDeckEntries],
          tideCrystals: {
            ...prev.tideCrystals,
            [startingTide]: prev.tideCrystals[startingTide] + 1,
          },
        };
      });
    },
    [],
  );
```

Add it to `mutations` object and dependency array.

- [ ] **Step 6: Rewrite QuestStartScreen around tide options**

In `QuestStartScreen.tsx`, use these imports:

```ts
import { useCallback, useRef } from "react";
import { motion } from "framer-motion";
import { generateInitialAtlas } from "../atlas/atlas-generator";
import { NAMED_TIDES, TIDE_COLORS, tideIconUrl } from "../data/card-database";
import { DREAMSIGNS } from "../data/dreamsigns";
import { buildStartingDeckPlan, selectStartingTideOptions } from "../quest-start/quest-start-generator";
import { logEvent } from "../logging";
import { useQuest } from "../state/quest-context";
import { useQuestConfig } from "../state/quest-config";
import type { NamedTide, Tide } from "../types/cards";
```

Keep `selectExcludedTides` for debug config.

Inside `QuestStartScreen`, generate stable options:

```ts
  const excludedTidesRef = useRef<Tide[] | null>(null);
  const startingOptionsRef = useRef<NamedTide[] | null>(null);

  if (excludedTidesRef.current === null) {
    excludedTidesRef.current = selectExcludedTides(config.excludedTideCount);
  }
  const excludedTides = excludedTidesRef.current;

  if (startingOptionsRef.current === null) {
    const options = selectStartingTideOptions(excludedTides as NamedTide[]);
    startingOptionsRef.current = options;
    logEvent("starting_tide_options_generated", { options });
  }
  const startingOptions = startingOptionsRef.current;
```

Replace `handleBeginQuest` with:

```ts
  const handleChooseStartingTide = useCallback(
    (startingTide: NamedTide) => {
      mutations.setExcludedTides(excludedTides);
      const startingDeck = buildStartingDeckPlan(cardDatabase, startingTide);
      mutations.chooseStartingTide(
        startingTide,
        startingDeck.starterCardNumbers,
        startingDeck.tideCardNumbers,
        startingDeck.neutralCardNumbers,
        startingDeck.consumedRandomCardNumbers,
      );

      const atlas = generateInitialAtlas(state.completionLevel, {
        cardDatabase,
        dreamsignPool: DREAMSIGNS,
        playerHasBanes: false,
        excludedTides,
      });
      const nodeCount = Object.keys(atlas.nodes).length - 1;

      logEvent("quest_started", {
        initialEssence: state.essence,
        initialDeckSize: 30,
        startingTide,
        dreamscapesGenerated: nodeCount,
        excludedTides,
      });

      mutations.updateAtlas(atlas);
      mutations.setScreen({ type: "atlas" });
    },
    [cardDatabase, excludedTides, mutations, state.completionLevel, state.essence],
  );
```

Render one button/card per `startingOptions`; each button calls:

```ts
onClick={() => {
  handleChooseStartingTide(tide);
}}
```

Each option should display:

```tsx
<img src={tideIconUrl(tide)} alt={tide} />
<span style={{ color: TIDE_COLORS[tide] }}>{tide}</span>
<span>Start with 10 {tide} cards, 10 Starter cards, 10 Neutral cards, and 1 {tide} crystal.</span>
```

- [ ] **Step 7: Run focused verification**

Run:

```bash
npm test -- src/quest-start/quest-start-generator.test.ts
npm run typecheck
```

Expected: PASS.

- [ ] **Step 8: Run browser smoke check**

Start dev server:

```bash
npm run dev
```

Open the app:

```bash
/Users/dthurn/Library/pnpm/agent-browser open http://localhost:5173
/Users/dthurn/Library/pnpm/agent-browser wait --load networkidle
/Users/dthurn/Library/pnpm/agent-browser screenshot /tmp/quest-starting-tide-qa/start-options.png
```

Expected screenshot: title plus exactly 3 named tide choices; no Neutral option.

- [ ] **Step 9: Commit**

```bash
git status --short
git add scripts/quest_prototype/src/types/quest.ts scripts/quest_prototype/src/state/quest-context.tsx scripts/quest_prototype/src/state/quest-config.ts scripts/quest_prototype/src/screens/QuestStartScreen.tsx
git commit -m "feat(quest): start quests from a selected tide"
```

---

## Task 5: Shared Quest Tide Profile

**Files:**
- Create: `scripts/quest_prototype/src/data/quest-tide-profile.ts`
- Create: `scripts/quest_prototype/src/data/quest-tide-profile.test.ts`
- Modify: `scripts/quest_prototype/src/data/tide-weights.ts`

- [ ] **Step 1: Write profile tests**

Create `scripts/quest_prototype/src/data/quest-tide-profile.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import type { CardData, NamedTide, Rarity, Tide } from "../types/cards";
import type { DeckEntry, Dreamcaller } from "../types/quest";
import {
  computeQuestTideProfile,
  tideProfileWeight,
  weightedSampleByProfile,
} from "./quest-tide-profile";

function makeCard(cardNumber: number, tide: Tide, rarity: Rarity = "Common"): CardData {
  return {
    name: `Card ${String(cardNumber)}`,
    id: `card-${String(cardNumber)}`,
    cardNumber,
    cardType: "Character",
    subtype: "",
    rarity,
    energyCost: 1,
    spark: 1,
    isFast: false,
    tide,
    tideCost: tide === "Neutral" ? 0 : 1,
    renderedText: "",
    imageNumber: cardNumber,
    artOwned: false,
  };
}

function entry(cardNumber: number): DeckEntry {
  return {
    entryId: `entry-${String(cardNumber)}`,
    cardNumber,
    transfiguration: null,
    isBane: false,
  };
}

function caller(tides: [NamedTide, NamedTide]): Dreamcaller {
  return {
    name: `${tides[0]} ${tides[1]} Caller`,
    tides,
    abilityDescription: "Test ability.",
    essenceBonus: 100,
    tideCrystalGrant: tides[0],
  };
}

describe("computeQuestTideProfile", () => {
  const db = new Map<number, CardData>([
    [1, makeCard(1, "Neutral", "Starter")],
    [2, makeCard(2, "Neutral")],
    [3, makeCard(3, "Pact")],
    [4, makeCard(4, "Pact")],
    [5, makeCard(5, "Pact")],
    [6, makeCard(6, "Pact")],
  ]);

  it("starts with origin strongest and neighbors secondary", () => {
    const profile = computeQuestTideProfile({
      startingTide: "Bloom",
      deck: [],
      cardDatabase: db,
      dreamcaller: null,
      tideCrystals: { Bloom: 1, Arc: 0, Ignite: 0, Pact: 0, Umbra: 0, Rime: 0, Surge: 0, Neutral: 0 },
      recentDraftPicks: [],
    });
    expect(profile.weights.Bloom).toBeGreaterThan(profile.weights.Arc);
    expect(profile.weights.Arc).toBeGreaterThan(profile.weights.Pact);
    expect(profile.weights.Surge).toBeGreaterThan(profile.weights.Pact);
  });

  it("ignores Starter cards and heavily discounts Neutral deck cards", () => {
    const profile = computeQuestTideProfile({
      startingTide: "Bloom",
      deck: [entry(1), entry(2)],
      cardDatabase: db,
      dreamcaller: null,
      tideCrystals: { Bloom: 1, Arc: 0, Ignite: 0, Pact: 0, Umbra: 0, Rime: 0, Surge: 0, Neutral: 0 },
      recentDraftPicks: [],
    });
    expect(profile.weights.Neutral).toBeLessThan(profile.weights.Bloom);
  });

  it("can pivot toward many non-neutral deck cards", () => {
    const profile = computeQuestTideProfile({
      startingTide: "Bloom",
      deck: [entry(3), entry(4), entry(5), entry(6)],
      cardDatabase: db,
      dreamcaller: null,
      tideCrystals: { Bloom: 1, Arc: 0, Ignite: 0, Pact: 0, Umbra: 0, Rime: 0, Surge: 0, Neutral: 0 },
      recentDraftPicks: [],
    });
    expect(profile.weights.Pact).toBeGreaterThan(profile.weights.Bloom);
  });

  it("adds weight for both dreamcaller tides", () => {
    const profile = computeQuestTideProfile({
      startingTide: "Bloom",
      deck: [],
      cardDatabase: db,
      dreamcaller: caller(["Umbra", "Rime"]),
      tideCrystals: { Bloom: 1, Arc: 0, Ignite: 0, Pact: 0, Umbra: 0, Rime: 0, Surge: 0, Neutral: 0 },
      recentDraftPicks: [],
    });
    expect(profile.contributions.dreamcaller.Umbra).toBeGreaterThan(0);
    expect(profile.contributions.dreamcaller.Rime).toBeGreaterThan(0);
  });
});

describe("weightedSampleByProfile", () => {
  it("samples available cards and never creates duplicates", () => {
    const cards = [makeCard(10, "Bloom"), makeCard(11, "Bloom"), makeCard(12, "Pact")];
    const profile = computeQuestTideProfile({
      startingTide: "Bloom",
      deck: [],
      cardDatabase: new Map(cards.map((card) => [card.cardNumber, card])),
      dreamcaller: null,
      tideCrystals: { Bloom: 1, Arc: 0, Ignite: 0, Pact: 0, Umbra: 0, Rime: 0, Surge: 0, Neutral: 0 },
      recentDraftPicks: [],
    });
    const sample = weightedSampleByProfile(cards, profile, 2);
    expect(sample).toHaveLength(2);
    expect(new Set(sample.map((card) => card.cardNumber)).size).toBe(2);
    expect(tideProfileWeight(profile, "Bloom")).toBeGreaterThan(tideProfileWeight(profile, "Pact"));
  });
});
```

- [ ] **Step 2: Run profile test to verify it fails**

Run:

```bash
npm test -- src/data/quest-tide-profile.test.ts
```

Expected: FAIL with import error.

- [ ] **Step 3: Implement quest-tide-profile.ts**

Create `scripts/quest_prototype/src/data/quest-tide-profile.ts` with:

```ts
import { NAMED_TIDES } from "./card-database";
import { leftNeighbor, rightNeighbor } from "./tide-circle";
import type { CardData, NamedTide, Tide } from "../types/cards";
import type { DeckEntry, Dreamcaller } from "../types/quest";

type TideWeightMap = Record<Tide, number>;

interface ProfileInput {
  startingTide: NamedTide | null;
  deck: readonly DeckEntry[];
  cardDatabase: ReadonlyMap<number, CardData>;
  dreamcaller: Dreamcaller | null;
  tideCrystals: Record<Tide, number>;
  recentDraftPicks: readonly number[];
}

/** Complete weighting profile used by quest offer generators. */
export interface QuestTideProfile {
  weights: TideWeightMap;
  contributions: {
    start: TideWeightMap;
    deck: TideWeightMap;
    dreamcaller: TideWeightMap;
    crystals: TideWeightMap;
    recentDraft: TideWeightMap;
    baseline: TideWeightMap;
  };
}

function emptyWeights(): TideWeightMap {
  return {
    Bloom: 0,
    Arc: 0,
    Ignite: 0,
    Pact: 0,
    Umbra: 0,
    Rime: 0,
    Surge: 0,
    Neutral: 0,
  };
}

function add(weights: TideWeightMap, tide: Tide, amount: number): void {
  weights[tide] += amount;
}

function sumContributions(contributions: QuestTideProfile["contributions"]): TideWeightMap {
  const weights = emptyWeights();
  for (const contribution of Object.values(contributions)) {
    for (const tide of [...NAMED_TIDES, "Neutral" as const]) {
      weights[tide] += contribution[tide];
    }
  }
  return weights;
}

function addStartContribution(weights: TideWeightMap, startingTide: NamedTide | null): void {
  if (startingTide === null) return;
  add(weights, startingTide, 5);
  add(weights, leftNeighbor(startingTide), 2);
  add(weights, rightNeighbor(startingTide), 2);
}

function addDeckContribution(
  weights: TideWeightMap,
  deck: readonly DeckEntry[],
  cardDatabase: ReadonlyMap<number, CardData>,
): void {
  for (const entry of deck) {
    const card = cardDatabase.get(entry.cardNumber);
    if (!card || card.rarity === "Starter") continue;
    add(weights, card.tide, card.tide === "Neutral" ? 0.15 : 1.75);
  }
}

function addDreamcallerContribution(weights: TideWeightMap, dreamcaller: Dreamcaller | null): void {
  if (dreamcaller === null) return;
  add(weights, dreamcaller.tides[0], 3);
  add(weights, dreamcaller.tides[1], 3);
}

function addCrystalContribution(weights: TideWeightMap, tideCrystals: Record<Tide, number>): void {
  for (const tide of NAMED_TIDES) {
    add(weights, tide, tideCrystals[tide] * 0.75);
  }
}

function addRecentDraftContribution(
  weights: TideWeightMap,
  recentDraftPicks: readonly number[],
  cardDatabase: ReadonlyMap<number, CardData>,
): void {
  for (let index = 0; index < Math.min(5, recentDraftPicks.length); index++) {
    const card = cardDatabase.get(recentDraftPicks[index]);
    if (!card || card.tide === "Neutral" || card.rarity === "Starter") continue;
    add(weights, card.tide, 1.25 * Math.pow(0.75, index));
  }
}

/** Computes the shared adaptive tide profile for quest generation. */
export function computeQuestTideProfile(input: ProfileInput): QuestTideProfile {
  const baseline = emptyWeights();
  for (const tide of NAMED_TIDES) add(baseline, tide, 1);
  add(baseline, "Neutral", 0.7);

  const start = emptyWeights();
  addStartContribution(start, input.startingTide);
  const deck = emptyWeights();
  addDeckContribution(deck, input.deck, input.cardDatabase);
  const dreamcaller = emptyWeights();
  addDreamcallerContribution(dreamcaller, input.dreamcaller);
  const crystals = emptyWeights();
  addCrystalContribution(crystals, input.tideCrystals);
  const recentDraft = emptyWeights();
  addRecentDraftContribution(recentDraft, input.recentDraftPicks, input.cardDatabase);

  const contributions = { start, deck, dreamcaller, crystals, recentDraft, baseline };
  return { weights: sumContributions(contributions), contributions };
}

/** Returns a non-zero sample weight for a tide-bearing item. */
export function tideProfileWeight(profile: QuestTideProfile, tide: Tide): number {
  return Math.max(0.01, profile.weights[tide]);
}

/** Weighted sample without replacement for CardData-like objects. */
export function weightedSampleByProfile<T extends { tide: Tide }>(
  pool: readonly T[],
  profile: QuestTideProfile,
  count: number,
): T[] {
  const remaining = [...pool];
  const selected: T[] = [];
  while (selected.length < count && remaining.length > 0) {
    const total = remaining.reduce(
      (sum, item) => sum + tideProfileWeight(profile, item.tide),
      0,
    );
    let roll = Math.random() * total;
    let selectedIndex = remaining.length - 1;
    for (let index = 0; index < remaining.length; index++) {
      roll -= tideProfileWeight(profile, remaining[index].tide);
      if (roll <= 0) {
        selectedIndex = index;
        break;
      }
    }
    selected.push(remaining[selectedIndex]);
    remaining.splice(selectedIndex, 1);
  }
  return selected;
}
```

- [ ] **Step 4: Run focused profile tests**

Run:

```bash
npm test -- src/data/quest-tide-profile.test.ts
npm run typecheck
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git status --short
git add scripts/quest_prototype/src/data/quest-tide-profile.ts scripts/quest_prototype/src/data/quest-tide-profile.test.ts
git commit -m "feat(quest): compute shared quest tide profile"
```

---

## Task 6: Two-Tide Dreamcallers And Offers

**Files:**
- Modify: `scripts/quest_prototype/src/types/quest.ts`
- Modify: `scripts/quest_prototype/src/data/dreamcallers.ts`
- Create: `scripts/quest_prototype/src/data/dreamcaller-offers.ts`
- Create: `scripts/quest_prototype/src/data/dreamcaller-offers.test.ts`
- Modify: `scripts/quest_prototype/src/data/synthetic-data.test.ts`
- Modify: `scripts/quest_prototype/src/screens/DreamcallerDraftScreen.tsx`
- Modify: `scripts/quest_prototype/src/state/quest-context.tsx`
- Modify: `scripts/quest_prototype/src/screens/BattleScreen.tsx`
- Modify: `scripts/quest_prototype/src/screens/QuestCompleteScreen.tsx`
- Modify: `scripts/quest_prototype/src/components/DeckViewer.tsx`

- [ ] **Step 1: Update Dreamcaller type**

In `scripts/quest_prototype/src/types/quest.ts`:

```ts
/** A selected character that grants bonuses. */
export interface Dreamcaller {
  name: string;
  tides: [NamedTide, NamedTide];
  abilityDescription: string;
  essenceBonus: number;
  tideCrystalGrant: NamedTide;
}
```

- [ ] **Step 2: Update synthetic dreamcallers data**

In `scripts/quest_prototype/src/data/dreamcallers.ts`, change every entry:

```ts
{
  name: "Lyria, Tide Weaver",
  tides: ["Bloom", "Arc"],
  abilityDescription:
    "She draws vitality from the roots of dreaming trees, mending wounds with whispered verse.",
  essenceBonus: 80,
  tideCrystalGrant: "Bloom",
},
```

Assign the 10 existing callers to named-tide pairs. Cover all 7 neighbor pairs at least once:

```text
Bloom/Arc
Arc/Ignite
Ignite/Pact
Pact/Umbra
Umbra/Rime
Rime/Surge
Surge/Bloom
```

Remove the Neutral dreamcaller by converting that entry to one of the named pairs.

- [ ] **Step 3: Update synthetic-data dreamcaller tests**

Replace the dreamcaller required-field test with:

```ts
it("every entry has two named tides and a matching named crystal", () => {
  for (const dc of DREAMCALLERS) {
    expect(dc.name.length).toBeGreaterThan(0);
    expect(dc.tides).toHaveLength(2);
    expect(dc.tides[0]).not.toBe(dc.tides[1]);
    expect(tideSet.has(dc.tides[0])).toBe(true);
    expect(tideSet.has(dc.tides[1])).toBe(true);
    expect(dc.tides).not.toContain("Neutral");
    expect(dc.abilityDescription.length).toBeGreaterThan(0);
    expect(dc.essenceBonus).toBeGreaterThanOrEqual(50);
    expect(dc.essenceBonus).toBeLessThanOrEqual(150);
    expect(dc.tides).toContain(dc.tideCrystalGrant);
  }
});
```

Replace coverage test with:

```ts
it("covers every named tide at least once", () => {
  const tides = new Set(DREAMCALLERS.flatMap((dc) => dc.tides));
  for (const t of NAMED_TIDES) {
    expect(tides.has(t)).toBe(true);
  }
});
```

- [ ] **Step 4: Write dreamcaller-offers tests**

Create `scripts/quest_prototype/src/data/dreamcaller-offers.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import type { Dreamcaller } from "../types/quest";
import type { NamedTide } from "../types/cards";
import { selectOfferedDreamcallers } from "./dreamcaller-offers";
import type { QuestTideProfile } from "./quest-tide-profile";

function caller(name: string, tides: [NamedTide, NamedTide]): Dreamcaller {
  return {
    name,
    tides,
    abilityDescription: "Ability.",
    essenceBonus: 100,
    tideCrystalGrant: tides[0],
  };
}

function profile(): QuestTideProfile {
  return {
    weights: { Bloom: 8, Arc: 4, Ignite: 1, Pact: 1, Umbra: 1, Rime: 1, Surge: 4, Neutral: 0.5 },
    contributions: {
      start: { Bloom: 5, Arc: 2, Ignite: 0, Pact: 0, Umbra: 0, Rime: 0, Surge: 2, Neutral: 0 },
      deck: { Bloom: 0, Arc: 0, Ignite: 0, Pact: 0, Umbra: 0, Rime: 0, Surge: 0, Neutral: 0 },
      dreamcaller: { Bloom: 0, Arc: 0, Ignite: 0, Pact: 0, Umbra: 0, Rime: 0, Surge: 0, Neutral: 0 },
      crystals: { Bloom: 0, Arc: 0, Ignite: 0, Pact: 0, Umbra: 0, Rime: 0, Surge: 0, Neutral: 0 },
      recentDraft: { Bloom: 0, Arc: 0, Ignite: 0, Pact: 0, Umbra: 0, Rime: 0, Surge: 0, Neutral: 0 },
      baseline: { Bloom: 1, Arc: 1, Ignite: 1, Pact: 1, Umbra: 1, Rime: 1, Surge: 1, Neutral: 0.5 },
    },
  };
}

describe("selectOfferedDreamcallers", () => {
  const pool = [
    caller("left", ["Surge", "Bloom"]),
    caller("right", ["Bloom", "Arc"]),
    caller("adaptive", ["Umbra", "Rime"]),
    caller("other", ["Ignite", "Pact"]),
  ];

  it("offers the two starting-tide neighbor forks when available", () => {
    const offered = selectOfferedDreamcallers({
      pool,
      startingTide: "Bloom",
      profile: profile(),
    });
    expect(offered).toHaveLength(3);
    expect(offered.map((dc) => dc.name)).toContain("left");
    expect(offered.map((dc) => dc.name)).toContain("right");
  });

  it("does not return duplicate dreamcallers", () => {
    const offered = selectOfferedDreamcallers({
      pool,
      startingTide: "Bloom",
      profile: profile(),
    });
    expect(new Set(offered.map((dc) => dc.name)).size).toBe(offered.length);
  });
});
```

- [ ] **Step 5: Implement dreamcaller-offers.ts**

Create `scripts/quest_prototype/src/data/dreamcaller-offers.ts`:

```ts
import { leftNeighbor, normalizedPairKey, pairContainsTide, rightNeighbor } from "./tide-circle";
import { tideProfileWeight, type QuestTideProfile } from "./quest-tide-profile";
import type { NamedTide } from "../types/cards";
import type { Dreamcaller } from "../types/quest";

interface OfferInput {
  pool: readonly Dreamcaller[];
  startingTide: NamedTide | null;
  profile: QuestTideProfile;
}

function dreamcallerWeight(dreamcaller: Dreamcaller, profile: QuestTideProfile): number {
  return (
    tideProfileWeight(profile, dreamcaller.tides[0]) +
    tideProfileWeight(profile, dreamcaller.tides[1])
  ) / 2;
}

function takeWeighted(
  pool: Dreamcaller[],
  profile: QuestTideProfile,
): Dreamcaller | null {
  if (pool.length === 0) return null;
  const total = pool.reduce((sum, dc) => sum + dreamcallerWeight(dc, profile), 0);
  let roll = Math.random() * total;
  for (const dc of pool) {
    roll -= dreamcallerWeight(dc, profile);
    if (roll <= 0) return dc;
  }
  return pool[pool.length - 1];
}

function takeMatchingPair(
  remaining: Dreamcaller[],
  pair: [NamedTide, NamedTide],
): Dreamcaller | null {
  const key = normalizedPairKey(pair);
  return remaining.find((dc) => normalizedPairKey(dc.tides) === key) ?? null;
}

function removeByName(pool: Dreamcaller[], selected: Dreamcaller): Dreamcaller[] {
  return pool.filter((dc) => dc.name !== selected.name);
}

/** Selects 3 dreamcaller offers: left fork, right fork, adaptive slot. */
export function selectOfferedDreamcallers(input: OfferInput): Dreamcaller[] {
  let remaining = [...input.pool];
  const offered: Dreamcaller[] = [];

  function add(dc: Dreamcaller | null): void {
    if (dc === null || offered.some((existing) => existing.name === dc.name)) return;
    offered.push(dc);
    remaining = removeByName(remaining, dc);
  }

  if (input.startingTide !== null) {
    add(takeMatchingPair(remaining, [leftNeighbor(input.startingTide), input.startingTide]));
    add(takeMatchingPair(remaining, [input.startingTide, rightNeighbor(input.startingTide)]));
  }

  while (offered.length < 3) {
    const fallback =
      input.startingTide === null
        ? null
        : remaining.find((dc) => pairContainsTide(dc.tides, input.startingTide)) ??
          remaining.find((dc) => pairContainsTide(dc.tides, leftNeighbor(input.startingTide)) || pairContainsTide(dc.tides, rightNeighbor(input.startingTide)));
    add(fallback ?? takeWeighted(remaining, input.profile));
    if (remaining.length === 0) break;
  }

  return offered;
}
```

- [ ] **Step 6: Wire DreamcallerDraftScreen**

Replace local `selectOfferedDreamcallers` in `DreamcallerDraftScreen.tsx` with imports:

```ts
import { selectOfferedDreamcallers } from "../data/dreamcaller-offers";
import { computeQuestTideProfile } from "../data/quest-tide-profile";
```

When offers are initialized:

```ts
const profile = computeQuestTideProfile({
  startingTide: state.startingTide,
  deck: state.deck,
  cardDatabase,
  dreamcaller: state.dreamcaller,
  tideCrystals: state.tideCrystals,
  recentDraftPicks: state.draftState?.draftedCards ?? [],
});
offeredRef.current = selectOfferedDreamcallers({
  pool: DREAMCALLERS,
  startingTide: state.startingTide,
  profile,
});
logEvent("dreamcaller_offers_generated", {
  startingTide: state.startingTide,
  offers: offeredRef.current.map((dc) => ({
    name: dc.name,
    tides: dc.tides,
    tideCrystalGrant: dc.tideCrystalGrant,
  })),
});
```

Update rendering to map `dreamcaller.tides` and display two icons.

- [ ] **Step 7: Wire selection logging**

In `quest-context.tsx`, change dreamcaller logging to:

```ts
    logEvent("dreamcaller_selected", {
      name: dreamcaller.name,
      tides: dreamcaller.tides,
      essenceBonus: dreamcaller.essenceBonus,
      tideCrystalGrant: dreamcaller.tideCrystalGrant,
    });
```

In `DreamcallerDraftScreen`, change site-completed fields:

```ts
dreamcallerTides: dreamcaller.tides,
```

- [ ] **Step 8: Update remaining dreamcaller callsites**

Replace `dreamcaller.tide` reads with either `dreamcaller.tides[0]` for a representative accent or `dreamcaller.tides` for data display.

Known callsites from the spec investigation:

```text
scripts/quest_prototype/src/components/DeckViewer.tsx
scripts/quest_prototype/src/screens/BattleScreen.tsx
scripts/quest_prototype/src/screens/QuestCompleteScreen.tsx
scripts/quest_prototype/src/components/HUD.tsx
```

- [ ] **Step 9: Run tests and typecheck**

Run:

```bash
npm test -- src/data/synthetic-data.test.ts src/data/dreamcaller-offers.test.ts
npm run typecheck
```

Expected: PASS.

- [ ] **Step 10: Commit**

```bash
git status --short
git add scripts/quest_prototype/src/types/quest.ts scripts/quest_prototype/src/data/dreamcallers.ts scripts/quest_prototype/src/data/dreamcaller-offers.ts scripts/quest_prototype/src/data/dreamcaller-offers.test.ts scripts/quest_prototype/src/data/synthetic-data.test.ts scripts/quest_prototype/src/screens/DreamcallerDraftScreen.tsx scripts/quest_prototype/src/state/quest-context.tsx scripts/quest_prototype/src/screens/BattleScreen.tsx scripts/quest_prototype/src/screens/QuestCompleteScreen.tsx scripts/quest_prototype/src/components/DeckViewer.tsx scripts/quest_prototype/src/components/HUD.tsx
git commit -m "feat(quest): offer two-tide dreamcaller archetype forks"
```

---

## Task 7: Draft Pool And Profile Bias

**Files:**
- Modify: `scripts/quest_prototype/src/types/draft.ts`
- Modify: `scripts/quest_prototype/src/draft/draft-engine.ts`
- Modify: `scripts/quest_prototype/src/draft/draft-engine.test.ts`
- Modify: `scripts/quest_prototype/src/screens/DraftSiteScreen.tsx`

- [ ] **Step 1: Update draft type context**

In `scripts/quest_prototype/src/types/draft.ts`, add:

```ts
import type { QuestTideProfile } from "../data/quest-tide-profile";
```

Add to `PackContext`:

```ts
  /** Optional adaptive quest profile used as an affinity seed. */
  questTideProfile?: QuestTideProfile;
```

Add to `DraftState`:

```ts
  /** Cards consumed by quest-start random grants. Not part of finite draft pool. */
  consumedStartingCardNumbers: number[];
```

- [ ] **Step 2: Write draft pool tests**

In `draft-engine.test.ts`, add:

```ts
it("excludes Starter cards and consumed starting cards from pool", () => {
  const db = buildDB([
    makeCard(1, "Bloom", "Common"),
    makeCard(2, "Bloom", "Starter"),
    makeCard(3, "Arc", "Common"),
    makeCard(4, "Neutral", "Common"),
  ]);
  const state = initializeDraftState(db, [], false, [1, 4]);
  expect(state.pool).toEqual([3]);
  expect(state.consumedStartingCardNumbers).toEqual([1, 4]);
});
```

Add a profile bias test:

```ts
it("uses questTideProfile to seed early pack affinity", () => {
  const db = buildEvenDB(40);
  const pool = Array.from(db.keys());
  const profile = {
    weights: { Bloom: 30, Arc: 1, Ignite: 1, Pact: 1, Umbra: 1, Rime: 1, Surge: 1, Neutral: 1 },
    contributions: {
      start: { Bloom: 29, Arc: 0, Ignite: 0, Pact: 0, Umbra: 0, Rime: 0, Surge: 0, Neutral: 0 },
      deck: { Bloom: 0, Arc: 0, Ignite: 0, Pact: 0, Umbra: 0, Rime: 0, Surge: 0, Neutral: 0 },
      dreamcaller: { Bloom: 0, Arc: 0, Ignite: 0, Pact: 0, Umbra: 0, Rime: 0, Surge: 0, Neutral: 0 },
      crystals: { Bloom: 0, Arc: 0, Ignite: 0, Pact: 0, Umbra: 0, Rime: 0, Surge: 0, Neutral: 0 },
      recentDraft: { Bloom: 0, Arc: 0, Ignite: 0, Pact: 0, Umbra: 0, Rime: 0, Surge: 0, Neutral: 0 },
      baseline: { Bloom: 1, Arc: 1, Ignite: 1, Pact: 1, Umbra: 1, Rime: 1, Surge: 1, Neutral: 1 },
    },
  };
  const counts: Record<string, number> = {};
  for (let i = 0; i < 300; i++) {
    const pack = generatePack(
      { type: "tide_current" },
      { pool, cardDatabase: db, draftedCards: [], pickNumber: 1, packSize: 4, questTideProfile: profile },
    );
    for (const num of pack) {
      const tide = db.get(num)!.tide;
      counts[tide] = (counts[tide] ?? 0) + 1;
    }
  }
  expect(counts.Bloom).toBeGreaterThan(counts.Pact * 3);
});
```

- [ ] **Step 3: Implement draft pool exclusions**

In `draft-engine.ts`, import:

```ts
import { draftPoolCards } from "../data/card-pools";
import { tideProfileWeight } from "../data/quest-tide-profile";
```

Change initializer signature:

```ts
export function initializeDraftState(
  cardDatabase: Map<number, CardData>,
  excludedTides: Tide[],
  poolBias: boolean = false,
  consumedStartingCardNumbers: number[] = [],
): DraftState {
```

Build pool via helper:

```ts
  const pool = draftPoolCards(cardDatabase, {
    excludedTides,
    consumedCardNumbers: new Set(consumedStartingCardNumbers),
  }).map((card) => card.cardNumber);
```

Return the new field:

```ts
    consumedStartingCardNumbers,
```

- [ ] **Step 4: Seed card weight from profile**

In `computeCardWeight`, change signature:

```ts
function computeCardWeight(
  card: CardData,
  affinity: Map<string, number>,
  focus: number,
  ctx: PackContext,
): number {
  const profileWeight =
    ctx.questTideProfile === undefined ? 1 : tideProfileWeight(ctx.questTideProfile, card.tide);
  const tideAffinity = affinity.get(card.tide) ?? 1.0;
  return profileWeight * Math.pow(tideAffinity, focus);
}
```

Update callers in `tideCurrentPack` and `poolBiasPack` to pass `ctx`.

- [ ] **Step 5: Pass current profile from DraftSiteScreen**

In `DraftSiteScreen.tsx`, compute profile before initializer and `enterDraftSite`:

```ts
const profile = computeQuestTideProfile({
  startingTide: state.startingTide,
  deck: state.deck,
  cardDatabase,
  dreamcaller: state.dreamcaller,
  tideCrystals: state.tideCrystals,
  recentDraftPicks: state.draftState?.draftedCards ?? [],
});
```

Initialize draft state with consumed cards:

```ts
ds = initializeDraftState(
  cardDatabase,
  state.excludedTides,
  questConfig.poolBias,
  state.consumedStartingCardNumbers,
);
```

When entering site and generating subsequent packs, make sure `questTideProfile: profile` reaches `generatePack`. If the simplest edit is changing `enterDraftSite` signature, use:

```ts
enterDraftSite(cloned, cardDatabase, { packSize: 4 }, profile);
```

and pass that optional profile through `enterDraftSite` into `generatePack`.

- [ ] **Step 6: Run draft tests**

Run:

```bash
npm test -- src/draft/draft-engine.test.ts
npm run typecheck
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git status --short
git add scripts/quest_prototype/src/types/draft.ts scripts/quest_prototype/src/draft/draft-engine.ts scripts/quest_prototype/src/draft/draft-engine.test.ts scripts/quest_prototype/src/screens/DraftSiteScreen.tsx
git commit -m "feat(quest): seed draft packs from quest tide profile"
```

---

## Task 8: Shops, Rewards, Dreamsigns, And Deferred Reward Sites

**Files:**
- Modify: `scripts/quest_prototype/src/shop/shop-generator.ts`
- Modify: `scripts/quest_prototype/src/shop/shop-generator.test.ts`
- Modify: `scripts/quest_prototype/src/screens/ShopScreen.tsx`
- Modify: `scripts/quest_prototype/src/screens/SpecialtyShopScreen.tsx`
- Modify: `scripts/quest_prototype/src/data/tide-weights.ts`
- Modify: `scripts/quest_prototype/src/screens/BattleScreen.tsx`
- Modify: `scripts/quest_prototype/src/atlas/atlas-generator.ts`
- Modify: `scripts/quest_prototype/src/screens/RewardSiteScreen.tsx`

- [ ] **Step 1: Change shop generator signatures**

In `shop-generator.ts`, import and accept profile:

```ts
import type { QuestTideProfile } from "../data/quest-tide-profile";
import { weightedSampleByProfile } from "../data/quest-tide-profile";
import { offerableCards } from "../data/card-pools";
```

Change signatures:

```ts
export function generateShopInventory(
  cardDatabase: Map<number, CardData>,
  playerDeck: DeckEntry[],
  excludedTides: Tide[] = [],
  profile: QuestTideProfile | null = null,
): ShopSlot[] {
```

```ts
export function generateSpecialtyShopInventory(
  cardDatabase: Map<number, CardData>,
  playerDeck: DeckEntry[],
  excludedTides: Tide[] = [],
  profile: QuestTideProfile | null = null,
): ShopSlot[] {
```

- [ ] **Step 2: Exclude Starter in shop tests**

Add to `shop-generator.test.ts`:

```ts
it("never offers Starter cards in regular or specialty shops", () => {
  const db = makeDatabase([
    makeCard({ cardNumber: 1, rarity: "Starter", tide: "Bloom" }),
    makeCard({ cardNumber: 2, rarity: "Rare", tide: "Bloom" }),
    makeCard({ cardNumber: 3, rarity: "Common", tide: "Bloom" }),
  ]);
  for (let i = 0; i < 50; i++) {
    const regular = generateShopInventory(db, []);
    const specialty = generateSpecialtyShopInventory(db, []);
    const offered = [...regular, ...specialty].flatMap((slot) =>
      slot.card === null ? [] : [slot.card],
    );
    expect(offered.every((card) => card.rarity !== "Starter")).toBe(true);
  }
});
```

- [ ] **Step 3: Use offerableCards and profile-weighted selection**

In `generateShopInventory`, replace all-card filter with:

```ts
  const allCards = offerableCards(cardDatabase, { excludedTides });
```

When selecting a card:

```ts
const card =
  profile === null
    ? selectWeightedCard(allCards, deckTideCounts)
    : weightedSampleByProfile(allCards, profile, 1)[0] ?? null;
```

In specialty shop:

```ts
const rareCards = offerableCards(cardDatabase, { excludedTides }).filter(
  (c) => c.rarity === "Rare",
);
```

and the same profile fallback selection pattern.

- [ ] **Step 4: Pass profile from shop screens**

In `ShopScreen.tsx` and `SpecialtyShopScreen.tsx`, compute profile with current `state` and pass it as the fourth argument.

Use this exact profile input:

```ts
const profile = computeQuestTideProfile({
  startingTide: state.startingTide,
  deck: state.deck,
  cardDatabase,
  dreamcaller: state.dreamcaller,
  tideCrystals: state.tideCrystals,
  recentDraftPicks: state.draftState?.draftedCards ?? [],
});
```

- [ ] **Step 5: Keep battle rare rewards profile-aware and Starter-free**

Either update `selectRareRewards` in `tide-weights.ts` to accept an optional `QuestTideProfile`, or create a replacement in `quest-tide-profile.ts`.

The card filter must be:

```ts
const rareCards = offerableCards(cardDatabase, { excludedTides }).filter(
  (card) => card.rarity === "Rare",
);
```

In `BattleScreen.tsx`, pass the computed current profile when generating `rareCardsRef.current`.

- [ ] **Step 6: Defer reward-site concrete reward**

In `atlas-generator.ts`, for Reward sites, keep `rewardType` if needed but stop writing concrete `cardNumber`, `dreamsignName`, `dreamsignTide`, and `dreamsignEffect` into site data.

In `RewardSiteScreen.tsx`, roll missing card/dreamsign rewards on first render using current profile:

```ts
const [rolledReward, setRolledReward] = useState<Record<string, unknown> | null>(null);
```

When `rewardType === "card"`, sample one offerable/profile-weighted card and set `cardNumber`.

When `rewardType === "dreamsign"`, sample one dreamsign with profile weighting and set the existing `dreamsignName`, `dreamsignTide`, and `dreamsignEffect` fields.

Log:

```ts
logEvent("reward_generated", {
  siteId: site.id,
  rewardType,
  rewardData: rolled,
});
```

- [ ] **Step 7: Run focused tests**

Run:

```bash
npm test -- src/shop/shop-generator.test.ts src/data/tide-weights.test.ts src/atlas/atlas-generator.test.ts
npm run typecheck
```

If `src/data/tide-weights.test.ts` does not exist, omit it from the command.

Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git status --short
git add scripts/quest_prototype/src/shop/shop-generator.ts scripts/quest_prototype/src/shop/shop-generator.test.ts scripts/quest_prototype/src/screens/ShopScreen.tsx scripts/quest_prototype/src/screens/SpecialtyShopScreen.tsx scripts/quest_prototype/src/data/tide-weights.ts scripts/quest_prototype/src/screens/BattleScreen.tsx scripts/quest_prototype/src/atlas/atlas-generator.ts scripts/quest_prototype/src/screens/RewardSiteScreen.tsx
git commit -m "feat(quest): weight shops and rewards from quest tide profile"
```

---

## Task 9: HUD And Deck Viewer Origin Visibility

**Files:**
- Modify: `scripts/quest_prototype/src/components/HUD.tsx`
- Modify: `scripts/quest_prototype/src/components/DeckViewer.tsx`

- [ ] **Step 1: Add compact origin badge to HUD**

In `HUD.tsx`, compute:

```ts
  const startingTide = state.startingTide;
  const startingTideColor =
    startingTide !== null ? TIDE_COLORS[startingTide] : "#6b7280";
```

Add a left-cluster block after essence or deck size:

```tsx
{startingTide !== null && (
  <div className="flex items-center gap-1.5" title={`Starting Tide: ${startingTide}`}>
    <span
      className="rounded-full px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider"
      style={{
        background: `${startingTideColor}20`,
        border: `1px solid ${startingTideColor}60`,
        color: startingTideColor,
      }}
    >
      {startingTide}
    </span>
  </div>
)}
```

- [ ] **Step 2: Add Quest Origin section to DeckViewer sidebar**

Above the Dreamcaller section in `DeckViewer.tsx`, add:

```tsx
{state.startingTide !== null && (
  <div className="mb-4">
    <h3
      className="mb-2 text-xs font-bold uppercase tracking-wider"
      style={{ color: "#a855f7" }}
    >
      Quest Origin
    </h3>
    <div
      className="flex items-center gap-2 rounded-lg p-3"
      style={{
        background: `${TIDE_COLORS[state.startingTide]}10`,
        border: `1px solid ${TIDE_COLORS[state.startingTide]}35`,
      }}
    >
      <img
        src={tideIconUrl(state.startingTide)}
        alt={state.startingTide}
        className="h-6 w-6 rounded-full"
        style={{ border: `1px solid ${TIDE_COLORS[state.startingTide]}` }}
      />
      <div className="flex min-w-0 flex-col">
        <span
          className="text-sm font-bold"
          style={{ color: TIDE_COLORS[state.startingTide] }}
        >
          {state.startingTide}
        </span>
        <span className="text-[10px] opacity-50">
          Starting tide and first crystal
        </span>
      </div>
    </div>
  </div>
)}
```

- [ ] **Step 3: Run typecheck**

Run:

```bash
npm run typecheck
```

Expected: PASS.

- [ ] **Step 4: Browser smoke check HUD and deck viewer**

Run app, select any starting tide, open deck viewer.

Commands:

```bash
/Users/dthurn/Library/pnpm/agent-browser screenshot /tmp/quest-starting-tide-qa/hud-origin.png
/Users/dthurn/Library/pnpm/agent-browser click "text=View Deck"
/Users/dthurn/Library/pnpm/agent-browser screenshot /tmp/quest-starting-tide-qa/deck-viewer-origin.png
```

Expected: HUD shows selected origin; deck viewer shows 30 cards and Quest Origin.

- [ ] **Step 5: Commit**

```bash
git status --short
git add scripts/quest_prototype/src/components/HUD.tsx scripts/quest_prototype/src/components/DeckViewer.tsx
git commit -m "feat(quest): show quest origin in HUD and deck viewer"
```

---

## Task 10: Structured Generation Logging And Log Analyzer

**Files:**
- Modify: `scripts/quest_prototype/src/data/quest-tide-profile.ts`
- Modify: `scripts/quest_prototype/src/screens/ShopScreen.tsx`
- Modify: `scripts/quest_prototype/src/screens/SpecialtyShopScreen.tsx`
- Modify: `scripts/quest_prototype/src/draft/draft-engine.ts`
- Create: `scripts/quest_prototype/src/qa/analyze-quest-log.mjs`

- [ ] **Step 1: Add a profile log helper**

In `quest-tide-profile.ts`, export:

```ts
/** JSON-safe summary for quest_tide_profile_computed log events. */
export function questTideProfileLogFields(profile: QuestTideProfile): Record<string, unknown> {
  return {
    weights: profile.weights,
    contributions: profile.contributions,
    orderedNamedTides: [...NAMED_TIDES]
      .sort((a, b) => profile.weights[b] - profile.weights[a])
      .map((tide) => ({ tide, weight: profile.weights[tide] })),
  };
}
```

- [ ] **Step 2: Log profile after computing it at generation surfaces**

When a screen computes a profile for generation, log once per generated offer:

```ts
logEvent("quest_tide_profile_computed", {
  source: "shop",
  startingTide: state.startingTide,
  ...questTideProfileLogFields(profile),
});
```

Use distinct source strings:

```text
draft_enter
dreamcaller_draft
shop
specialty_shop
battle_reward
reward_site
```

- [ ] **Step 3: Log shop inventory**

After shop inventory is generated:

```ts
logEvent("shop_inventory_generated", {
  siteType: "Shop",
  slots: inventory.map((slot) => ({
    itemType: slot.itemType,
    cardNumber: slot.card?.cardNumber ?? null,
    cardName: slot.card?.name ?? null,
    cardTide: slot.card?.tide ?? null,
    cardRarity: slot.card?.rarity ?? null,
    dreamsignName: slot.dreamsign?.name ?? null,
    dreamsignTide: slot.dreamsign?.tide ?? null,
    tideCrystal: slot.tideCrystal,
    basePrice: slot.basePrice,
    discountPercent: slot.discountPercent,
  })),
});
```

Use `siteType: "SpecialtyShop"` in SpecialtyShopScreen.

- [ ] **Step 4: Create log analyzer**

Create `scripts/quest_prototype/src/qa/analyze-quest-log.mjs`:

```js
import { readFileSync } from "node:fs";

const path = process.argv[2];

if (!path) {
  throw new Error("Usage: node src/qa/analyze-quest-log.mjs /path/to/quest-log.jsonl");
}

const events = readFileSync(path, "utf8")
  .trim()
  .split("\n")
  .filter(Boolean)
  .map((line) => JSON.parse(line));

function requireEvent(name) {
  const event = events.find((entry) => entry.event === name);
  if (!event) throw new Error(`Missing event: ${name}`);
  return event;
}

const selected = requireEvent("starting_tide_selected");
const deck = requireEvent("starting_deck_initialized");
const quest = requireEvent("quest_started");

if (deck.totalDeckSize !== 30) {
  throw new Error(`Expected starting deck size 30; found ${deck.totalDeckSize}`);
}

for (const key of ["starterCardNumbers", "tideCardNumbers", "neutralCardNumbers"]) {
  if (!Array.isArray(deck[key]) || deck[key].length !== 10) {
    throw new Error(`Expected ${key} to have 10 card numbers`);
  }
}

const summary = {
  selectedStartingTide: selected.startingTide,
  grantedCrystal: selected.grantedCrystal,
  questInitialDeckSize: quest.initialDeckSize,
  startingDeckTotal: deck.totalDeckSize,
  dreamcallerOffers: events
    .filter((entry) => entry.event === "dreamcaller_offers_generated")
    .flatMap((entry) => entry.offers ?? []),
  shopInventoryEvents: events.filter((entry) => entry.event === "shop_inventory_generated").length,
  rewardEvents: events.filter((entry) => entry.event === "reward_generated").length,
  finalCardAddEvents: events.filter((entry) => entry.event === "card_added").length,
};

console.log(JSON.stringify(summary, null, 2));
```

- [ ] **Step 5: Run typecheck and tests**

Run:

```bash
npm test
npm run typecheck
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git status --short
git add scripts/quest_prototype/src/data/quest-tide-profile.ts scripts/quest_prototype/src/screens/ShopScreen.tsx scripts/quest_prototype/src/screens/SpecialtyShopScreen.tsx scripts/quest_prototype/src/draft/draft-engine.ts scripts/quest_prototype/src/qa/analyze-quest-log.mjs
git commit -m "feat(quest): log tide-profile generation decisions"
```

---

## Task 11: Full Web Verification And Manual QA

**Files:**
- No production code edits expected
- Write QA notes outside the repo under `/tmp/quest-starting-tide-qa/`

- [ ] **Step 1: Run automated gates**

Run:

```bash
npm run setup-assets
npm test
npm run typecheck
npm run lint
npm run build
```

Expected: all commands exit 0.

- [ ] **Step 2: Start app in foreground in a dedicated terminal**

Run:

```bash
npm run dev
```

Expected: Vite prints a local URL, normally `http://localhost:5173/`.

- [ ] **Step 3: Browser QA quest start**

Run:

```bash
mkdir -p /tmp/quest-starting-tide-qa
/Users/dthurn/Library/pnpm/agent-browser open http://localhost:5173
/Users/dthurn/Library/pnpm/agent-browser wait --load networkidle
/Users/dthurn/Library/pnpm/agent-browser screenshot /tmp/quest-starting-tide-qa/01-start-options.png
/Users/dthurn/Library/pnpm/agent-browser errors
/Users/dthurn/Library/pnpm/agent-browser console
```

Expected: 3 named tide options, no browser page errors.

- [ ] **Step 4: Browser QA selected start and deck viewer**

Before clicking, record expectation: after selecting a tide, deck size = 30, starting crystal = 1 for selected tide, HUD origin = selected tide.

Run:

```bash
/Users/dthurn/Library/pnpm/agent-browser snapshot -i
```

Click one starting tide button by `@ref`, then:

```bash
/Users/dthurn/Library/pnpm/agent-browser screenshot /tmp/quest-starting-tide-qa/02-atlas-hud.png
/Users/dthurn/Library/pnpm/agent-browser click "text=View Deck"
/Users/dthurn/Library/pnpm/agent-browser screenshot /tmp/quest-starting-tide-qa/03-deck-viewer.png
```

Expected: deck viewer reports 30 cards, shows Quest Origin, shows 10 Starter cards somewhere in the deck grid.

- [ ] **Step 5: Browser QA draft and dreamcaller**

Navigate to available dreamscape and visit Draft and Dreamcaller Draft sites. Use `snapshot -i` to find current clickable refs and screenshot after each action:

```bash
/Users/dthurn/Library/pnpm/agent-browser screenshot /tmp/quest-starting-tide-qa/04-draft-pick-1.png
/Users/dthurn/Library/pnpm/agent-browser screenshot /tmp/quest-starting-tide-qa/05-dreamcaller-offers.png
```

Expected: draft pick flow increases deck size by 5 after completion; dreamcaller cards show two tide icons and at least one offer includes the starting tide.

- [ ] **Step 6: Browser QA shop/reward/battle paths**

Play through enough sites to exercise at least one shop-like screen, one reward
site when one appears on the selected route, and one battle victory reward.
Screenshot each generated offer:

```bash
/Users/dthurn/Library/pnpm/agent-browser screenshot /tmp/quest-starting-tide-qa/06-shop-or-reward.png
/Users/dthurn/Library/pnpm/agent-browser screenshot /tmp/quest-starting-tide-qa/07-battle-reward.png
```

Expected: no visible Starter cards in generated offers; purchases/rewards change deck and essence by visible amounts.

- [ ] **Step 7: Download and analyze log**

Click Download Log in the HUD. Then find the newest file:

```bash
ls -t ~/Downloads/quest-log-*.jsonl | head -1
node src/qa/analyze-quest-log.mjs "$(ls -t ~/Downloads/quest-log-*.jsonl | head -1)"
```

Expected: analyzer prints JSON summary and exits 0. Summary must include selectedStartingTide and startingDeckTotal 30.

- [ ] **Step 8: Reset/reload QA**

Run:

```bash
/Users/dthurn/Library/pnpm/agent-browser open http://localhost:5173
/Users/dthurn/Library/pnpm/agent-browser wait --load networkidle
/Users/dthurn/Library/pnpm/agent-browser screenshot /tmp/quest-starting-tide-qa/08-reloaded-start.png
```

Expected: the new session is back at starting tide selection; previous deck/origin is not visible.

- [ ] **Step 9: Final status check**

From repo root:

```bash
git status --short
git log --oneline -12
```

Expected: only intentional files are changed. If all implementation tasks committed, status is empty.

---

## Plan Self-Review Checklist

Before implementing, verify these mappings from spec to plan:

- Starting tide options: Task 3 pure helper and Task 4 UI.
- 30-card starting deck: Task 3 helper and Task 4 state mutation.
- Starter loadout-only data: Task 1 data load, Task 2 pool helper, Task 7 draft pool, Task 8 shop/reward pools.
- Consumed random grants: Task 4 state, Task 7 draft initialization.
- No default excluded tides: Task 4 config step.
- HUD/deck viewer origin: Task 9.
- Two-tide dreamcallers: Task 6.
- Shared quest tide profile: Task 5 and Task 7/8/10 wiring.
- Deferred reward-site rolls: Task 8.
- Structured logs: Task 4, Task 6, Task 8, Task 10.
- Web-only gates: Execution Rules and Task 11.
- Browser QA and log analysis: Task 11.
