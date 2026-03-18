# Quest Prototype

A standalone web prototype of Dreamtides Quest Mode at
`scripts/quest_prototype/`. Implements the full roguelike loop: cube draft,
Dream Atlas navigation, 15 site types, auto-resolved battles, and essence
economy across 7 battles. State is in-memory (resets on page load). No backend,
no persistence.

## Running the Prototype

```
cd scripts/quest_prototype
npm install          # required — node_modules is not committed
npm run dev          # runs setup-assets.mjs then starts Vite dev server
```

`npm run dev` invokes `scripts/setup-assets.mjs` automatically before starting
Vite. The setup script is idempotent and:

1. Parses `rendered-cards.toml` to write `public/card-data.json` (483 cards,
   Special-rarity excluded).
2. Symlinks `public/cards/{cardNumber}.webp` into the local image cache at
   `~/Library/Caches/io.github.dreamtides.tv/image_cache/`. Missing images are
   skipped with a warning.
3. Copies the 7 tide PNGs into `public/tides/`.

The `public/cards/`, `public/tides/`, and `public/card-data.json` paths are
gitignored.

Other commands:

```
npm run typecheck    # tsc --noEmit
npm run lint         # eslint src/
npm test             # vitest run
npm run build        # production build
```

**Worktrees:** Each git worktree starts without `node_modules`. Run
`npm install` before any typecheck, lint, or test commands.

## Tech Stack

React 19, Vite 7, TypeScript 5.8 (strict mode). Tailwind CSS v4 via
`@tailwindcss/vite` plugin — no `tailwind.config.js` needed; import via
`@import "tailwindcss"` in `src/index.css`. Framer Motion for animations.
`smol-toml` for the asset setup script.

TypeScript is configured for bundler mode (`moduleResolution: "bundler"`, no
`@types/node`). Node built-in modules (`node:fs`, etc.) are not available in
type-checked code. Tests that need file I/O should mock `fetch` or use Vitest's
`node` environment, not `import("node:fs")`.

ESLint uses `typescript-eslint` `recommendedTypeChecked` with `no-unsafe-*`
rules at error level. The `eslint-plugin-react-hooks` plugin is **not**
configured — `react-hooks/exhaustive-deps` is not enforced.

## Architecture

All state lives in a single `QuestState` object provided by `QuestProvider` in
`src/state/quest-context.tsx`. Components read state and call mutations through
`useQuestContext()`. The current screen is stored in `state.currentScreen` and
drives `ScreenRouter.tsx`.

```
src/
  main.tsx            — entry point, mounts App
  App.tsx             — QuestProvider wrapper
  index.css           — Tailwind import
  types/
    cards.ts          — CardData type, Tide union
    quest.ts          — QuestState, Screen, site types
    draft.ts          — DraftState, BotState
  state/
    quest-context.tsx — QuestProvider, QuestMutations, useQuestContext
  logging.ts          — centralized JSONL event logger
  components/
    ScreenRouter.tsx  — screen dispatch by state.currentScreen
    CardDisplay.tsx   — full card rendering with art, stats, rules text
    CardOverlay.tsx   — enlarged card overlay (hover/click)
    DeckViewer.tsx    — deck browser panel
    HUD.tsx           — persistent bottom bar
    card-text.ts      — rules text symbol substitution
    AtlasNode.tsx     — atlas graph node component
    SiteCard.tsx      — site icon card component
  screens/            — one file per screen (QuestStart, Atlas,
  |                     Dreamscape, site types, QuestComplete)
  atlas/
    atlas-generator.ts  — dream atlas + dreamscape generation
  draft/
    draft-engine.ts     — 10-seat cube draft (pick, rotate, bots)
  data/               — synthetic data (dreamcallers, dreamsigns,
  |                     journeys, offers, biomes)
  shop/               — shop item generation helpers
  transfiguration/    — transfiguration type logic
```

## Registering a New Site Type

Adding a site type requires changes in two places:

1. **Screen component** — create `src/screens/MySiteScreen.tsx` and add a case
   to `ScreenRouter.tsx`.
2. **Atlas pool** — add the site type with a weight to `buildAdditionalSitePool`
   in `src/atlas/atlas-generator.ts`. Without this step the site type is
   unreachable during gameplay.

## Extending QuestState

`QuestContextValue`, `QuestMutations`, and `QuestState` are defined in
`src/state/quest-context.tsx` and `src/types/quest.ts`. When multiple parallel
tasks need to add fields to these interfaces, they will conflict at merge time.
Prefer adding interface extensions in a preparatory commit on the task branch
before writing the screen that uses them.

## JSONL Event Logging

Every game event is logged via `logEvent(eventName, fields)` in
`src/logging.ts`. The logger appends to an in-memory array and writes to
`console.log` as single-line JSON. A "Download Log" button in the HUD exports
the full `.jsonl` file.

Reserved fields (`timestamp`, `event`, `seq`) are assigned by the logger and
cannot be overridden by caller-supplied `fields`. The spread order is
`{ timestamp, event, seq, ...fields }`.

Every mutation that changes game state must call `logEvent` before or after the
mutation. Missing log calls are a conformance blocker.

## Card Data Normalization

The TOML source has several field variants that `setup-assets.mjs` normalizes to
JSON:

| Field         | TOML source values           | JSON output    |
| ------------- | ---------------------------- | -------------- |
| `spark`       | absent, `""`, `"*"`, integer | `null` or int  |
| `energy-cost` | `"*"`, integer               | `null` or int  |
| `subtype`     | absent, string               | `""` or string |

The `"*"` value appears on 4 "Abomination" cards for spark and 2 cards for
energy cost. The absent `spark` key appears on some Event cards. Normalize all
three variants to `null`.

Keys are converted from TOML kebab-case to camelCase in JSON output.
