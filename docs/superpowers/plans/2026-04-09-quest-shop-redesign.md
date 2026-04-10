# Quest Shop-Focused Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redesign the quest prototype's early game to remove tide selection,
use a fixed starter deck, make shops the primary card acquisition path, and
reset essence between dreamscapes.

**Architecture:** Replace QuestStartScreen with direct initialization logic in
quest-context. Modify atlas-generator for new level 0 composition. Update
economy defaults in quest-config. Add essence reset to dreamscape transitions
and an essence warning before battles.

**Tech Stack:** TypeScript, React, Vitest

______________________________________________________________________

### Task 1: Update Economy Defaults in quest-config.ts

**Files:**

- Modify:
  `scripts/constructed_quest_prototype/src/state/quest-config.ts:121-135`

- Modify:
  `scripts/constructed_quest_prototype/src/atlas/atlas-generator.test.ts:15-53`
  (TEST_CONFIG)

- [ ] **Step 1: Update default values in getQuestConfig()**

In `src/state/quest-config.ts`, change the following defaults in the
`getQuestConfig()` function:

```typescript
    startingEssence: parseIntParam(params, "startingEssence", 400, 0, 9999),
    // ...
    cardPriceMin: parseIntParam(params, "cardPriceMin", 30, 0, 9999),
    cardPriceMax: parseIntParam(params, "cardPriceMax", 70, 0, 9999),
    rerollBase: parseIntParam(params, "rerollBase", 10, 0, 9999),
    rerollIncrement: parseIntParam(params, "rerollIncrement", 5, 0, 9999),
```

- [ ] **Step 2: Update pack prices in pack-shop-generator.ts**

In `src/shop/pack-shop-generator.ts`, change hardcoded pack prices:

```typescript
// Line 150 (alliance pack): change price from 125 to 100
            price: 100,
// Line 159 (removal pack): change price from 125 to 100
            price: 100,
// Line 167 (aggro pack): change price from 100 to 75
            price: 75,
// Line 175 (events pack): change price from 100 to 75
            price: 75,
// Line 186 (tide pack): change price from 100 to 75
            price: 75,
```

- [ ] **Step 3: Update TEST_CONFIG in atlas-generator.test.ts**

In `src/atlas/atlas-generator.test.ts`, update the TEST_CONFIG to match new
defaults:

```typescript
const TEST_CONFIG: QuestConfig = {
  // ... keep existing values except:
  startingEssence: 400,
  cardPriceMin: 30,
  cardPriceMax: 70,
  rerollBase: 10,
  rerollIncrement: 5,
  // ... rest unchanged
};
```

- [ ] **Step 4: Run tests**

Run: `cd scripts/constructed_quest_prototype && npx vitest run` Expected: All
tests pass

- [ ] **Step 5: Commit**

```bash
git add scripts/constructed_quest_prototype/src/state/quest-config.ts scripts/constructed_quest_prototype/src/shop/pack-shop-generator.ts scripts/constructed_quest_prototype/src/atlas/atlas-generator.test.ts
git commit -m "Update economy defaults: cheaper cards, rerolls, and packs for shop-focused redesign"
```

______________________________________________________________________

### Task 2: Replace QuestStartScreen with Direct Initialization

**Files:**

- Modify:
  `scripts/constructed_quest_prototype/src/state/quest-context.tsx:72-101`
  (createDefaultState, setCurrentDreamscape)
- Modify:
  `scripts/constructed_quest_prototype/src/components/ScreenRouter.tsx:45-48`
- Modify: `scripts/constructed_quest_prototype/src/App.tsx:16`
- Modify: `scripts/constructed_quest_prototype/src/types/quest.ts:121-127`
- Modify:
  `scripts/constructed_quest_prototype/src/atlas/atlas-generator.ts:17-24`
  (SiteGenerationContext)

This task replaces the tide selection screen with automatic initialization. The
player starts directly in the first dreamscape with a fixed starter deck.

- [ ] **Step 1: Add dreamscapeTide to SiteGenerationContext**

In `src/atlas/atlas-generator.ts`, update the interface:

```typescript
export interface SiteGenerationContext {
  cardDatabase: Map<number, CardData>;
  dreamsignPool: ReadonlyArray<Omit<Dreamsign, "isBane">>;
  playerHasBanes: boolean;
  startingTide: NamedTide | null;
  playerPool: DeckEntry[];
  config: QuestConfig;
  /** The random tide assigned to dreamscape 1 for loot packs and shop weighting. */
  dreamscapeTide?: NamedTide;
}
```

- [ ] **Step 2: Add initializeQuest mutation to quest-context.tsx**

In `src/state/quest-context.tsx`, add a new mutation to `QuestMutations`
interface (after `resetQuest`):

```typescript
  initializeQuest: (cardDatabase: Map<number, CardData>, config: QuestConfig) => void;
```

Add the import for `QuestConfig` at the top:

```typescript
import type { QuestConfig } from "./quest-config";
```

Add the import for atlas generation and data:

```typescript
import { generateInitialAtlas, resetAtlasGenerator } from "../atlas/atlas-generator";
import { DREAMSIGNS } from "../data/dreamsigns";
import { NAMED_TIDES } from "../data/card-database";
```

Then implement the mutation inside `QuestProvider`, before the `mutations`
useMemo:

```typescript
  /** Card numbers for each starter card and the number of copies in the fixed deck. */
  const STARTER_DECK_ALLOCATION: Array<[number, number]> = [
    [718, 4], // Glimpse of What Was (1-cost Event)
    [720, 3], // Worlds Await (1-cost Event)
    [711, 4], // Nocturne Strummer (2-cost Character)
    [717, 2], // Flashpoint Detonation (2-cost Event)
    [719, 2], // Sign of Arrival (2-cost Event)
    [715, 3], // Final Witness (3-cost Character)
    [712, 3], // Ringwatcher (3-cost Character)
    [713, 4], // Marked Direwolf (4-cost Character)
    [714, 3], // Runebound Champion (5-cost Character)
    [716, 2], // Wildflower Colossus (6-cost Character)
  ];

  const initializeQuest = useCallback(
    (db: Map<number, CardData>, config: QuestConfig) => {
      setState((prev) => {
        // Build fixed starter deck
        const deck: DeckEntry[] = [];
        for (const [cardNumber, copies] of STARTER_DECK_ALLOCATION) {
          for (let i = 0; i < copies; i++) {
            entryIdCounter.current += 1;
            deck.push({
              entryId: `deck-${String(entryIdCounter.current)}`,
              cardNumber,
              transfiguration: null,
              isBane: false,
            });
          }
        }

        logEvent("starting_deck_initialized", {
          totalDeckSize: deck.length,
          allocation: STARTER_DECK_ALLOCATION.map(([cn, copies]) => ({
            cardNumber: cn,
            name: db.get(cn)?.name ?? "Unknown",
            copies,
          })),
        });

        // Pick a random tide for the first dreamscape
        const dreamscapeTide = NAMED_TIDES[
          Math.floor(Math.random() * NAMED_TIDES.length)
        ];

        logEvent("dreamscape_tide_selected", { tide: dreamscapeTide });

        // Generate initial atlas (startingTide is null since player has no tide)
        const atlas = generateInitialAtlas(0, {
          cardDatabase: db,
          dreamsignPool: DREAMSIGNS,
          playerHasBanes: false,
          startingTide: null,
          playerPool: [],
          config,
          dreamscapeTide,
        });

        const firstNodeId = atlas.edges[0]?.[1] ?? null;

        logEvent("quest_started", {
          initialEssence: config.startingEssence,
          dreamscapeTide,
          startingDeckSize: deck.length,
        });

        return {
          ...prev,
          essence: config.startingEssence,
          deck,
          pool: [],
          atlas,
          currentDreamscape: firstNodeId,
          visitedSites: [],
          screen: { type: "dreamscape" } as Screen,
          startingTide: null,
        };
      });
    },
    [],
  );
```

Add `initializeQuest` to the `mutations` useMemo object and its dependency
array.

- [ ] **Step 3: Remove "questStart" and "viewStartingDeck" screen types**

In `src/types/quest.ts`, remove those two screen variants:

```typescript
export type Screen =
  | { type: "dreamscape" }
  | { type: "atlas" }
  | { type: "site"; siteId: string }
  | { type: "questComplete" };
```

Update `createDefaultState()` in `src/state/quest-context.tsx` to use
`"dreamscape"` as initial screen:

```typescript
    screen: { type: "dreamscape" },
```

- [ ] **Step 4: Update ScreenRouter.tsx**

Remove the `QuestStartScreen` and `StartingDeckScreen` imports and cases:

```typescript
// Remove these imports:
// import { QuestStartScreen } from "../screens/QuestStartScreen";
// import { StartingDeckScreen } from "../screens/StartingDeckScreen";

// Remove these cases from renderScreen():
//   case "questStart":
//     return <QuestStartScreen />;
//   case "viewStartingDeck":
//     return <StartingDeckScreen />;
```

- [ ] **Step 5: Update App.tsx to call initializeQuest on mount**

Replace the `QuestApp` component to trigger initialization:

```typescript
import { useEffect } from "react";
import { useQuestConfig } from "./state/quest-config";

function QuestApp({
  cardDatabase,
}: {
  cardDatabase: Map<number, CardData>;
}) {
  const { state, mutations } = useQuest();
  const config = useQuestConfig();
  const [deckEditorOpen, setDeckEditorOpen] = useState(false);

  // Initialize quest on first render if not yet initialized
  const initializedRef = useRef(false);
  useEffect(() => {
    if (!initializedRef.current && state.deck.length === 0) {
      initializedRef.current = true;
      mutations.initializeQuest(cardDatabase, config);
    }
  }, [state.deck.length, mutations, cardDatabase, config]);

  const showHud = state.screen.type !== "questComplete" && state.deck.length > 0;

  // ... rest unchanged
```

Add the `useRef` import.

- [ ] **Step 6: Run tests and typecheck**

Run:
`cd scripts/constructed_quest_prototype && npx tsc --noEmit && npx vitest run`
Expected: Passes (some tests may need minor updates for removed screen types)

- [ ] **Step 7: Commit**

```bash
git add scripts/constructed_quest_prototype/src/state/quest-context.tsx scripts/constructed_quest_prototype/src/components/ScreenRouter.tsx scripts/constructed_quest_prototype/src/App.tsx scripts/constructed_quest_prototype/src/types/quest.ts scripts/constructed_quest_prototype/src/atlas/atlas-generator.ts
git commit -m "Replace tide selection with direct initialization using fixed starter deck"
```

______________________________________________________________________

### Task 3: Update Level 0 Dreamscape Composition

**Files:**

- Modify:
  `scripts/constructed_quest_prototype/src/atlas/atlas-generator.ts:70-200`
- Modify:
  `scripts/constructed_quest_prototype/src/atlas/atlas-generator.ts:17-24`
  (SiteGenerationContext)
- Modify:
  `scripts/constructed_quest_prototype/src/atlas/atlas-generator.test.ts`

The first dreamscape should have: DreamcallerDraft, 2 LootPacks (same random
tide), CardShop, PackShop, Battle. The `dreamscapeTide` field was added to
`SiteGenerationContext` in Task 2.

- [ ] **Step 1: Update level 0 composition in generateSiteComposition()**

The `dreamscapeTide` field was added to `SiteGenerationContext` in Task 2.

Replace the `clampedLevel === 0` branch:

```typescript
  if (clampedLevel === 0) {
    // Level 0: DreamcallerDraft, 2 LootPacks (same tide), CardShop, PackShop, Battle
    sites.push({
      id: nextSiteId(),
      type: "DreamcallerDraft",
      isEnhanced: false,
      isVisited: false,
    });

    const packTide: Tide = context.dreamscapeTide ??
      pickRandom(["Bloom", "Arc", "Ignite", "Pact", "Umbra", "Rime", "Surge"] as Tide[]);

    for (let i = 0; i < 2; i++) {
      sites.push({
        id: nextSiteId(),
        type: "LootPack",
        isEnhanced: false,
        isVisited: false,
        data: { packTide },
      });
    }

    sites.push({
      id: nextSiteId(),
      type: "CardShop",
      isEnhanced: false,
      isVisited: false,
      data: { dreamscapeTide: packTide },
    });
    sites.push({
      id: nextSiteId(),
      type: "PackShop",
      isEnhanced: false,
      isVisited: false,
      data: { dreamscapeTide: packTide },
    });
  }
```

- [ ] **Step 2: Ensure level 1+ always includes CardShop and PackShop**

In the `else` branch of `generateSiteComposition()`, after the pool sites loop,
add guaranteed shops if not already present:

```typescript
    // Ensure at least one CardShop and one PackShop
    if (!sites.some((s) => s.type === "CardShop")) {
      sites.push({
        id: nextSiteId(),
        type: "CardShop",
        isEnhanced: false,
        isVisited: false,
      });
    }
    if (!sites.some((s) => s.type === "PackShop")) {
      sites.push({
        id: nextSiteId(),
        type: "PackShop",
        isEnhanced: false,
        isVisited: false,
      });
    }
```

Add this right before the `// Battle site always last` comment.

- [ ] **Step 3: Update atlas-generator.test.ts for new level 0 composition**

Update the existing level 0 test to expect the new composition:

```typescript
  it("level 0 produces DreamcallerDraft, 2 LootPacks, CardShop, PackShop, Battle", () => {
    const sites = generateSiteComposition(0, defaultContext({ dreamscapeTide: "Bloom" }));
    const types = sites.map((s) => s.type);
    expect(types[0]).toBe("DreamcallerDraft");
    expect(types.filter((t) => t === "LootPack")).toHaveLength(2);
    expect(types).toContain("CardShop");
    expect(types).toContain("PackShop");
    expect(types[types.length - 1]).toBe("Battle");
    expect(types).toHaveLength(6);
  });
```

Add a test for level 1+ guaranteed shops:

```typescript
  it("level 1+ always includes CardShop and PackShop", () => {
    // Run multiple times to account for randomness
    for (let i = 0; i < 20; i++) {
      const sites = generateSiteComposition(1, defaultContext());
      const types = sites.map((s) => s.type);
      expect(types).toContain("CardShop");
      expect(types).toContain("PackShop");
    }
  });
```

- [ ] **Step 4: Run tests**

Run: `cd scripts/constructed_quest_prototype && npx vitest run` Expected: All
tests pass

- [ ] **Step 5: Commit**

```bash
git add scripts/constructed_quest_prototype/src/atlas/atlas-generator.ts scripts/constructed_quest_prototype/src/atlas/atlas-generator.test.ts
git commit -m "Update level 0 composition: 2 loot packs, CardShop + PackShop, guaranteed shops in all dreamscapes"
```

______________________________________________________________________

### Task 4: Shop Tide Neighborhood Weighting for Dreamscape 1

**Files:**

- Modify: `scripts/constructed_quest_prototype/src/screens/ShopScreen.tsx:47-49`
- Modify:
  `scripts/constructed_quest_prototype/src/screens/PackShopScreen.tsx:41-48`
- Modify: `scripts/constructed_quest_prototype/src/shop/shop-generator.ts:45-97`
- Modify:
  `scripts/constructed_quest_prototype/src/shop/pack-shop-generator.ts:120-194`

For dreamscape 1, shops should weight cards toward the dreamscape tide and its
neighbors (the 3-tide neighborhood). The `dreamscapeTide` is stored in
`site.data` from Task 3.

- [ ] **Step 1: Update ShopScreen to pass dreamscapeTide as seed tides**

In `src/screens/ShopScreen.tsx`, update the shop inventory generation to use
`site.data?.dreamscapeTide` when available:

```typescript
import { adjacentTides } from "../data/card-database";
import type { NamedTide } from "../types/cards";

// Inside ShopScreen component, replace seedTides usage:
  const seedTides = useMemo(() => {
    const dreamscapeTide = site.data?.dreamscapeTide as NamedTide | undefined;
    if (dreamscapeTide) {
      return [dreamscapeTide, ...adjacentTides(dreamscapeTide)];
    }
    return startingTideSeedTides(state.startingTide);
  }, [site.data, state.startingTide]);

  const [slots, setSlots] = useState<ShopSlot[]>(() =>
    generateCardShopInventory(cardDatabase, state.pool, seedTides, config, playableTides),
  );
```

Also update the reroll handler (~line 101) to use the same `seedTides`:

```typescript
      generateCardShopInventory(cardDatabase, state.pool, seedTides, config, playableTides),
```

- [ ] **Step 2: Update PackShopScreen to pass dreamscapeTide as seed tides**

In `src/screens/PackShopScreen.tsx`, apply the same pattern:

```typescript
import { adjacentTides } from "../data/card-database";
import type { NamedTide } from "../types/cards";

// Inside PackShopScreen component:
  const seedTides = useMemo(() => {
    const dreamscapeTide = site.data?.dreamscapeTide as NamedTide | undefined;
    if (dreamscapeTide) {
      return [dreamscapeTide, ...adjacentTides(dreamscapeTide)];
    }
    return startingTideSeedTides(state.startingTide);
  }, [site.data, state.startingTide]);

  const [packs, setPacks] = useState<PackShopSlot[]>(() =>
    generatePackShopInventory(
      cardDatabase,
      state.pool,
      seedTides,
      config,
    ),
  );
```

- [ ] **Step 3: Run typecheck**

Run: `cd scripts/constructed_quest_prototype && npx tsc --noEmit` Expected: No
type errors

- [ ] **Step 4: Commit**

```bash
git add scripts/constructed_quest_prototype/src/screens/ShopScreen.tsx scripts/constructed_quest_prototype/src/screens/PackShopScreen.tsx
git commit -m "Use dreamscape tide neighborhood for shop weighting in first dreamscape"
```

______________________________________________________________________

### Task 5: Essence Reset Between Dreamscapes

**Files:**

- Modify:
  `scripts/constructed_quest_prototype/src/screens/AtlasScreen.tsx:87-97`

When the player clicks a dreamscape node on the atlas, reset their essence to 0
before entering.

- [ ] **Step 1: Add essence reset to handleNodeClick in AtlasScreen**

In `src/screens/AtlasScreen.tsx`, update `handleNodeClick`:

```typescript
  const handleNodeClick = useCallback(
    (nodeId: string) => {
      if (didDrag.current) return;
      const node = atlas.nodes[nodeId];
      if (!node || node.status !== "available") return;

      // Reset essence to 0 when entering a new dreamscape
      if (state.essence > 0) {
        logEvent("essence_reset_on_dreamscape_entry", {
          essenceLost: state.essence,
          dreamscapeId: nodeId,
        });
        mutations.changeEssence(-state.essence, "dreamscape_transition");
      }

      mutations.setCurrentDreamscape(nodeId);
      mutations.setScreen({ type: "dreamscape" });
    },
    [atlas.nodes, mutations, state.essence],
  );
```

Add the `logEvent` import if not already present, and add `state` to the
component's destructuring.

- [ ] **Step 2: Run typecheck**

Run: `cd scripts/constructed_quest_prototype && npx tsc --noEmit` Expected: No
type errors

- [ ] **Step 3: Commit**

```bash
git add scripts/constructed_quest_prototype/src/screens/AtlasScreen.tsx
git commit -m "Reset essence to 0 when entering a new dreamscape"
```

______________________________________________________________________

### Task 6: Unspent Essence Warning Before Battle

**Files:**

- Modify:
  `scripts/constructed_quest_prototype/src/screens/BattleScreen.tsx:102-262`
  (PreBattlePhase)

- Modify: `scripts/constructed_quest_prototype/src/state/quest-context.tsx` (add
  essenceWarningShown to state)

- Modify: `scripts/constructed_quest_prototype/src/types/quest.ts:130-145`
  (QuestState)

- [ ] **Step 1: Add essenceWarningShown to QuestState**

In `src/types/quest.ts`, add to `QuestState`:

```typescript
export interface QuestState {
  // ... existing fields
  essenceWarningShown: boolean;
}
```

In `src/state/quest-context.tsx`, update `createDefaultState()`:

```typescript
    essenceWarningShown: false,
```

Add a mutation to `QuestMutations` interface:

```typescript
  dismissEssenceWarning: () => void;
```

Implement it in `QuestProvider`:

```typescript
  const dismissEssenceWarning = useCallback(() => {
    setState((prev) => ({ ...prev, essenceWarningShown: true }));
  }, []);
```

Add `dismissEssenceWarning` to the `mutations` useMemo object and its dependency
array.

- [ ] **Step 2: Add essence warning dialog to PreBattlePhase**

In `src/screens/BattleScreen.tsx`, update `PreBattlePhase` to accept and show an
essence warning:

Add new props to the PreBattleProps interface:

```typescript
interface PreBattleProps {
  // ... existing props
  essence: number;
  essenceWarningShown: boolean;
  onDismissEssenceWarning: () => void;
}
```

Inside the `PreBattlePhase` component, add state and logic:

```typescript
  const [showEssenceWarning, setShowEssenceWarning] = useState(false);

  const handleStartClick = useCallback(() => {
    if (!essenceWarningShown && essence >= 30) {
      setShowEssenceWarning(true);
    } else {
      onStartBattle();
    }
  }, [essenceWarningShown, essence, onStartBattle]);

  const handleConfirmBattle = useCallback(() => {
    setShowEssenceWarning(false);
    onDismissEssenceWarning();
    onStartBattle();
  }, [onDismissEssenceWarning, onStartBattle]);

  const handleCancelBattle = useCallback(() => {
    setShowEssenceWarning(false);
  }, []);
```

Change the "Start Battle" button's `onClick` from `onStartBattle` to
`handleStartClick`.

Add the warning dialog (after the DeckEditor, before the closing fragment):

```typescript
      {showEssenceWarning && (
        <motion.div
          className="fixed inset-0 z-50 flex items-center justify-center"
          style={{ background: "rgba(0, 0, 0, 0.7)" }}
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
        >
          <motion.div
            className="flex max-w-md flex-col items-center rounded-xl px-8 py-6"
            style={{
              background: "linear-gradient(145deg, #1a1025 0%, #0f0a18 60%, #0d0814 100%)",
              border: "2px solid rgba(251, 191, 36, 0.4)",
              boxShadow: "0 0 30px rgba(251, 191, 36, 0.2)",
            }}
            initial={{ scale: 0.9, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
          >
            <h3
              className="mb-3 text-xl font-bold"
              style={{ color: "#fbbf24" }}
            >
              Unspent Essence
            </h3>
            <p
              className="mb-6 text-center text-sm leading-relaxed opacity-80"
              style={{ color: "#e2e8f0" }}
            >
              You have {String(essence)} essence remaining. Unspent essence is
              lost after this battle. Continue?
            </p>
            <div className="flex gap-4">
              <motion.button
                className="cursor-pointer rounded-lg px-5 py-2 font-medium text-white"
                style={{
                  background: "rgba(100, 100, 100, 0.3)",
                  border: "1px solid rgba(255, 255, 255, 0.2)",
                }}
                whileHover={{ scale: 1.05 }}
                whileTap={{ scale: 0.97 }}
                onClick={handleCancelBattle}
              >
                Go Back
              </motion.button>
              <motion.button
                className="cursor-pointer rounded-lg px-5 py-2 font-bold text-white"
                style={{
                  background: "linear-gradient(135deg, #7c3aed 0%, #a855f7 100%)",
                  border: "1px solid rgba(168, 85, 247, 0.5)",
                }}
                whileHover={{ scale: 1.05 }}
                whileTap={{ scale: 0.97 }}
                onClick={handleConfirmBattle}
              >
                Start Battle
              </motion.button>
            </div>
          </motion.div>
        </motion.div>
      )}
```

- [ ] **Step 3: Pass new props from BattleScreen to PreBattlePhase**

In the `BattleScreen` component (~line 1058-1066), update the PreBattlePhase
rendering:

```typescript
      {phase === "preBattle" && (
        <motion.div key="preBattle">
          <PreBattlePhase
            enemy={enemy}
            completionLevel={completionLevel}
            isMiniboss={isMiniboss}
            isFinalBoss={isFinalBoss}
            cardDatabase={cardDatabase}
            onStartBattle={handleStartBattle}
            essence={state.essence}
            essenceWarningShown={state.essenceWarningShown}
            onDismissEssenceWarning={mutations.dismissEssenceWarning}
          />
        </motion.div>
      )}
```

- [ ] **Step 4: Run typecheck and tests**

Run:
`cd scripts/constructed_quest_prototype && npx tsc --noEmit && npx vitest run`
Expected: Passes

- [ ] **Step 5: Commit**

```bash
git add scripts/constructed_quest_prototype/src/screens/BattleScreen.tsx scripts/constructed_quest_prototype/src/state/quest-context.tsx scripts/constructed_quest_prototype/src/types/quest.ts
git commit -m "Add one-time unspent essence warning before battle when player has >= 30 essence"
```

______________________________________________________________________

### Task 7: Fix Remaining References to Removed State

**Files:**

- Modify: `scripts/constructed_quest_prototype/src/screens/BattleScreen.tsx:967`
  (startingTide in generateNewNodes)

- Modify:
  `scripts/constructed_quest_prototype/src/state/quest-state-machine.test.ts`

- Delete: `scripts/constructed_quest_prototype/src/screens/QuestStartScreen.tsx`

- Delete:
  `scripts/constructed_quest_prototype/src/screens/StartingDeckScreen.tsx`

- [ ] **Step 1: Delete QuestStartScreen.tsx and StartingDeckScreen.tsx**

These files are no longer referenced after Task 2.

```bash
rm scripts/constructed_quest_prototype/src/screens/QuestStartScreen.tsx
rm scripts/constructed_quest_prototype/src/screens/StartingDeckScreen.tsx
```

- [ ] **Step 2: Fix quest-state-machine.test.ts**

Update any test that references `{ type: "questStart" }` or
`{ type: "viewStartingDeck" }` screens. Change the default state screen to
`{ type: "dreamscape" }` and remove/update any assertions about removed screen
types.

In the test file, update the default state:

```typescript
    screen: { type: "dreamscape" },
```

Remove or update any test cases that assert
`screenName({ type: "questStart" })`.

- [ ] **Step 3: Run all tests**

Run:
`cd scripts/constructed_quest_prototype && npx tsc --noEmit && npx vitest run`
Expected: All pass

- [ ] **Step 4: Commit**

```bash
git add -A scripts/constructed_quest_prototype/
git commit -m "Remove QuestStartScreen, StartingDeckScreen, and fix remaining references"
```

______________________________________________________________________

### Task 8: Manual Smoke Test

- [ ] **Step 1: Start dev server**

Run: `cd scripts/constructed_quest_prototype && npx vite dev`

- [ ] **Step 2: Verify start flow**

Open http://localhost:5173 in browser. Verify:

- No tide selection screen appears

- Player starts directly in a dreamscape

- Deck has 30 cards (all starters with correct allocation)

- Starting essence is 400

- First dreamscape has: DreamcallerDraft, 2 LootPacks, CardShop, PackShop,
  Battle

- [ ] **Step 3: Verify shops**

- Open Card Shop: cards should be weighted toward the loot pack tide
  neighborhood

- Card prices should be in 30-70 range

- Reroll cost should start at 10, increment by 5

- Open Pack Shop: tide packs should cost 75, special packs 75-100

- [ ] **Step 4: Verify essence warning**

- With essence remaining (>= 30), click to start a battle

- Warning dialog should appear with essence amount

- "Go Back" should dismiss the dialog

- "Start Battle" should proceed and suppress future warnings

- [ ] **Step 5: Verify essence reset**

- Complete a battle and reach the atlas

- Note current essence

- Click on a new dreamscape

- Essence should be 0

- [ ] **Step 6: Final commit if any fixes needed**

```bash
git add -A scripts/constructed_quest_prototype/
git commit -m "Fix issues found during smoke testing"
```
