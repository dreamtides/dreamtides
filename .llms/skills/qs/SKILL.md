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

- Run `npm run typecheck`, `npm run lint`, and `npm test` after all changes.
- Run `just fmt` then `just review` for the overall repo gate.
- **Browser QA with agent-browser is MANDATORY for every change.** Do not skip
  this step. Do not substitute code review for browser testing. Changes are not
  complete until agent-browser QA passes.

## Browser QA with agent-browser

Every change must be verified in a real browser using `agent-browser`. This is
not optional. The tool is at `/Users/dthurn/Library/pnpm/agent-browser`.

### Setup

Start the dev server in the background, then open it:

```bash
cd scripts/quest_prototype && npm run dev &
sleep 2
agent-browser open http://localhost:5173
agent-browser wait --load networkidle
```

### Navigation Flow

The quest prototype is a single-page app. Navigate by clicking UI elements.
Use `snapshot` to see interactive elements and their `@ref` IDs:

```bash
# See what's on screen (interactive elements only)
agent-browser snapshot -i

# Click "Begin Quest" button (use @ref from snapshot)
agent-browser click @e3

# Wait for screen transition to complete
agent-browser wait 500
```

Typical quest flow: Begin Quest → Atlas (click dreamscape node) → Dreamscape
(click sites one by one) → Battle → Victory → back to Atlas → repeat.

### Taking and Analyzing Screenshots

Take screenshots at every significant UI state. Use `--annotate` for labeled
screenshots that are easier to analyze:

```bash
# Regular screenshot
agent-browser screenshot /tmp/qs-quest-start.png

# Annotated screenshot (adds element labels — better for analysis)
agent-browser screenshot --annotate /tmp/qs-atlas-annotated.png

# Full-page screenshot (captures below the fold)
agent-browser screenshot --full /tmp/qs-dreamscape-full.png
```

After taking a screenshot, **read it with the Read tool** to visually inspect:

```
Read /tmp/qs-quest-start.png
```

When analyzing screenshots, verify:
- Dark fantasy theme (deep purple-black background #0a0612)
- Correct text colors (off-white #e2e8f0, gold #fbbf24 for essence)
- Card art loads (not broken image placeholders)
- Layout is correct (no overlapping elements, proper spacing)
- Animations completed (no half-rendered states)
- HUD shows correct values (essence, deck size, battle counter)

### Checking Console Output

Use `eval` to read browser console logs for JSONL events:

```bash
# Get all JSONL log entries
agent-browser eval "JSON.stringify(window.__questLog || [])"

# Check for JavaScript errors
agent-browser eval "JSON.stringify(window.__errors || [])"
```

Or check the accessibility snapshot for HUD values:

```bash
agent-browser snapshot -i | grep -i "essence\|cards\|battle"
```

### Responsive Testing

Test at both desktop (1280px) and tablet (768px) widths:

```bash
# Desktop
agent-browser eval "window.resizeTo(1280, 800)"
agent-browser screenshot /tmp/qs-desktop.png

# Tablet
agent-browser eval "window.resizeTo(768, 1024)"
agent-browser screenshot /tmp/qs-tablet.png
```

### Minimum QA Checklist

For every change, at minimum:

1. Start dev server and open in agent-browser
2. Click "Begin Quest"
3. Navigate to a dreamscape and visit at least 2 sites
4. Screenshot each site screen you visit — read and verify visually
5. Complete a battle and verify victory screen
6. Check that HUD values update correctly
7. If your change affects a specific site type, visit that site type and
   screenshot before/after states
8. Verify zero JS errors via `eval`
9. Test at 768px width if your change affects layout

### Common Patterns

```bash
# Full QA session example
agent-browser open http://localhost:5173
agent-browser wait --load networkidle
agent-browser screenshot /tmp/qs-start.png

# Begin quest
agent-browser snapshot -i
agent-browser click "Begin Quest"
agent-browser wait 1000
agent-browser screenshot /tmp/qs-atlas.png

# Enter first dreamscape
agent-browser snapshot -i
# (find the dreamscape node ref from snapshot)
agent-browser click @e5
agent-browser wait 500
agent-browser screenshot /tmp/qs-dreamscape.png

# Visit a site
agent-browser snapshot -i
# (find the site button ref from snapshot)
agent-browser click @e8
agent-browser wait 500
agent-browser screenshot /tmp/qs-site.png
```
