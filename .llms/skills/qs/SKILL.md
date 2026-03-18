---
name: qs
description: Use when working with the quest prototype, adding quest features, fixing quest bugs, or running quest prototype tests and typechecking. Triggers on quest prototype, quest sim, qs, quest bug, quest_prototype.
---

# Quest Prototype (QS)

Read the documentation before making changes:

- **Full documentation**: [docs/quest_prototype/quest_prototype.md](../../../docs/quest_prototype/quest_prototype.md) — architecture, module layout, site registration, state extension, JSONL logging, and card data normalization.

## Running

```bash
cd scripts/quest_prototype
npm install          # required — node_modules is not committed
npm run setup-assets # generates card-data.json, symlinks card images, copies tide PNGs
npm run dev          # starts Vite dev server (runs setup-assets first)
```

## Commands

- `npm run typecheck` — tsc --noEmit
- `npm run lint` — eslint src/
- `npm test` — vitest run
- `npm run build` — production build

**Worktrees:** Each git worktree starts without `node_modules`. Run `npm install` before any typecheck, lint, or test commands.

## Key Files

| File | Role |
|------|------|
| `src/state/quest-context.tsx` | QuestProvider, mutations, useQuestContext |
| `src/types/quest.ts` | QuestState, Screen, SiteType |
| `src/types/draft.ts` | DraftState, BotState |
| `src/components/ScreenRouter.tsx` | Screen dispatch by currentScreen |
| `src/atlas/atlas-generator.ts` | Dream Atlas + dreamscape generation |
| `src/draft/draft-engine.ts` | 10-seat cube draft engine |
| `src/logging.ts` | JSONL event logger |
| `src/data/` | Synthetic data (dreamcallers, dreamsigns, journeys, offers, biomes) |
| `src/shop/shop-generator.ts` | Shop item generation |
| `src/transfiguration/transfiguration-logic.ts` | Transfiguration type logic |

## Adding a New Site Type

Two changes required:

1. **Screen component** — create `src/screens/MySiteScreen.tsx` and add a case to `ScreenRouter.tsx`.
2. **Atlas pool** — add the site type with a weight to `buildAdditionalSitePool` in `src/atlas/atlas-generator.ts`. Without this the site is unreachable during gameplay.

## Extending QuestState

`QuestContextValue`, `QuestMutations`, and `QuestState` are in `src/state/quest-context.tsx` and `src/types/quest.ts`. Every mutation that changes game state must call `logEvent` — missing log calls are a conformance blocker.

## Acceptance Criteria

- **Manual testing is CRITICAL.** After every change, run the prototype in a browser and verify correct behavior:
  - `cd scripts/quest_prototype && npm run dev`
  - Navigate through at least one full dreamscape (select sites, make picks, reach battle).
  - Verify the changed behavior works correctly in context.
  - Check browser console for JSONL events and zero JS errors.
- Run `npm run typecheck`, `npm run lint`, and `npm test` after all changes.
- Run `just fmt` then `just review` for the overall repo gate.
