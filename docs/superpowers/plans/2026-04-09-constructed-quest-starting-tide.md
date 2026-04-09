# Constructed Quest Starting Tide Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redesign `scripts/constructed_quest_prototype` so a quest starts by choosing a named tide, builds a 30-card starting deck (10 Starter + 10 tide + 10 Neutral), converts dreamcallers to two-tide pairs, excludes Starter cards from all random offers, and mixes neutral cards into tide packs.

**Architecture:** Modify the existing quest start flow, dreamcaller data, and card selection helpers in place. No new modules — adapt existing files. Change `QuestState.startingTides: Tide[]` to `QuestState.startingTide: NamedTide | null`. Update all downstream consumers (shops, packs, forge, atlas, battle, HUD, deck viewer) to derive tide seeding from the single starting tide + its circle neighbors.

**Tech Stack:** React 19, Vite 7, TypeScript 5, Vitest, ESLint, agent-browser manual QA

**Design spec:** `docs/superpowers/specs/2026-04-09-constructed-quest-starting-tide-design.md`

---

## Execution Rules

Run commands from `scripts/constructed_quest_prototype` unless a step explicitly says otherwise.

Primary verification commands:

```bash
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

> Use the `qa` skill for browser verification and use `agent-browser` to take screenshots. Before each click, state the expected deck size, screen, visible starting tide, essence, crystals, and selected-card effect. After each click, take a screenshot and compare actual visible values. Keep screenshots under `/tmp/constructed-quest-starting-tide-qa/`. Do not claim completion if deck size, crystal count, dreamcaller tide icons, shop item state, or pack contents differ from expected behavior.

Copy this block into implementation-worker prompts for logging / run-analysis tasks:

> Use the `qs-analyze` skill after downloading a quest log. Analyze JSONL events; do not infer missing data. If the log cannot answer starting options, selected tide, starter/tide/neutral card groups, dreamcaller offer pairs, shop offer tides, or final deck size, add explicit logging and retest.

## File Map

**Modify:**

| File | Responsibility |
|------|----------------|
| `src/types/cards.ts` | Add `Starter` to Rarity, add `NamedTide` type alias |
| `src/types/quest.ts` | Change `startingTides: Tide[]` to `startingTide: NamedTide \| null`, convert Dreamcaller to two-tide |
| `src/data/card-database.ts` | Add Starter to `RARITY_COLORS` |
| `src/data/dreamcallers.ts` | Convert all 10 dreamcallers to two-tide neighbor pairs |
| `src/state/quest-context.tsx` | Replace `setStartingTides` with `setStartingTide`, update `setDreamcaller` logging |
| `src/state/quest-config.ts` | Remove `startingTides` and `sequentialTides` config params |
| `src/screens/QuestStartScreen.tsx` | Show 3 tide choices, build 30-card deck on selection |
| `src/screens/DreamcallerDraftScreen.tsx` | Two-tide offer logic with left-fork/right-fork/adaptive |
| `src/screens/ShopScreen.tsx` | Pass `startingTide` + neighbors instead of `startingTides` |
| `src/screens/PackShopScreen.tsx` | Pass `startingTide` + neighbors instead of `startingTides` |
| `src/screens/BattleScreen.tsx` | Update atlas generation context, update enemy generation for two-tide |
| `src/screens/ForgeScreen.tsx` | Derive eligible tides from `startingTide` + neighbors |
| `src/shop/shop-generator.ts` | Accept `startingTide` + neighbors, exclude Starter rarity |
| `src/shop/pack-shop-generator.ts` | Accept `startingTide` + neighbors, exclude Starter rarity |
| `src/pack/pack-generator.ts` | Exclude Starter rarity, add 0-1 neutral cards to tide packs |
| `src/atlas/atlas-generator.ts` | Change `SiteGenerationContext.startingTides` to derive from single tide |
| `src/forge/forge-logic.ts` | Derive eligible tides from `startingTide` + neighbors, exclude Starter |
| `src/data/tide-weights.ts` | Exclude Starter in `selectRareRewards` |
| `src/components/DeckViewer.tsx` | Add Quest Origin display, update dreamcaller to show two tides |
| `src/components/HUD.tsx` | Update dreamcaller display for two-tide |
| `src/data/synthetic-data.test.ts` | Update dreamcaller tests for two-tide shape |
| `src/data/card-database.test.ts` | Update RARITY_COLORS test for Starter |
| `src/state/quest-state-machine.test.ts` | Update `createTestState` for new field shape |
| `src/atlas/atlas-generator.test.ts` | Update context mock for new field shape |

---

## Task 1: Type Changes And Rarity

**Files:**
- Modify: `src/types/cards.ts`
- Modify: `src/types/quest.ts`
- Modify: `src/data/card-database.ts`
- Modify: `src/data/synthetic-data.test.ts`
- Modify: `src/data/card-database.test.ts`
- Modify: `src/state/quest-state-machine.test.ts`

- [ ] **Step 1: Add Starter to Rarity and add NamedTide**

In `src/types/cards.ts`, replace lines 2 and 13:

```ts
/** The 7 non-neutral tides on the tide circle. */
export type NamedTide = Exclude<Tide, "Neutral">;

/** Card rarity levels (Special excluded from the draft pool). */
export type Rarity = "Common" | "Uncommon" | "Rare" | "Legendary" | "Starter";
```

- [ ] **Step 2: Add Starter to RARITY_COLORS**

In `src/data/card-database.ts`, replace the `RARITY_COLORS` constant (lines 60-65):

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

- [ ] **Step 3: Change QuestState and Dreamcaller types**

In `src/types/quest.ts`, add the import for `NamedTide`:

```ts
import type { CardData, NamedTide, Tide } from "./cards";
```

Replace the `Dreamcaller` interface (lines 77-83):

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

Replace `startingTides: Tide[];` in `QuestState` (line 140):

```ts
  startingTide: NamedTide | null;
```

- [ ] **Step 4: Update synthetic-data.test.ts**

In `src/data/synthetic-data.test.ts`, update the ALL_RARITIES constant (line 22):

```ts
const ALL_RARITIES: Rarity[] = ["Common", "Uncommon", "Rare", "Legendary", "Starter"];
```

Replace the dreamcaller tests (lines 72-98):

```ts
describe("dreamcallers", () => {
  it("has exactly 10 entries", () => {
    expect(DREAMCALLERS).toHaveLength(10);
  });

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

  it("covers every named tide at least once", () => {
    const tides = new Set(DREAMCALLERS.flatMap((dc) => dc.tides));
    for (const t of NAMED_TIDES) {
      expect(tides.has(t)).toBe(true);
    }
  });

  it("covers all 7 neighbor pairs", () => {
    const TIDE_CIRCLE: NamedTide[] = ["Bloom", "Arc", "Ignite", "Pact", "Umbra", "Rime", "Surge"];
    const pairs = new Set<string>();
    for (const dc of DREAMCALLERS) {
      const sorted = [...dc.tides].sort((a, b) => TIDE_CIRCLE.indexOf(a) - TIDE_CIRCLE.indexOf(b));
      pairs.add(`${sorted[0]}/${sorted[1]}`);
    }
    for (let i = 0; i < 7; i++) {
      const a = TIDE_CIRCLE[i];
      const b = TIDE_CIRCLE[(i + 1) % 7];
      const sorted = [a, b].sort((x, y) => TIDE_CIRCLE.indexOf(x) - TIDE_CIRCLE.indexOf(y));
      expect(pairs.has(`${sorted[0]}/${sorted[1]}`)).toBe(true);
    }
  });

  it("has unique names", () => {
    const names = DREAMCALLERS.map((dc) => dc.name);
    expect(new Set(names).size).toBe(names.length);
  });
});
```

Add the `NamedTide` import at the top:

```ts
import type { Tide, Rarity, NamedTide } from "../types/cards";
```

- [ ] **Step 5: Update card-database.test.ts**

In `src/data/card-database.test.ts`, find the RARITY_COLORS test and add Starter:

```ts
Starter: "#d4a017",
```

Also find the `validRarities` set used in integration tests and add `"Starter"`.

- [ ] **Step 6: Update quest-state-machine.test.ts**

In `src/state/quest-state-machine.test.ts`, replace `startingTides: []` with `startingTide: null` in the `createTestState` helper (line 30).

- [ ] **Step 7: Run typecheck to see remaining errors**

Run:

```bash
npm run typecheck 2>&1 | head -80
```

Expected: Type errors in files not yet updated (quest-context, screens, generators). The files modified in this task should be internally consistent. Collect the error list for subsequent tasks.

- [ ] **Step 8: Commit**

```bash
git add scripts/constructed_quest_prototype/src/types/cards.ts scripts/constructed_quest_prototype/src/types/quest.ts scripts/constructed_quest_prototype/src/data/card-database.ts scripts/constructed_quest_prototype/src/data/synthetic-data.test.ts scripts/constructed_quest_prototype/src/data/card-database.test.ts scripts/constructed_quest_prototype/src/state/quest-state-machine.test.ts
git commit -m "feat(quest): add Starter rarity, NamedTide type, two-tide Dreamcaller"
```

---

## Task 2: Dreamcaller Data

**Files:**
- Modify: `src/data/dreamcallers.ts`

- [ ] **Step 1: Convert all 10 dreamcallers to two-tide neighbor pairs**

Replace the entire `DREAMCALLERS` array in `src/data/dreamcallers.ts`:

```ts
import type { Dreamcaller } from "../types/quest";

/** The 10 available dreamcallers, each representing a neighbor-pair archetype. */
export const DREAMCALLERS: readonly Dreamcaller[] = [
  {
    name: "Lyria, Tide Weaver",
    tides: ["Bloom", "Arc"],
    abilityDescription:
      "She draws vitality from the roots of dreaming trees, mending wounds with whispered verse.",
    essenceBonus: 80,
    tideCrystalGrant: "Bloom",
  },
  {
    name: "Kael of the Ashen Veil",
    tides: ["Arc", "Ignite"],
    abilityDescription:
      "Lightning traces his every step. He reads the storm-patterns that pulse between realms.",
    essenceBonus: 100,
    tideCrystalGrant: "Arc",
  },
  {
    name: "Serath, the Cindermaw",
    tides: ["Ignite", "Pact"],
    abilityDescription:
      "Flame dances in her eyes and boils the air around her. She burns away all that is false.",
    essenceBonus: 60,
    tideCrystalGrant: "Ignite",
  },
  {
    name: "Mireille Duskpact",
    tides: ["Pact", "Umbra"],
    abilityDescription:
      "Her bargains are sealed in blood and starlight. Every alliance she forges bends fate itself.",
    essenceBonus: 120,
    tideCrystalGrant: "Pact",
  },
  {
    name: "Thalvor the Hollow King",
    tides: ["Umbra", "Rime"],
    abilityDescription:
      "He wears a crown of shadows and speaks to the nothing between worlds. Even light obeys him.",
    essenceBonus: 70,
    tideCrystalGrant: "Umbra",
  },
  {
    name: "Isolde Frostborne",
    tides: ["Rime", "Surge"],
    abilityDescription:
      "Ice crystallizes in her wake, preserving memories frozen in perfect clarity.",
    essenceBonus: 90,
    tideCrystalGrant: "Rime",
  },
  {
    name: "Nyvex, Depth Strider",
    tides: ["Surge", "Bloom"],
    abilityDescription:
      "He walks the crushing deep where drowned gods slumber, carrying their forgotten songs.",
    essenceBonus: 110,
    tideCrystalGrant: "Surge",
  },
  {
    name: "Eryndra Wildsong",
    tides: ["Bloom", "Surge"],
    abilityDescription:
      "She belongs to no tide and all tides. The raw chaos of the dreamscape answers her call.",
    essenceBonus: 150,
    tideCrystalGrant: "Bloom",
  },
  {
    name: "Vaelith, Ember Augur",
    tides: ["Ignite", "Arc"],
    abilityDescription:
      "He reads the future in ashes and cinder. Every flame he kindles reveals a hidden truth.",
    essenceBonus: 50,
    tideCrystalGrant: "Ignite",
  },
  {
    name: "Orivane, the Pale Witness",
    tides: ["Pact", "Rime"],
    abilityDescription:
      "She has seen the end of all dreams and returned unchanged. Her gaze unravels deception.",
    essenceBonus: 130,
    tideCrystalGrant: "Pact",
  },
] as const;
```

This covers all 7 neighbor pairs: Bloom/Arc, Arc/Ignite, Ignite/Pact, Pact/Umbra, Umbra/Rime, Rime/Surge, Surge/Bloom. Three pairs get a second caller (Bloom/Surge via Eryndra, Arc/Ignite via Vaelith, Pact/Rime via Orivane).

- [ ] **Step 2: Run the synthetic data test**

Run:

```bash
npm test -- src/data/synthetic-data.test.ts
```

Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add scripts/constructed_quest_prototype/src/data/dreamcallers.ts
git commit -m "feat(quest): convert dreamcallers to two-tide neighbor pairs"
```

---

## Task 3: Quest State Context

**Files:**
- Modify: `src/state/quest-context.tsx`
- Modify: `src/state/quest-config.ts`

- [ ] **Step 1: Update quest-config.ts**

In `src/state/quest-config.ts`, remove these fields from `QuestConfig`:

- `startingTides: number;` (line 8)
- `sequentialTides: boolean;` (line 10)

Remove these lines from `getQuestConfig()`:

- `startingTides: parseIntParam(params, "startingTides", 3, 1, 8),` (line 120)
- `sequentialTides: parseBoolParam(params, "sequentialTides", true),` (line 121)

- [ ] **Step 2: Update quest-context.tsx imports and types**

In `src/state/quest-context.tsx`, update the import (line 10):

```ts
import type { CardData, NamedTide, Tide } from "../types/cards";
```

Replace `setStartingTides` in the `QuestMutations` interface (line 52):

```ts
  setStartingTide: (tide: NamedTide) => void;
```

- [ ] **Step 3: Update createDefaultState**

Replace `startingTides: []` (line 99):

```ts
    startingTide: null,
```

- [ ] **Step 4: Replace setStartingTides mutation**

Replace the `setStartingTides` callback (lines 428-431):

```ts
  const setStartingTide = useCallback((tide: NamedTide) => {
    logEvent("starting_tide_set", { tide });
    setState((prev) => ({ ...prev, startingTide: tide }));
  }, []);
```

- [ ] **Step 5: Update setDreamcaller logging**

Replace the `setDreamcaller` callback (lines 303-310):

```ts
  const setDreamcaller = useCallback((dreamcaller: Dreamcaller) => {
    logEvent("dreamcaller_selected", {
      name: dreamcaller.name,
      tides: dreamcaller.tides,
      essenceBonus: dreamcaller.essenceBonus,
    });
    setState((prev) => ({ ...prev, dreamcaller }));
  }, []);
```

- [ ] **Step 6: Update mutations object**

In the `mutations` useMemo (line 501+), replace `setStartingTides` with `setStartingTide` in both the object literal and the dependency array.

- [ ] **Step 7: Commit**

```bash
git add scripts/constructed_quest_prototype/src/state/quest-context.tsx scripts/constructed_quest_prototype/src/state/quest-config.ts
git commit -m "feat(quest): replace startingTides array with single startingTide"
```

---

## Task 4: Quest Start Screen

**Files:**
- Modify: `src/screens/QuestStartScreen.tsx`

- [ ] **Step 1: Rewrite QuestStartScreen**

Replace the entire content of `src/screens/QuestStartScreen.tsx`:

```tsx
import { useCallback, useRef } from "react";
import { motion } from "framer-motion";
import { useQuest } from "../state/quest-context";
import { generateInitialAtlas } from "../atlas/atlas-generator";
import { NAMED_TIDES, adjacentTides, TIDE_COLORS, tideIconUrl } from "../data/card-database";
import { DREAMSIGNS } from "../data/dreamsigns";
import { weightedSample } from "../data/tide-weights";
import { logEvent } from "../logging";
import { useQuestConfig } from "../state/quest-config";
import type { NamedTide, CardData } from "../types/cards";

/** Shuffles an array using Fisher-Yates. Returns a new array. */
function shuffled<T>(items: readonly T[]): T[] {
  const result = [...items];
  for (let i = result.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [result[i], result[j]] = [result[j], result[i]];
  }
  return result;
}

/** Generates 3 distinct named tide options for quest start. */
function generateStartingTideOptions(): NamedTide[] {
  return shuffled(NAMED_TIDES).slice(0, 3) as NamedTide[];
}

/** Builds the 30-card starting deck for a chosen tide. */
function buildStartingDeck(
  cardDatabase: Map<number, CardData>,
  startingTide: NamedTide,
): { starterCards: CardData[]; tideCards: CardData[]; neutralCards: CardData[] } {
  const allCards = Array.from(cardDatabase.values());

  // 10 Starter cards (fixed loadout)
  const starterCards = allCards.filter((c) => c.rarity === "Starter");

  // 10 random cards from starting tide (excluding Starter, Legendary)
  const tideCandidates = allCards.filter(
    (c) =>
      c.tide === startingTide &&
      c.rarity !== "Starter" &&
      c.rarity !== "Legendary",
  );
  const tideCards = weightedSample(tideCandidates, 10, () => 1);

  // 10 random Neutral cards (excluding Starter, Legendary)
  const neutralCandidates = allCards.filter(
    (c) =>
      c.tide === "Neutral" &&
      c.rarity !== "Starter" &&
      c.rarity !== "Legendary",
  );
  const neutralCards = weightedSample(neutralCandidates, 10, () => 1);

  return { starterCards, tideCards, neutralCards };
}

/** Starting tide selection screen. */
export function QuestStartScreen() {
  const { state, mutations, cardDatabase } = useQuest();
  const config = useQuestConfig();

  // Generate stable tide options once
  const optionsRef = useRef<NamedTide[] | null>(null);
  if (optionsRef.current === null) {
    optionsRef.current = generateStartingTideOptions();
    logEvent("starting_tide_options_generated", {
      options: optionsRef.current,
    });
  }
  const options = optionsRef.current;

  const handleChooseTide = useCallback(
    (startingTide: NamedTide) => {
      // Set starting tide and grant crystal
      mutations.setStartingTide(startingTide);
      mutations.addTideCrystal(startingTide, 1);

      // Build starting deck
      const { starterCards, tideCards, neutralCards } = buildStartingDeck(
        cardDatabase,
        startingTide,
      );

      logEvent("starting_deck_initialized", {
        startingTide,
        starterCount: starterCards.length,
        tideCount: tideCards.length,
        neutralCount: neutralCards.length,
        totalDeckSize:
          starterCards.length + tideCards.length + neutralCards.length,
      });

      // Add all cards to pool
      const allCards = [...starterCards, ...tideCards, ...neutralCards];
      for (const card of allCards) {
        mutations.addToPool(card.cardNumber, "starter");
      }

      // Initialize deck from pool
      mutations.initializeDeckFromPool();

      // Adjust essence if needed
      if (config.startingEssence !== 250) {
        mutations.changeEssence(
          config.startingEssence - 250,
          "starting_essence",
        );
      }

      // Generate initial atlas
      const neighbors = adjacentTides(startingTide);
      const atlas = generateInitialAtlas(state.completionLevel, {
        cardDatabase,
        dreamsignPool: DREAMSIGNS,
        playerHasBanes: false,
        startingTide,
        playerPool: state.pool,
        config,
      });

      const firstNodeId = atlas.edges[0]?.[1];
      const nodeCount = Object.keys(atlas.nodes).length - 1;

      logEvent("quest_started", {
        initialEssence: config.startingEssence,
        dreamscapesGenerated: nodeCount,
        startingTide,
        startingDeckSize:
          starterCards.length + tideCards.length + neutralCards.length,
      });

      mutations.updateAtlas(atlas);

      if (firstNodeId) {
        mutations.setCurrentDreamscape(firstNodeId);
      }

      mutations.setScreen({ type: "dreamscape" });
    },
    [state.completionLevel, mutations, cardDatabase, config],
  );

  return (
    <div className="flex min-h-screen flex-col items-center justify-center px-4">
      <motion.h1
        className="mb-2 text-center text-5xl font-extrabold tracking-wide md:text-7xl lg:text-8xl"
        style={{
          background:
            "linear-gradient(135deg, #a855f7 0%, #7c3aed 40%, #c084fc 100%)",
          WebkitBackgroundClip: "text",
          WebkitTextFillColor: "transparent",
          textShadow:
            "0 0 60px rgba(168, 85, 247, 0.4), 0 0 120px rgba(124, 58, 237, 0.2)",
          filter: "drop-shadow(0 0 40px rgba(168, 85, 247, 0.3))",
        }}
        initial={{ opacity: 0, y: -30 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.8, ease: "easeOut" }}
      >
        Dreamtides
      </motion.h1>

      <motion.p
        className="mb-10 text-center text-lg opacity-60 md:text-xl"
        style={{ color: "#e2e8f0" }}
        initial={{ opacity: 0 }}
        animate={{ opacity: 0.6 }}
        transition={{ duration: 0.8, delay: 0.3 }}
      >
        Choose Your Starting Tide
      </motion.p>

      <motion.div
        className="flex w-full max-w-3xl flex-col items-center gap-4 md:flex-row md:justify-center md:gap-6"
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6, delay: 0.5 }}
      >
        {options.map((tide) => {
          const color = TIDE_COLORS[tide];
          const neighbors = adjacentTides(tide);
          return (
            <motion.button
              key={tide}
              className="flex w-full max-w-[260px] cursor-pointer flex-col items-center rounded-xl px-5 py-6"
              style={{
                background:
                  "linear-gradient(145deg, #1a1025 0%, #0f0a18 60%, #0d0814 100%)",
                border: `2px solid ${color}40`,
                boxShadow: `0 0 20px ${color}15`,
              }}
              whileHover={{
                boxShadow: `0 0 30px ${color}40`,
                scale: 1.05,
                borderColor: `${color}80`,
              }}
              whileTap={{ scale: 0.97 }}
              onClick={() => {
                handleChooseTide(tide);
              }}
            >
              <img
                src={tideIconUrl(tide)}
                alt={tide}
                className="mb-3 h-14 w-14 rounded-full object-contain"
                style={{ border: `2px solid ${color}` }}
              />
              <span
                className="mb-2 text-xl font-bold"
                style={{ color }}
              >
                {tide}
              </span>
              <span
                className="text-center text-xs leading-relaxed opacity-60"
                style={{ color: "#e2e8f0" }}
              >
                10 {tide} cards, 10 Starter cards, 10 Neutral cards, +1{" "}
                {tide} crystal
              </span>
              <span
                className="mt-2 text-center text-[10px] opacity-40"
                style={{ color: "#e2e8f0" }}
              >
                Neighbors: {neighbors.join(" & ")}
              </span>
            </motion.button>
          );
        })}
      </motion.div>
    </div>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add scripts/constructed_quest_prototype/src/screens/QuestStartScreen.tsx
git commit -m "feat(quest): starting tide selection screen with 30-card deck"
```

---

## Task 5: Atlas Generator And Pack Generator

**Files:**
- Modify: `src/atlas/atlas-generator.ts`
- Modify: `src/atlas/atlas-generator.test.ts`
- Modify: `src/pack/pack-generator.ts`

- [ ] **Step 1: Update SiteGenerationContext in atlas-generator.ts**

In `src/atlas/atlas-generator.ts`, replace the `SiteGenerationContext` interface (lines 16-23):

```ts
/** Parameters for site generation that require external data. */
export interface SiteGenerationContext {
  cardDatabase: Map<number, CardData>;
  dreamsignPool: ReadonlyArray<Omit<Dreamsign, "isBane">>;
  playerHasBanes: boolean;
  startingTide: NamedTide | null;
  playerPool: DeckEntry[];
  config: QuestConfig;
}
```

Add `NamedTide` to the import from `../types/cards`:

```ts
import type { CardData, NamedTide, Tide } from "../types/cards";
```

- [ ] **Step 2: Update generateRewardData**

In the `generateRewardData` function (line 119), replace:

```ts
const { cardDatabase, dreamsignPool, startingTides } = context;
const excludedSet = new Set(startingTides);
```

With:

```ts
const { cardDatabase, dreamsignPool, startingTide } = context;
const excludedSet = new Set<Tide>();
if (startingTide !== null) {
  excludedSet.add(startingTide);
  for (const adj of adjacentTides(startingTide)) {
    excludedSet.add(adj);
  }
}
```

Also add `Starter` exclusion to the card filter on the next line:

```ts
const cards = Array.from(cardDatabase.values()).filter(
  (c) => !excludedSet.has(c.tide) && c.rarity !== "Starter",
);
```

- [ ] **Step 3: Update level 0 loot pack tide selection**

In `generateSiteComposition` (around line 174), replace:

```ts
const packTide = context.startingTides.length > 0
  ? context.startingTides[Math.floor(Math.random() * context.startingTides.length)]
  : pickRandom(["Bloom", "Arc", "Ignite", "Pact", "Umbra", "Rime", "Surge"] as Tide[]);
```

With:

```ts
const startTideAndNeighbors: Tide[] = [];
if (context.startingTide !== null) {
  startTideAndNeighbors.push(context.startingTide, ...adjacentTides(context.startingTide));
}
const packTide = startTideAndNeighbors.length > 0
  ? pickRandom(startTideAndNeighbors)
  : pickRandom(["Bloom", "Arc", "Ignite", "Pact", "Umbra", "Rime", "Surge"] as Tide[]);
```

- [ ] **Step 4: Update atlas-generator.test.ts**

In `src/atlas/atlas-generator.test.ts`, remove the `startingTides: 3` and `sequentialTides: true` lines from `TEST_CONFIG` (lines 18-19). Replace `startingTides: ["Bloom", "Arc", "Surge"]` in the `defaultContext` function (line 88) with `startingTide: "Bloom"`.

- [ ] **Step 5: Add Starter exclusion and neutral mixing to pack-generator.ts**

In `src/pack/pack-generator.ts`, replace the entire `generateLootPack` function:

```ts
/**
 * Generates loot pack contents for a given tide theme.
 * Filters the card database to matching-tide cards, applies duplicate
 * protection weights based on copies already in the player's pool,
 * then performs weighted sampling without replacement.
 * Approximately 50% of the time, one slot is replaced with a Neutral card.
 */
export function generateLootPack(
  cardDatabase: Map<number, CardData>,
  playerPool: ReadonlyArray<DeckEntry>,
  packTide: Tide,
  config: QuestConfig,
  isEnhanced: boolean,
): CardData[] {
  const candidates = Array.from(cardDatabase.values()).filter(
    (c) => c.tide === packTide && c.rarity !== "Starter",
  );

  if (candidates.length === 0) return [];

  // Count copies of each card number in the player's pool
  const copyCounts = new Map<number, number>();
  for (const entry of playerPool) {
    copyCounts.set(entry.cardNumber, (copyCounts.get(entry.cardNumber) ?? 0) + 1);
  }

  const packSize = isEnhanced
    ? config.lootPackSize * 2
    : config.lootPackSize;

  const tideCards = weightedSample(candidates, packSize, (card) => {
    const copies = copyCounts.get(card.cardNumber) ?? 0;
    return duplicateWeight(copies, config);
  });

  // Approximately 50% chance to replace one card with a Neutral
  const neutralCandidates = Array.from(cardDatabase.values()).filter(
    (c) => c.tide === "Neutral" && c.rarity !== "Starter" && c.rarity !== "Legendary",
  );
  if (neutralCandidates.length > 0 && tideCards.length > 0 && Math.random() < 0.5) {
    const neutralCard = neutralCandidates[Math.floor(Math.random() * neutralCandidates.length)];
    tideCards[tideCards.length - 1] = neutralCard;
  }

  return tideCards;
}
```

- [ ] **Step 6: Commit**

```bash
git add scripts/constructed_quest_prototype/src/atlas/atlas-generator.ts scripts/constructed_quest_prototype/src/atlas/atlas-generator.test.ts scripts/constructed_quest_prototype/src/pack/pack-generator.ts
git commit -m "feat(quest): update atlas context for single tide, add neutral mixing to packs"
```

---

## Task 6: Shop Generators

**Files:**
- Modify: `src/shop/shop-generator.ts`
- Modify: `src/shop/pack-shop-generator.ts`

- [ ] **Step 1: Update shop-generator.ts**

In `src/shop/shop-generator.ts`, change the `generateCardShopInventory` signature. Replace the `startingTides: Tide[]` parameter (line 39) with `seedTides: Tide[]` and add Starter exclusion.

Replace lines 36-84:

```ts
/**
 * Generates card shop inventory with `config.cardShopSize` card slots.
 * Cards are weighted toward the player's seed tides. Prices are
 * randomized in [cardPriceMin, cardPriceMax] rounded to nearest 5,
 * with 1-2 random slots receiving a 30-70% discount.
 */
export function generateCardShopInventory(
  cardDatabase: Map<number, CardData>,
  playerPool: DeckEntry[],
  seedTides: Tide[],
  config: QuestConfig,
): ShopSlot[] {
  const poolTideCounts = countDeckTides(playerPool, cardDatabase);

  // Use seed tides to seed counts if pool is empty
  for (const tide of seedTides) {
    if (!poolTideCounts.has(tide)) {
      poolTideCounts.set(tide, 1);
    }
  }

  const allCards = Array.from(cardDatabase.values()).filter(
    (c) => c.tide !== "Neutral" && c.rarity !== "Starter",
  );

  const selected = weightedSample(
    allCards,
    config.cardShopSize,
    (card) => tideWeight(card.tide, poolTideCounts),
  );

  const slots: ShopSlot[] = selected.map((card) => {
    const rawPrice =
      config.cardPriceMin +
      Math.random() * (config.cardPriceMax - config.cardPriceMin);
    const basePrice = Math.round(rawPrice / 5) * 5;
    return {
      card,
      basePrice,
      discountPercent: 0,
      purchased: false,
    };
  });

  // Apply discounts to 1-2 random slots
  const discountCount = Math.random() < 0.5 ? 1 : 2;
  const indices = slots.map((_, i) => i).sort(() => Math.random() - 0.5);
  for (let d = 0; d < discountCount && d < indices.length; d++) {
    // 30, 40, 50, 60, or 70 percent discount
    const discount = 30 + Math.floor(Math.random() * 5) * 10;
    slots[indices[d]] = { ...slots[indices[d]], discountPercent: discount };
  }

  return slots;
}
```

- [ ] **Step 2: Update pack-shop-generator.ts**

In `src/shop/pack-shop-generator.ts`, update `pickWeightedTide` parameter name (line 14-16):

```ts
function pickWeightedTide(
  poolTideCounts: Map<Tide, number>,
  seedTides: Tide[],
): Tide {
  for (const tide of seedTides) {
```

Update `generateTidePackCards` to exclude Starter and add neutral mixing (lines 41-50):

```ts
/** Generates cards for a tide pack: 3-4 cards from the chosen tide + 0-1 neutral. */
function generateTidePackCards(
  cardDatabase: Map<number, CardData>,
  tide: Tide,
): CardData[] {
  const candidates = Array.from(cardDatabase.values()).filter(
    (c) => c.tide === tide && c.rarity !== "Starter",
  );
  if (candidates.length === 0) return [];
  const tideCards = weightedSample(candidates, 4, () => 1);

  // Approximately 50% chance to replace last card with a Neutral
  const neutralCandidates = Array.from(cardDatabase.values()).filter(
    (c) => c.tide === "Neutral" && c.rarity !== "Starter" && c.rarity !== "Legendary",
  );
  if (neutralCandidates.length > 0 && tideCards.length > 0 && Math.random() < 0.5) {
    const neutralCard = neutralCandidates[Math.floor(Math.random() * neutralCandidates.length)];
    tideCards[tideCards.length - 1] = neutralCard;
  }

  return tideCards;
}
```

Update `generateAlliancePackCards` to exclude Starter (line 59):

```ts
const candidates = Array.from(cardDatabase.values()).filter((c) =>
  allowedTides.has(c.tide) && c.rarity !== "Starter",
);
```

Update `generateRemovalPackCards` to exclude Starter (line 71):

```ts
const candidates = Array.from(cardDatabase.values()).filter((c) => {
  if (c.rarity === "Starter") return false;
  const text = c.renderedText.toLowerCase();
  return REMOVAL_KEYWORDS.some((kw) => text.includes(kw));
});
```

Update `generateAggroPackCards` to exclude Starter (line 83):

```ts
const candidates = Array.from(cardDatabase.values()).filter(
  (c) =>
    c.cardType === "Character" &&
    c.energyCost !== null &&
    c.energyCost <= 3 &&
    c.rarity !== "Starter",
);
```

Update `generateEventsPackCards` to exclude Starter (line 97):

```ts
const candidates = Array.from(cardDatabase.values()).filter(
  (c) => c.cardType === "Event" && c.rarity !== "Starter",
);
```

Update `generatePackShopInventory` parameter name from `startingTides: Tide[]` to `seedTides: Tide[]` (line 110). Update the two calls to `pickWeightedTide` to pass `seedTides` instead of `startingTides`.

- [ ] **Step 3: Commit**

```bash
git add scripts/constructed_quest_prototype/src/shop/shop-generator.ts scripts/constructed_quest_prototype/src/shop/pack-shop-generator.ts
git commit -m "feat(quest): exclude Starter from shops, add neutral mixing to tide packs"
```

---

## Task 7: Screens — Shop, PackShop, Battle, Forge

**Files:**
- Modify: `src/screens/ShopScreen.tsx`
- Modify: `src/screens/PackShopScreen.tsx`
- Modify: `src/screens/BattleScreen.tsx`
- Modify: `src/screens/ForgeScreen.tsx`
- Modify: `src/forge/forge-logic.ts`
- Modify: `src/data/tide-weights.ts`

- [ ] **Step 1: Create a helper to derive seed tides**

Add a helper function at the top of `src/data/tide-weights.ts` (after imports):

```ts
import type { NamedTide } from "../types/cards";

/** Derives seed tides from a starting tide: the tide itself + its two neighbors. */
export function startingTideSeedTides(startingTide: NamedTide | null): Tide[] {
  if (startingTide === null) return [];
  return [startingTide, ...adjacentTides(startingTide)];
}
```

Also add Starter exclusion to `selectRareRewards` (line 83):

```ts
const rareCards = Array.from(cardDatabase.values()).filter(
  (c) => c.rarity === "Rare" && !excludedSet.has(c.tide),
);
```

This already excludes non-Rare, but verify no Starter cards have rarity "Rare" (they don't — Starters are rarity "Starter").

- [ ] **Step 2: Update ShopScreen.tsx**

In `src/screens/ShopScreen.tsx`, add import:

```ts
import { startingTideSeedTides } from "../data/tide-weights";
```

Replace `state.startingTides` with `startingTideSeedTides(state.startingTide)` in two places:

Line 29 (initial state):
```ts
const [slots, setSlots] = useState<ShopSlot[]>(() =>
  generateCardShopInventory(cardDatabase, state.pool, startingTideSeedTides(state.startingTide), config),
);
```

Line 78 (reroll):
```ts
setSlots(generateCardShopInventory(cardDatabase, state.pool, startingTideSeedTides(state.startingTide), config));
```

Also update the dependency array around line 80 to use `state.startingTide` instead of `state.startingTides`.

- [ ] **Step 3: Update PackShopScreen.tsx**

In `src/screens/PackShopScreen.tsx`, add import:

```ts
import { startingTideSeedTides } from "../data/tide-weights";
```

Replace `state.startingTides` (line 44):

```ts
const [packs, setPacks] = useState<PackShopSlot[]>(() =>
  generatePackShopInventory(cardDatabase, state.pool, startingTideSeedTides(state.startingTide), config),
);
```

- [ ] **Step 4: Update BattleScreen.tsx**

In `src/screens/BattleScreen.tsx`, add import:

```ts
import { startingTideSeedTides } from "../data/tide-weights";
```

**Update enemy generation** (line 37-58). The `generateEnemy` function uses `DREAMCALLERS` and reads `template.tide`. Since dreamcallers now have `tides: [NamedTide, NamedTide]`, update to use the first tide:

```ts
function generateEnemy(): EnemyData {
  const template = DREAMCALLERS[Math.floor(Math.random() * DREAMCALLERS.length)];
  const prefixes = [
    "Shadow", "Nightmare", "Phantom", "Dark",
    "Cursed", "Twisted", "Fallen", "Spectral",
  ];
  const prefix = prefixes[Math.floor(Math.random() * prefixes.length)];
  const baseName = template.name.split(",")[0].split(" the ")[0];

  return {
    name: `${prefix} ${baseName}`,
    abilityText: template.abilityDescription,
    dreamsignCount: Math.floor(Math.random() * 5) + 1,
    tide: template.tides[0],
  };
}
```

**Update atlas generation context** (around line 967). Replace:

```ts
startingTides: state.startingTides,
```

With:

```ts
startingTide: state.startingTide,
```

Update the dependency array (around line 997) to use `state.startingTide` instead of `state.startingTides`.

- [ ] **Step 5: Update forge-logic.ts**

In `src/forge/forge-logic.ts`, add import:

```ts
import type { NamedTide } from "../types/cards";
```

Update `getForgeEligibleCards` signature (line 89):

```ts
export function getForgeEligibleCards(
  cardDatabase: Map<number, CardData>,
  startingTide: NamedTide | null,
  excludeTide?: Tide,
): CardData[] {
  const tideSet = new Set<Tide>();
  if (startingTide !== null) {
    tideSet.add(startingTide);
    for (const adj of adjacentTides(startingTide)) {
      tideSet.add(adj);
    }
  }
  const eligible = Array.from(cardDatabase.values()).filter(
    (c) =>
      (tideSet.size === 0 || tideSet.has(c.tide)) &&
      c.tide !== "Neutral" &&
      c.rarity !== "Starter" &&
      (excludeTide === undefined || c.tide !== excludeTide),
  );

  if (eligible.length <= 20) return eligible;

  const shuffled = eligible.slice();
  for (let i = shuffled.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
  }
  return shuffled.slice(0, 20);
}
```

Also add Starter exclusion to `generateForgeRecipes` output candidates (line 43):

```ts
const outputCandidates = Array.from(cardDatabase.values()).filter(
  (c) => c.tide !== "Neutral" && c.rarity !== "Starter",
);
```

- [ ] **Step 6: Update ForgeScreen.tsx**

In `src/screens/ForgeScreen.tsx`, replace `startingTides` destructuring (line 25):

```ts
const { pool, deck, startingTide } = state;
```

Update the call to `getForgeEligibleCards` (line 98-102):

```ts
const eligible = getForgeEligibleCards(
  cardDatabase,
  startingTide,
  currentRecipe.sacrificeTide,
);
```

Update the dependency array (line 110) to use `startingTide` instead of `startingTides`.

- [ ] **Step 7: Commit**

```bash
git add scripts/constructed_quest_prototype/src/screens/ShopScreen.tsx scripts/constructed_quest_prototype/src/screens/PackShopScreen.tsx scripts/constructed_quest_prototype/src/screens/BattleScreen.tsx scripts/constructed_quest_prototype/src/screens/ForgeScreen.tsx scripts/constructed_quest_prototype/src/forge/forge-logic.ts scripts/constructed_quest_prototype/src/data/tide-weights.ts
git commit -m "feat(quest): update all screens for single startingTide"
```

---

## Task 8: Dreamcaller Draft Screen

**Files:**
- Modify: `src/screens/DreamcallerDraftScreen.tsx`

- [ ] **Step 1: Rewrite the dreamcaller offer logic and UI**

Replace the entire content of `src/screens/DreamcallerDraftScreen.tsx`:

```tsx
import { useCallback, useEffect, useRef, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import type { Dreamcaller, SiteState } from "../types/quest";
import type { NamedTide, Tide } from "../types/cards";
import { useQuest } from "../state/quest-context";
import { DREAMCALLERS } from "../data/dreamcallers";
import { TIDE_COLORS, tideIconUrl, adjacentTides } from "../data/card-database";
import { countDeckTides, tideWeight, weightedSample } from "../data/tide-weights";
import { logEvent } from "../logging";

/** Returns the left neighbor of a named tide on the circle. */
function leftNeighbor(tide: NamedTide): NamedTide {
  const circle: NamedTide[] = ["Bloom", "Arc", "Ignite", "Pact", "Umbra", "Rime", "Surge"];
  const idx = circle.indexOf(tide);
  return circle[(idx + circle.length - 1) % circle.length];
}

/** Returns the right neighbor of a named tide on the circle. */
function rightNeighbor(tide: NamedTide): NamedTide {
  const circle: NamedTide[] = ["Bloom", "Arc", "Ignite", "Pact", "Umbra", "Rime", "Surge"];
  const idx = circle.indexOf(tide);
  return circle[(idx + 1) % circle.length];
}

/** Returns true if a dreamcaller's pair contains both specified tides. */
function matchesPair(dc: Dreamcaller, a: NamedTide, b: NamedTide): boolean {
  return (dc.tides[0] === a && dc.tides[1] === b) || (dc.tides[0] === b && dc.tides[1] === a);
}

/** Returns true if a dreamcaller's pair contains the specified tide. */
function containsTide(dc: Dreamcaller, tide: NamedTide): boolean {
  return dc.tides[0] === tide || dc.tides[1] === tide;
}

/**
 * Selects 3 dreamcallers for the draft:
 * 1. Left fork: pair containing startingTide + left neighbor
 * 2. Right fork: pair containing startingTide + right neighbor
 * 3. Adaptive: weighted sample from remaining callers
 *
 * Falls back gracefully when desired pairs are unavailable.
 */
function selectOfferedDreamcallers(
  startingTide: NamedTide | null,
  deck: Array<{ cardNumber: number }>,
  cardDatabase: Map<number, { tide: Tide }>,
): Dreamcaller[] {
  const pool = [...DREAMCALLERS];

  if (startingTide === null) {
    // No starting tide: shuffle and pick 3
    for (let i = pool.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [pool[i], pool[j]] = [pool[j], pool[i]];
    }
    return pool.slice(0, 3);
  }

  const left = leftNeighbor(startingTide);
  const right = rightNeighbor(startingTide);
  const selected: Dreamcaller[] = [];
  const usedNames = new Set<string>();

  // Slot 1: Left fork (startingTide + left neighbor)
  const leftCandidates = pool.filter((dc) => matchesPair(dc, startingTide, left));
  const leftFallback1 = pool.filter((dc) => containsTide(dc, startingTide) && !usedNames.has(dc.name));
  const leftFallback2 = pool.filter((dc) => (containsTide(dc, left) || containsTide(dc, right)) && !usedNames.has(dc.name));

  const leftPick = leftCandidates[0] ?? leftFallback1[0] ?? leftFallback2[0] ?? pool[0];
  selected.push(leftPick);
  usedNames.add(leftPick.name);

  // Slot 2: Right fork (startingTide + right neighbor)
  const rightCandidates = pool.filter((dc) => matchesPair(dc, startingTide, right) && !usedNames.has(dc.name));
  const rightFallback1 = pool.filter((dc) => containsTide(dc, startingTide) && !usedNames.has(dc.name));
  const rightFallback2 = pool.filter((dc) => (containsTide(dc, left) || containsTide(dc, right)) && !usedNames.has(dc.name));

  const rightPick = rightCandidates[0] ?? rightFallback1[0] ?? rightFallback2[0] ?? pool.filter((dc) => !usedNames.has(dc.name))[0] ?? pool[0];
  selected.push(rightPick);
  usedNames.add(rightPick.name);

  // Slot 3: Adaptive (weighted by deck composition)
  const remaining = pool.filter((dc) => !usedNames.has(dc.name));
  if (remaining.length === 0) {
    selected.push(pool.filter((dc) => !usedNames.has(dc.name))[0] ?? pool[0]);
  } else if (deck.length === 0) {
    selected.push(remaining[Math.floor(Math.random() * remaining.length)]);
  } else {
    const tideCounts = countDeckTides(deck, cardDatabase);
    const picked = weightedSample(remaining, 1, (dc) => {
      return Math.max(tideWeight(dc.tides[0], tideCounts), tideWeight(dc.tides[1], tideCounts));
    });
    selected.push(picked[0] ?? remaining[0]);
  }

  logEvent("dreamcaller_offers_generated", {
    offered: selected.map((dc) => ({ name: dc.name, tides: dc.tides })),
    startingTide,
  });

  return selected;
}

interface DreamcallerCardProps {
  dreamcaller: Dreamcaller;
  isSelected: boolean;
  isDismissed: boolean;
  onSelect: () => void;
}

/** Renders a single dreamcaller option with name, tides, ability, and bonus. */
function DreamcallerCard({
  dreamcaller,
  isSelected,
  isDismissed,
  onSelect,
}: DreamcallerCardProps) {
  const primaryColor = TIDE_COLORS[dreamcaller.tides[0]];
  const secondaryColor = TIDE_COLORS[dreamcaller.tides[1]];

  return (
    <motion.div
      className="flex flex-col items-center rounded-xl px-5 py-6 md:px-6 md:py-8"
      style={{
        background: "linear-gradient(145deg, #1a1025 0%, #0f0a18 60%, #0d0814 100%)",
        border: `2px solid ${primaryColor}40`,
        boxShadow: `0 0 20px ${primaryColor}15`,
        minWidth: "220px",
        maxWidth: "320px",
        flex: "1 1 0",
      }}
      animate={
        isSelected
          ? { scale: 1.08, boxShadow: `0 0 40px ${primaryColor}60` }
          : isDismissed
            ? { opacity: 0, scale: 0.9 }
            : { scale: 1, opacity: 1 }
      }
      transition={{ duration: 0.5, ease: "easeOut" }}
    >
      {/* Two tide icons */}
      <div className="mb-3 flex items-center gap-2">
        <img
          src={tideIconUrl(dreamcaller.tides[0])}
          alt={dreamcaller.tides[0]}
          className="h-12 w-12 rounded-full object-contain md:h-14 md:w-14"
          style={{ border: `2px solid ${primaryColor}` }}
        />
        <span className="text-lg opacity-40" style={{ color: "#e2e8f0" }}>
          +
        </span>
        <img
          src={tideIconUrl(dreamcaller.tides[1])}
          alt={dreamcaller.tides[1]}
          className="h-12 w-12 rounded-full object-contain md:h-14 md:w-14"
          style={{ border: `2px solid ${secondaryColor}` }}
        />
      </div>

      {/* Dreamcaller name */}
      <h3
        className="mb-2 text-center text-xl font-bold leading-tight md:text-2xl"
        style={{ color: primaryColor }}
      >
        {dreamcaller.name}
      </h3>

      {/* Tide labels */}
      <div className="mb-3 flex items-center gap-2">
        <span
          className="rounded-full px-2 py-0.5 text-xs font-medium"
          style={{
            background: `${primaryColor}20`,
            color: primaryColor,
            border: `1px solid ${primaryColor}30`,
          }}
        >
          {dreamcaller.tides[0]}
        </span>
        <span
          className="rounded-full px-2 py-0.5 text-xs font-medium"
          style={{
            background: `${secondaryColor}20`,
            color: secondaryColor,
            border: `1px solid ${secondaryColor}30`,
          }}
        >
          {dreamcaller.tides[1]}
        </span>
      </div>

      {/* Ability description */}
      <p
        className="mb-4 text-center text-sm leading-relaxed opacity-80"
        style={{ color: "#e2e8f0" }}
      >
        {dreamcaller.abilityDescription}
      </p>

      {/* Essence bonus */}
      <div className="mb-2 flex items-center gap-1.5">
        <span style={{ color: "#fbbf24" }}>{"\u25C6"}</span>
        <span className="text-lg font-bold" style={{ color: "#fbbf24" }}>
          +{String(dreamcaller.essenceBonus)}
        </span>
        <span className="text-xs opacity-50">Essence</span>
      </div>

      {/* Tide crystal grant */}
      <div className="mb-5 flex items-center gap-1.5">
        <img
          src={tideIconUrl(dreamcaller.tideCrystalGrant)}
          alt={dreamcaller.tideCrystalGrant}
          className="h-4 w-4 rounded-full object-contain"
        />
        <span className="text-sm font-medium" style={{ color: TIDE_COLORS[dreamcaller.tideCrystalGrant] }}>
          1 {dreamcaller.tideCrystalGrant} Crystal
        </span>
      </div>

      {/* Select button */}
      {!isSelected && !isDismissed && (
        <motion.button
          className="cursor-pointer rounded-lg px-6 py-2 text-sm font-bold text-white"
          style={{
            background: "linear-gradient(135deg, #7c3aed 0%, #6d28d9 100%)",
            border: "2px solid rgba(168, 85, 247, 0.6)",
            boxShadow: "0 0 12px rgba(124, 58, 237, 0.3)",
          }}
          whileHover={{
            boxShadow: "0 0 20px rgba(124, 58, 237, 0.5)",
            scale: 1.05,
          }}
          whileTap={{ scale: 0.97 }}
          onClick={onSelect}
        >
          Select
        </motion.button>
      )}

      {isSelected && (
        <span
          className="rounded-full px-4 py-1.5 text-sm font-bold"
          style={{
            background: `${primaryColor}20`,
            color: primaryColor,
            border: `1px solid ${primaryColor}`,
          }}
        >
          Selected
        </span>
      )}
    </motion.div>
  );
}

/** Screen for selecting a dreamcaller from 3 options. */
export function DreamcallerDraftScreen({ site }: { site: SiteState }) {
  const { state, mutations, cardDatabase } = useQuest();
  const [selectedIndex, setSelectedIndex] = useState<number | null>(null);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    return () => {
      if (timerRef.current !== null) {
        clearTimeout(timerRef.current);
      }
    };
  }, []);

  const offeredRef = useRef<Dreamcaller[] | null>(null);
  if (offeredRef.current === null) {
    offeredRef.current = selectOfferedDreamcallers(
      state.startingTide,
      state.deck,
      cardDatabase,
    );
  }
  const offered = offeredRef.current;

  const handleSelect = useCallback(
    (index: number) => {
      if (selectedIndex !== null) return;

      const dreamcaller = offered[index];
      setSelectedIndex(index);

      mutations.setDreamcaller(dreamcaller);
      mutations.changeEssence(dreamcaller.essenceBonus, "dreamcaller_bonus");
      mutations.addTideCrystal(dreamcaller.tideCrystalGrant, 1);

      logEvent("site_completed", {
        siteType: "DreamcallerDraft",
        outcome: `Selected ${dreamcaller.name}`,
        dreamcallerName: dreamcaller.name,
        dreamcallerTides: dreamcaller.tides,
        essenceBonus: dreamcaller.essenceBonus,
      });

      timerRef.current = setTimeout(() => {
        mutations.markSiteVisited(site.id);
        mutations.setScreen({ type: "dreamscape" });
      }, 500);
    },
    [selectedIndex, offered, mutations, site.id],
  );

  return (
    <motion.div
      className="flex min-h-screen flex-col items-center px-4 py-8 md:px-8 md:py-12"
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      transition={{ duration: 0.4 }}
    >
      <motion.h1
        className="mb-2 text-center text-3xl font-extrabold tracking-wide md:text-4xl"
        style={{
          background: "linear-gradient(135deg, #a855f7 0%, #7c3aed 50%, #c084fc 100%)",
          WebkitBackgroundClip: "text",
          WebkitTextFillColor: "transparent",
          filter: "drop-shadow(0 0 20px rgba(168, 85, 247, 0.3))",
        }}
        initial={{ opacity: 0, y: -10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5, delay: 0.1 }}
      >
        Choose Your Dreamcaller
      </motion.h1>

      <motion.p
        className="mb-8 text-center text-sm opacity-50 md:mb-10 md:text-base"
        style={{ color: "#e2e8f0" }}
        initial={{ opacity: 0 }}
        animate={{ opacity: 0.5 }}
        transition={{ duration: 0.5, delay: 0.2 }}
      >
        Select a dreamcaller to guide your journey through the dreamscape
      </motion.p>

      <AnimatePresence>
        <motion.div
          className="flex w-full max-w-4xl flex-col items-center gap-4 md:flex-row md:items-stretch md:justify-center md:gap-6"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.3 }}
        >
          {offered.map((dreamcaller, index) => (
            <DreamcallerCard
              key={dreamcaller.name}
              dreamcaller={dreamcaller}
              isSelected={selectedIndex === index}
              isDismissed={selectedIndex !== null && selectedIndex !== index}
              onSelect={() => {
                handleSelect(index);
              }}
            />
          ))}
        </motion.div>
      </AnimatePresence>
    </motion.div>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add scripts/constructed_quest_prototype/src/screens/DreamcallerDraftScreen.tsx
git commit -m "feat(quest): two-tide dreamcaller draft with archetype forks"
```

---

## Task 9: DraftSite Starter Exclusion

**Files:**
- Modify: `src/screens/DraftSiteScreen.tsx`

- [ ] **Step 1: Add Starter exclusion to DraftSiteScreen**

In `src/screens/DraftSiteScreen.tsx`, update the draft card selection (around line 22). Change:

```ts
const allCards = Array.from(cardDatabase.values());
```

To:

```ts
const allCards = Array.from(cardDatabase.values()).filter(
  (c) => c.rarity !== "Starter",
);
```

- [ ] **Step 2: Commit**

```bash
git add scripts/constructed_quest_prototype/src/screens/DraftSiteScreen.tsx
git commit -m "feat(quest): exclude Starter cards from draft site offers"
```

---

## Task 10: DeckViewer Quest Origin And Two-Tide Dreamcaller

**Files:**
- Modify: `src/components/DeckViewer.tsx`
- Modify: `src/components/HUD.tsx`

- [ ] **Step 1: Add Quest Origin to DeckViewer**

In `src/components/DeckViewer.tsx`, find the Dreamcaller sidebar section (around line 610). Add a Quest Origin section just above it:

```tsx
{/* Quest Origin section */}
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
        background: "rgba(124, 58, 237, 0.1)",
        border: "1px solid rgba(124, 58, 237, 0.2)",
      }}
    >
      <img
        src={tideIconUrl(state.startingTide)}
        alt={state.startingTide}
        className="h-5 w-5 rounded-full"
        style={{
          border: `1px solid ${TIDE_COLORS[state.startingTide]}`,
        }}
      />
      <span
        className="text-sm font-bold"
        style={{ color: TIDE_COLORS[state.startingTide] }}
      >
        {state.startingTide}
      </span>
    </div>
  </div>
)}
```

- [ ] **Step 2: Update dreamcaller display for two tides**

In the same file, find the dreamcaller display (around line 618-656). Replace the single tide icon and name with two:

```tsx
{state.dreamcaller !== null ? (
  <div
    className="rounded-lg p-3"
    style={{
      background: "rgba(124, 58, 237, 0.1)",
      border: "1px solid rgba(124, 58, 237, 0.2)",
    }}
  >
    <div className="flex items-center gap-2">
      <img
        src={tideIconUrl(state.dreamcaller.tides[0])}
        alt={state.dreamcaller.tides[0]}
        className="h-5 w-5 rounded-full"
        style={{
          border: `1px solid ${TIDE_COLORS[state.dreamcaller.tides[0]]}`,
        }}
      />
      <img
        src={tideIconUrl(state.dreamcaller.tides[1])}
        alt={state.dreamcaller.tides[1]}
        className="h-5 w-5 rounded-full"
        style={{
          border: `1px solid ${TIDE_COLORS[state.dreamcaller.tides[1]]}`,
        }}
      />
      <span
        className="text-sm font-bold"
        style={{
          color: TIDE_COLORS[state.dreamcaller.tides[0]],
        }}
      >
        {state.dreamcaller.name}
      </span>
    </div>
    <p
      className="mt-2 text-[11px] leading-relaxed opacity-70"
      style={{ color: "#e2e8f0" }}
    >
      {state.dreamcaller.abilityDescription}
    </p>
  </div>
) : (
  <p className="text-xs opacity-40">
    No dreamcaller selected.
  </p>
)}
```

- [ ] **Step 3: Update HUD.tsx for two-tide dreamcaller**

In `src/components/HUD.tsx`, replace the dreamcaller color derivation (lines 65-68):

```ts
const dreamcallerName = state.dreamcaller?.name ?? null;
const dreamcallerTide = state.dreamcaller?.tides[0] ?? null;
const dreamcallerColor = dreamcallerTide !== null ? TIDE_COLORS[dreamcallerTide] : "#6b7280";
```

- [ ] **Step 4: Commit**

```bash
git add scripts/constructed_quest_prototype/src/components/DeckViewer.tsx scripts/constructed_quest_prototype/src/components/HUD.tsx
git commit -m "feat(quest): add Quest Origin display, update dreamcaller for two tides"
```

---

## Task 11: Build Verification

**Files:** None (verification only)

- [ ] **Step 1: Run all tests**

```bash
npm test
```

Expected: All tests PASS.

- [ ] **Step 2: Run typecheck**

```bash
npm run typecheck
```

Expected: No type errors.

- [ ] **Step 3: Run lint**

```bash
npm run lint
```

Expected: No lint errors.

- [ ] **Step 4: Run build**

```bash
npm run build
```

Expected: Build succeeds.

- [ ] **Step 5: Fix any errors found in steps 1-4**

If there are errors, fix them. Common issues:
- References to `dreamcaller.tide` (should be `dreamcaller.tides[0]` or `dreamcaller.tides`)
- References to `state.startingTides` (should be `state.startingTide`)
- Missing `NamedTide` imports
- Test assertions using old field shapes

- [ ] **Step 6: Commit any fixes**

```bash
git add -u scripts/constructed_quest_prototype/
git commit -m "fix(quest): resolve build errors from starting tide migration"
```

---

## Task 12: Manual QA

**Files:** None (QA only)

Use the `qa` skill for browser verification. Use `agent-browser` to take screenshots. Keep screenshots under `/tmp/constructed-quest-starting-tide-qa/`.

- [ ] **Step 1: Start dev server**

```bash
npm run dev
```

- [ ] **Step 2: QA scenario — Starting tide selection**

Open the app and verify:
- 3 named tide options displayed (no Neutral)
- Each shows tide icon, name, neighbors, and deck composition description
- Clicking a tide starts the quest

Take screenshot of the tide selection screen.

- [ ] **Step 3: QA scenario — Starting deck verification**

After selecting a tide:
- Open deck viewer
- Verify deck size is exactly 30
- Verify Quest Origin section shows the selected tide
- Verify deck contains ~10 Starter cards (Neutral), ~10 starting tide cards, ~10 Neutral non-starter cards
- Verify starting tide crystal appears in crystal display

Take screenshot of deck viewer.

- [ ] **Step 4: QA scenario — Dreamcaller draft**

Visit the dreamcaller draft site:
- Verify 3 dreamcaller options displayed
- Verify each shows TWO tide icons
- Verify the left-fork and right-fork match the starting tide's neighbors
- Select a dreamcaller and verify essence and crystal are granted

Take screenshot of dreamcaller draft.

- [ ] **Step 5: QA scenario — Loot pack**

Visit a loot pack site:
- Verify no Starter cards in the pack
- Check if a neutral card occasionally appears (may need multiple visits)

Take screenshot.

- [ ] **Step 6: QA scenario — Card shop**

Visit a card shop:
- Verify no Starter cards offered
- Verify cards are weighted toward starting tide

Take screenshot.

- [ ] **Step 7: QA scenario — Pack shop**

Visit a pack shop:
- Verify no Starter cards in any packs

Take screenshot.

- [ ] **Step 8: QA scenario — Battle and reload**

Win a battle, then reload the page:
- Verify no state leaks (returns to quest start screen)
- Verify 3 fresh tide options appear

Take screenshot.

- [ ] **Step 9: Commit QA results**

No code changes expected from QA. If bugs are found, fix them in new commits.
