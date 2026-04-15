---
name: qs
description: Use when working with the quest prototype, adding quest features, fixing quest bugs, or running quest prototype tests and typechecking. Triggers on quest prototype, quest sim, qs, quest bug, quest_prototype.
---

# Quest Prototype (QS)

Read these first:

- **Architecture + current flow**: [docs/quest_prototype/quest_prototype.md](../../../docs/quest_prototype/quest_prototype.md)
- **Browser QA + tooling notes**: [docs/quest_prototype/qa_tooling.md](../../../docs/quest_prototype/qa_tooling.md)

## Current Runtime Model

- The prototype lives in `scripts/quest_prototype/`.
- Quest start is now a **Dreamcaller selection** screen, not a `"Begin Quest"`
  landing page. The player chooses 1 of 3 Dreamcallers, then
  `src/screens/quest-start-bootstrap.ts` resolves a fixed package, initializes
  the starter deck, draft state, and atlas, and enters the first dreamscape
  directly.
- The live flow does **not** use the legacy tide-pick step or a
  `DreamcallerDraft` surface.
- Top-level state is `QuestState` in `src/types/quest.ts`. Routing is driven by
  `state.screen`, not `currentScreen`.
- Use `useQuest()` from `src/state/quest-context.tsx`, not
  `useQuestContext()`.
- Logging goes through `logEvent()` in `src/logging.ts`. In dev it:
  1. writes one JSON line to `console.log`
  2. stores entries in-memory for tests via `getLogEntries()`
  3. POSTs to `/api/log`
  4. gets appended by Vite to `scripts/quest_prototype/logs/quest-log.jsonl`

## Running

```bash
cd scripts/quest_prototype
npm install
npm run dev
```

`npm run dev` runs `scripts/setup-assets.mjs` automatically first. That setup
script is idempotent and refreshes:

- `public/card-data.json`
- `public/dreamcaller-data.json`
- `public/cards/` symlinks into the local image cache
- `public/tides/` copied tide icons

Useful one-offs:

```bash
npm run setup-assets
npm run typecheck
npm run lint
npm test
npm run build
```

**Worktrees:** `node_modules` is not committed. Run `npm install` before
typecheck, lint, tests, or browser QA in a fresh worktree.

## Key Files

| File | Role |
|------|------|
| `src/App.tsx` | App shell, `QuestProvider`, HUD, deck viewer intro, debug overlay |
| `src/state/quest-context.tsx` | `QuestState` mutations, `useQuest()`, reset/default-state logic |
| `src/types/quest.ts` | `QuestState`, `Screen`, `SiteType`, atlas/site types |
| `src/components/ScreenRouter.tsx` | Dispatches `state.screen` and site screens |
| `src/screens/QuestStartScreen.tsx` | Dreamcaller offer UI |
| `src/screens/quest-start-bootstrap.ts` | Resolves package and enters the first dreamscape |
| `src/data/quest-content.ts` | Loads normalized quest content and validates Dreamcaller packages |
| `src/atlas/atlas-generator.ts` | Atlas generation, site pools, site metadata, dreamscape creation |
| `src/draft/draft-engine.ts` | Fixed-pool draft logic with 4-card offers and persisted draft state |
| `src/logging.ts` | JSONL event logging, in-memory log access, download helper |
| `src/components/HUD.tsx` | Essence/deck/dreamcaller HUD, debug button, log download |
| `src/screens/DebugScreen.tsx` | Package and draft-pool debug overlay |

## Adding a New Site Type

At minimum, update the places that define type, routing, and reachability:

1. Add the new variant to `SiteType` in `src/types/quest.ts`.
2. Implement the site UI in `src/screens/`.
3. Route it from `SiteScreen` in `src/components/ScreenRouter.tsx`.
4. Add display metadata to `SITE_TYPE_META` in `src/atlas/atlas-generator.ts`.
5. If the site should spawn normally, add it to `buildAdditionalSitePool()`.
6. If a biome can enhance it, add or update entries in `src/data/biomes.ts`.
7. Add or update tests for atlas generation and the new screen behavior.

Without the atlas pool change, the site is unreachable in normal gameplay.

## Extending Quest State

- Update `QuestState` in `src/types/quest.ts`.
- Thread the new data through `QuestContextValue`, `QuestMutations`, and
  `createDefaultState()` in `src/state/quest-context.tsx`.
- Update any bootstrap/reset helpers that need to preserve or clear the field.
- Every mutation or state transition that changes quest state should emit a
  `logEvent()` entry. Missing quest logs are a conformance problem.

For tests, prefer the public surface and assert log behavior through
`getLogEntries()` rather than adding test-only hooks.

## Acceptance Criteria

- Run `npm run typecheck`, `npm run lint`, and `npm test` after changes.
- Run browser QA with `agent-browser`. This is mandatory for quest prototype
  work.
- Run `just fmt` and then `just review` at the repo root before finishing.

## Browser QA With agent-browser

Confirm the tool exists:

```bash
which agent-browser
```

Start the dev server in one shell:

```bash
cd /Users/dthurn/dreamtides/scripts/quest_prototype
npm install
npm run dev
```

Open the app in another:

```bash
agent-browser open http://localhost:5173
agent-browser wait --load networkidle
agent-browser snapshot -i
```

### Current Smoke Path

Use this flow unless the change targets something narrower:

1. Confirm the app starts on the **Dreamcaller selection** screen with exactly
   3 choices.
2. Pick a Dreamcaller.
3. Confirm the starter-deck overlay opens, then click **Begin Quest**.
4. Confirm the run is already in quest play with no legacy tide-pick step and
   no `DreamcallerDraft` surface.
5. Enter a dreamscape site. Battle should stay locked until the non-battle
   sites are visited.
6. Reach a draft site and confirm the offer shows 4 unique cards when enough
   unique cards remain.
7. Reach a Dreamsign surface when relevant and confirm shown Dreamsigns are
   spent from the shared pool for the run.
8. Complete a battle and confirm atlas progression afterward.
9. Open the HUD debug overlay when relevant and verify package/draft summary
   details appear there, not in normal player-facing UI.

### Screenshots

Take screenshots at each meaningful state transition:

```bash
agent-browser screenshot /tmp/qs-start.png
agent-browser screenshot --annotate /tmp/qs-dreamcaller.png
agent-browser screenshot --annotate /tmp/qs-dreamscape.png
agent-browser screenshot --full /tmp/qs-site.png
```

Inspect screenshots visually after capture. Verify:

- card art and Dreamcaller portraits load
- layout spacing is stable
- battle/site buttons are not clipped or overlapped
- HUD values make sense for the current state
- normal screens do not expose package internals or legacy tide-crystal UI

### Logs And Errors

Do **not** rely on `window.__questLog` or `window.__errors`; the current app
does not publish those globals.

Use the current logging surfaces instead:

```bash
tail -n 40 /Users/dthurn/dreamtides/scripts/quest_prototype/logs/quest-log.jsonl
```

Also watch the dev-server terminal for JavaScript or asset-load errors. If you
need a saved copy from the UI, use the HUD `Download Log` button.

### Responsive Checks

If the change affects layout, test both desktop and tablet widths:

```bash
agent-browser eval "window.resizeTo(1280, 800)"
agent-browser screenshot /tmp/qs-desktop.png

agent-browser eval "window.resizeTo(768, 1024)"
agent-browser screenshot /tmp/qs-tablet.png
```
