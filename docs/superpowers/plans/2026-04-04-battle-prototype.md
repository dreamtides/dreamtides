# Battle Prototype Web Client Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a playable web-based battle client that connects to the existing Rust dev server for prototyping Dreamtides battles against AI.

**Architecture:** React/Vite/TypeScript app at `scripts/battle_prototype/` that talks to the dev server on port 26598 via proxied HTTP. The server sends `BattleView` snapshots containing all card state, legal actions, and UI controls. The client renders zones, wires click handlers to embedded `GameAction` values, and polls for updates.

**Tech Stack:** Vite 7.x, React 19.x, TypeScript 5.x (strict), Tailwind CSS 4.x

**Design spec:** `docs/superpowers/specs/2026-04-04-battle-prototype-design.md`

---

## QA Requirements (MUST BE INCLUDED IN EVERY SUBAGENT PROMPT)

**Copy this block verbatim into every subagent prompt:**

> You MUST use `agent-browser` to take a screenshot and verify your work after every meaningful code change. You MUST NOT claim completion without screenshot evidence showing the feature works correctly. If a screenshot shows any issue, you MUST fix it before proceeding. When verifying game state, log the expected values from the server response and compare them against what is visible on screen. Take screenshots BEFORE and AFTER every user interaction (clicking a card, pressing a button) to verify state transitions. After implementing any interactive feature, play at least 2 full turns of a real battle to verify correctness.

---

## File Map

**New files to create:**

| File | Responsibility |
|------|---------------|
| `scripts/battle_prototype/package.json` | Dependencies and scripts |
| `scripts/battle_prototype/vite.config.ts` | Vite config with proxy to dev server |
| `scripts/battle_prototype/tsconfig.json` | TypeScript strict config |
| `scripts/battle_prototype/tsconfig.node.json` | Node-side TS config |
| `scripts/battle_prototype/index.html` | HTML entry point |
| `scripts/battle_prototype/eslint.config.js` | ESLint config |
| `scripts/battle_prototype/scripts/setup-assets.mjs` | Symlink card art, copy tide icons |
| `scripts/battle_prototype/src/main.tsx` | React DOM mount |
| `scripts/battle_prototype/src/App.tsx` | Root component, connect on mount |
| `scripts/battle_prototype/src/index.css` | Tailwind + dark theme CSS |
| `scripts/battle_prototype/src/types/battle.ts` | TypeScript types mirroring Rust display_data |
| `scripts/battle_prototype/src/api/client.ts` | HTTP client: connect, performAction, poll |
| `scripts/battle_prototype/src/state/battle-context.tsx` | React context for BattleView + polling |
| `scripts/battle_prototype/src/util/command-parser.ts` | Extract BattleView from CommandSequence |
| `scripts/battle_prototype/src/util/flex-node-parser.ts` | Extract text/buttons from FlexNode tree |
| `scripts/battle_prototype/src/components/BattleScreen.tsx` | Main layout container |
| `scripts/battle_prototype/src/components/PlayerStatus.tsx` | Score, energy, spark, deck/void counts |
| `scripts/battle_prototype/src/components/BattlefieldZone.tsx` | Row of up to 8 characters |
| `scripts/battle_prototype/src/components/StackZone.tsx` | Center column stack display |
| `scripts/battle_prototype/src/components/HandZone.tsx` | Bottom hand of cards |
| `scripts/battle_prototype/src/components/CardDisplay.tsx` | Single card rendering |
| `scripts/battle_prototype/src/components/ActionBar.tsx` | InterfaceView buttons |
| `scripts/battle_prototype/src/components/OverlayPrompt.tsx` | FlexNode text/button extraction |
| `scripts/battle_prototype/src/components/DebugPanel.tsx` | Debug controls |

---

## Task 1: Project Scaffold

**Files:**
- Create: `scripts/battle_prototype/package.json`
- Create: `scripts/battle_prototype/vite.config.ts`
- Create: `scripts/battle_prototype/tsconfig.json`
- Create: `scripts/battle_prototype/tsconfig.node.json`
- Create: `scripts/battle_prototype/index.html`
- Create: `scripts/battle_prototype/eslint.config.js`
- Create: `scripts/battle_prototype/src/main.tsx`
- Create: `scripts/battle_prototype/src/index.css`
- Create: `scripts/battle_prototype/.gitignore`

- [ ] **Step 1: Create package.json**

```json
{
  "name": "battle-prototype",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "node scripts/setup-assets.mjs && vite",
    "build": "node scripts/setup-assets.mjs && tsc && vite build",
    "preview": "vite preview",
    "lint": "eslint src/",
    "typecheck": "tsc --noEmit",
    "setup-assets": "node scripts/setup-assets.mjs"
  },
  "dependencies": {
    "react": "^19.1.0",
    "react-dom": "^19.1.0"
  },
  "devDependencies": {
    "@eslint/js": "^9.39.2",
    "@tailwindcss/vite": "^4.0.0",
    "@types/react": "^19.1.8",
    "@types/react-dom": "^19.1.6",
    "@typescript-eslint/eslint-plugin": "^8.54.0",
    "@typescript-eslint/parser": "^8.54.0",
    "@vitejs/plugin-react": "^4.6.0",
    "eslint": "^9.39.2",
    "smol-toml": "^1.3.1",
    "tailwindcss": "^4.0.0",
    "typescript": "~5.8.3",
    "typescript-eslint": "^8.54.0",
    "vite": "^7.0.4"
  }
}
```

- [ ] **Step 2: Create vite.config.ts with proxy**

```typescript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: {
    proxy: {
      "/connect": {
        target: "http://localhost:26598",
        changeOrigin: true,
      },
      "/perform_action": {
        target: "http://localhost:26598",
        changeOrigin: true,
      },
      "/poll": {
        target: "http://localhost:26598",
        changeOrigin: true,
      },
      "/log": {
        target: "http://localhost:26598",
        changeOrigin: true,
      },
    },
  },
});
```

- [ ] **Step 3: Create tsconfig.json**

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noImplicitAny": true,
    "strictNullChecks": true,
    "noImplicitReturns": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true
  },
  "include": ["src"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

- [ ] **Step 4: Create tsconfig.node.json**

```json
{
  "compilerOptions": {
    "composite": true,
    "skipLibCheck": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowSyntheticDefaultImports": true
  },
  "include": ["vite.config.ts"]
}
```

- [ ] **Step 5: Create index.html**

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Dreamtides Battle Prototype</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

- [ ] **Step 6: Create eslint.config.js**

```javascript
import eslint from "@eslint/js";
import tseslint from "typescript-eslint";

export default tseslint.config(
  eslint.configs.recommended,
  ...tseslint.configs.recommendedTypeChecked,
  {
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      "@typescript-eslint/no-unsafe-argument": "error",
      "@typescript-eslint/no-unsafe-assignment": "error",
      "@typescript-eslint/no-unsafe-call": "error",
      "@typescript-eslint/no-unsafe-member-access": "error",
      "@typescript-eslint/no-unsafe-return": "error",
      "@typescript-eslint/no-unused-vars": [
        "error",
        {
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
        },
      ],
    },
  },
  {
    ignores: ["node_modules/", "dist/", "eslint.config.js", "vite.config.ts"],
  }
);
```

- [ ] **Step 7: Create src/main.tsx**

```tsx
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App.tsx";
import "./index.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
```

- [ ] **Step 8: Create src/index.css**

```css
@import "tailwindcss";

:root {
  --color-bg: #0a0612;
  --color-primary: #7c3aed;
  --color-primary-light: #a855f7;
  --color-gold: #d4a017;
  --color-gold-light: #fbbf24;
  --color-text: #e2e8f0;
  --color-text-dim: #94a3b8;
  --color-surface: #1a1525;
  --color-surface-light: #2a2040;
  --color-border: #3a3050;
}

body {
  margin: 0;
  background-color: var(--color-bg);
  color: var(--color-text);
  font-family:
    system-ui,
    -apple-system,
    "Segoe UI",
    Roboto,
    "Helvetica Neue",
    Arial,
    sans-serif;
  min-height: 100vh;
}
```

- [ ] **Step 9: Create .gitignore**

```
node_modules/
dist/
public/cards/
public/tides/
public/card-data.json
logs/
```

- [ ] **Step 10: Create placeholder src/App.tsx**

```tsx
export default function App() {
  return (
    <div className="flex items-center justify-center min-h-screen">
      <p className="text-xl" style={{ color: "var(--color-text-dim)" }}>
        Battle Prototype — connecting...
      </p>
    </div>
  );
}
```

- [ ] **Step 11: Install dependencies and verify dev server starts**

Run: `cd scripts/battle_prototype && npm install && npm run dev`

Expected: Vite dev server starts on a local port (e.g., 5173). Opening the URL shows "Battle Prototype — connecting...".

- [ ] **Step 12: Commit**

```bash
git add scripts/battle_prototype/
git commit -m "feat: scaffold battle prototype Vite project with proxy config"
```

---

## Task 2: Asset Pipeline

**Files:**
- Create: `scripts/battle_prototype/scripts/setup-assets.mjs`

- [ ] **Step 1: Create setup-assets.mjs**

This is adapted from the quest prototype's script. It parses `rendered-cards.toml`, creates `card-data.json` (mapping card names/sprite addresses to card numbers for image lookup), symlinks card images, and copies tide icons.

```javascript
import { readFileSync, mkdirSync, rmSync, symlinkSync, copyFileSync, readdirSync, existsSync } from "node:fs";
import { writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { resolve, join } from "node:path";
import { homedir } from "node:os";
import { execSync } from "node:child_process";
import { parse } from "smol-toml";

const ROOT = resolve(import.meta.dirname, "..");
const PROJECT_ROOT = resolve(ROOT, "..", "..");
const IMAGE_CACHE_DIR = join(homedir(), "Library", "Caches", "io.github.dreamtides.tv", "image_cache");

const PUBLIC_DIR = join(ROOT, "public");
const CARDS_DIR = join(PUBLIC_DIR, "cards");
const TIDES_DIR = join(PUBLIC_DIR, "tides");
const JSON_PATH = join(PUBLIC_DIR, "card-data.json");

function findMainWorktreeRoot() {
  try {
    const gitCommonDir = execSync("git rev-parse --git-common-dir", {
      cwd: ROOT,
      encoding: "utf8",
    }).trim();
    const absGitDir = resolve(ROOT, gitCommonDir);
    return resolve(absGitDir, "..");
  } catch {
    return PROJECT_ROOT;
  }
}

function resolveAssetPath(...segments) {
  const localPath = join(PROJECT_ROOT, ...segments);
  if (existsSync(localPath)) {
    return localPath;
  }
  const mainRoot = findMainWorktreeRoot();
  const mainPath = join(mainRoot, ...segments);
  if (existsSync(mainPath)) {
    return mainPath;
  }
  return localPath;
}

function kebabToCamel(str) {
  return str.replace(/-([a-z])/g, (_, c) => c.toUpperCase());
}

function transformCard(card) {
  const result = {};
  for (const [key, value] of Object.entries(card)) {
    const camelKey = kebabToCamel(key);
    if (camelKey === "spark" || camelKey === "energyCost") {
      result[camelKey] = value === "" || value === "*" ? null : value;
    } else {
      result[camelKey] = value;
    }
  }
  if (!("spark" in result)) {
    result.spark = null;
  }
  if (!("subtype" in result) || result.subtype == null) {
    result.subtype = "";
  }
  return result;
}

function imageHash(imageNumber) {
  const url = `https://www.shutterstock.com/image-illustration/-260nw-${imageNumber}.jpg`;
  return createHash("sha256").update(url).digest("hex");
}

function recreateDir(dir) {
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dir, { recursive: true });
}

function main() {
  const tomlPath = resolveAssetPath("client", "Assets", "StreamingAssets", "Tabula", "rendered-cards.toml");
  const tideIconsDir = resolveAssetPath("client", "Assets", "ThirdParty", "GameAssets", "Tides");

  console.log("Parsing rendered-cards.toml...");
  const tomlContent = readFileSync(tomlPath, "utf8");
  const parsed = parse(tomlContent);
  const allCards = parsed.cards;

  if (!Array.isArray(allCards)) {
    throw new Error("Expected [[cards]] array in TOML file");
  }

  console.log(`Found ${allCards.length} total cards`);
  const cards = allCards.filter((c) => c.rarity !== "Special");
  console.log(`Filtered to ${cards.length} non-Special cards`);

  const jsonCards = cards.map(transformCard);

  mkdirSync(PUBLIC_DIR, { recursive: true });
  writeFileSync(JSON_PATH, JSON.stringify(jsonCards, null, 2) + "\n");
  console.log(`Wrote ${jsonCards.length} cards to card-data.json`);

  recreateDir(CARDS_DIR);
  let linked = 0;
  let missing = 0;

  for (const card of jsonCards) {
    const hash = imageHash(card.imageNumber);
    const cachePath = join(IMAGE_CACHE_DIR, hash);
    const symlinkPath = join(CARDS_DIR, `${card.cardNumber}.webp`);

    if (existsSync(cachePath)) {
      symlinkSync(cachePath, symlinkPath);
      linked++;
    } else {
      missing++;
    }
  }

  console.log(`Linked ${linked} of ${jsonCards.length} card images (${missing} missing)`);

  recreateDir(TIDES_DIR);

  if (!existsSync(tideIconsDir)) {
    console.warn("Warning: tide icons directory not found, skipping tide icon copy");
  } else {
    const tideFiles = readdirSync(tideIconsDir).filter(
      (f) => f.endsWith(".png") && !f.endsWith(".meta")
    );

    for (const file of tideFiles) {
      copyFileSync(join(tideIconsDir, file), join(TIDES_DIR, file));
    }

    console.log(`Copied ${tideFiles.length} tide icons to public/tides/`);
  }

  console.log("Asset setup complete.");
}

main();
```

- [ ] **Step 2: Run setup-assets and verify**

Run: `cd scripts/battle_prototype && npm run setup-assets`

Expected: Console output showing cards parsed, images linked, tide icons copied. Verify `public/card-data.json` exists and contains card entries. Verify `public/cards/` contains `.webp` symlinks.

- [ ] **Step 3: Commit**

```bash
git add scripts/battle_prototype/scripts/setup-assets.mjs
git commit -m "feat: add asset pipeline for card images and tide icons"
```

---

## Task 3: TypeScript Types

**Files:**
- Create: `scripts/battle_prototype/src/types/battle.ts`

This is the largest single file. It mirrors the Rust `display_data` types as TypeScript interfaces. All enums are externally tagged in serde (default) — unit variants serialize as strings, data variants as `{ "VariantName": data }`.

- [ ] **Step 1: Create types/battle.ts**

```typescript
// ============================================================
// Core primitives — Rust newtype wrappers serialize transparently
// ============================================================

/** Rust Energy(u32) — serializes as plain number */
export type Energy = number;
/** Rust Spark(u32) — serializes as plain number */
export type Spark = number;
/** Rust Points(u32) — serializes as plain number */
export type Points = number;
/** Rust TurnId(u32) — serializes as plain number */
export type TurnId = number;
/** Rust Milliseconds { milliseconds_value: u32 } */
export interface Milliseconds {
  milliseconds_value: number;
}
/** Rust UserId(Uuid) — serializes as UUID string */
export type UserId = string;
/** Rust BattleId(Uuid) — serializes as UUID string */
export type BattleId = string;
/** Rust ClientCardId = String */
export type ClientCardId = string;

// ============================================================
// Display types
// ============================================================

export interface DisplayColor {
  red: number;
  green: number;
  blue: number;
  alpha: number;
}

export interface SpriteAddress {
  sprite: string;
}

export interface PrefabAddress {
  prefab: string;
}

export interface AudioClipAddress {
  audio_clip: string;
}

export interface EffectAddress {
  effect: string;
}

// DisplayImage: externally tagged enum
// { "Sprite": { "sprite": "..." } } or { "Prefab": { ... } }
export type DisplayImage =
  | { Sprite: SpriteAddress }
  | { Prefab: { prefab: PrefabAddress; studio_type: string } };

// ============================================================
// Positions
// ============================================================

export type DisplayPlayer = "User" | "Enemy";

export type StackType =
  | "Default"
  | "TargetingUserBattlefield"
  | "TargetingEnemyBattlefield"
  | "TargetingBothBattlefields";

// Position: externally tagged enum — unit variants are strings,
// data variants are { "VariantName": data }
export type Position =
  | "Default"
  | "Offscreen"
  | { OnStack: StackType }
  | "Drawn"
  | { InHand: DisplayPlayer }
  | { InDeck: DisplayPlayer }
  | { InVoid: DisplayPlayer }
  | { InBanished: DisplayPlayer }
  | { OnBattlefield: DisplayPlayer }
  | { InPlayerStatus: DisplayPlayer }
  | "Browser"
  | { CardOrderSelector: string }
  | "HandStorage"
  | { InDreamwell: DisplayPlayer }
  | "DreamwellActivation"
  | "GameModifier"
  | "OnScreenStorage"
  | { AboveVoid: DisplayPlayer }
  | string; // catch-all for quest-specific positions

export interface ObjectPosition {
  position: Position;
  sorting_key: number;
}

// ============================================================
// Card types
// ============================================================

export type CardFacing = "FaceUp" | "FaceDown";

export type CardPrefab =
  | "Character"
  | "Event"
  | "Identity"
  | "Token"
  | "Dreamwell"
  | "Enemy"
  | "Dreamsign"
  | "IconCard"
  | "Journey"
  | "OfferCost";

// GameAction: externally tagged enum
// Unit variants: "NoOp", "PassPriority", etc.
// Data variants: { "BattleAction": { "PlayCardFromHand": ... } }
// We use a loose type since actions are opaque — we receive them from
// the server and send them back without inspecting internals.
export type GameAction = unknown;

export interface CardActions {
  can_play?: GameAction;
  can_select_order?: unknown;
  on_play_sound?: AudioClipAddress;
  on_click?: GameAction;
  play_effect_preview?: unknown;
  button_attachment?: ButtonView;
}

export interface CardEffects {
  looping_effect?: EffectAddress;
  reverse_dissolve_on_appear?: unknown;
}

export interface RevealedCardView {
  image: DisplayImage;
  name: string;
  cost?: string;
  produced?: string;
  spark?: string;
  card_type: string;
  rules_text: string;
  outline_color?: DisplayColor;
  is_fast: boolean;
  actions: CardActions;
  effects: CardEffects;
  info_zoom_data?: unknown;
}

export interface CardView {
  id: ClientCardId;
  position: ObjectPosition;
  revealed?: RevealedCardView;
  revealed_to_opponents: boolean;
  card_facing: CardFacing;
  backless: boolean;
  create_position?: ObjectPosition;
  create_sound?: AudioClipAddress;
  destroy_position?: ObjectPosition;
  prefab: CardPrefab;
}

// ============================================================
// Player and battle view
// ============================================================

export type DisplayedTurnIndicator = "Left" | "Right";

export interface PlayerView {
  score: Points;
  can_act: boolean;
  energy: Energy;
  produced_energy: Energy;
  total_spark: Spark;
  turn_indicator?: DisplayedTurnIndicator;
  is_victory_imminent: boolean;
}

export interface ButtonView {
  label: string;
  action?: GameAction;
}

export interface CardBrowserView {
  close_button?: GameAction;
}

export interface CardOrderSelectorView {
  include_deck: boolean;
  include_void: boolean;
}

export interface InterfaceView {
  has_open_panels: boolean;
  screen_overlay?: FlexNode;
  primary_action_button?: ButtonView;
  primary_action_show_on_idle_duration?: Milliseconds;
  secondary_action_button?: ButtonView;
  increment_button?: ButtonView;
  decrement_button?: ButtonView;
  dev_button?: ButtonView;
  undo_button?: ButtonView;
  browser?: CardBrowserView;
  card_order_selector?: CardOrderSelectorView;
}

export interface DisplayArrow {
  source: unknown;
  target: unknown;
  color: string;
}

export type BattlePreviewState = "None" | "Pending" | { Active: unknown };

export interface BattleView {
  id: BattleId;
  user: PlayerView;
  enemy: PlayerView;
  cards: CardView[];
  interface: InterfaceView;
  arrows: DisplayArrow[];
  preview: BattlePreviewState;
  turn_number: TurnId;
}

// ============================================================
// FlexNode (for overlay parsing)
// ============================================================

export interface TextNode {
  label: string;
}

export type NodeType =
  | { Text: TextNode }
  | { TypewriterTextNode: { label: string } }
  | { ScrollViewNode: unknown }
  | { DraggableNode: unknown }
  | { TextFieldNode: unknown }
  | { SliderNode: unknown };

export interface EventHandlers {
  on_click?: GameAction;
  on_long_press?: GameAction;
  on_mouse_enter?: GameAction;
  on_mouse_leave?: GameAction;
  on_mouse_down?: GameAction;
  on_mouse_up?: GameAction;
  on_field_changed?: GameAction;
}

export interface FlexNode {
  name?: string;
  node_type?: NodeType;
  children: FlexNode[];
  event_handlers?: EventHandlers;
  style?: unknown;
  hover_style?: unknown;
  pressed_style?: unknown;
  on_attach_style?: unknown;
  on_attach_style_duration?: Milliseconds;
}

// ============================================================
// Commands
// ============================================================

export interface UpdateBattleCommand {
  battle: BattleView;
  update_sound?: AudioClipAddress;
}

// Command: externally tagged enum. We only care about UpdateBattle.
export type Command =
  | { UpdateBattle: UpdateBattleCommand }
  | Record<string, unknown>;

export interface ParallelCommandGroup {
  commands: Command[];
}

export interface CommandSequence {
  groups: ParallelCommandGroup[];
}

// ============================================================
// Request / Response types
// ============================================================

export type PollResponseType = "None" | "Incremental" | "Final";

export interface Metadata {
  user_id: UserId;
  battle_id?: BattleId;
  request_id?: string;
  integration_test_id?: string;
}

export interface ConnectRequest {
  metadata: Metadata;
  persistent_data_path: string;
  streaming_assets_path: string;
  vs_opponent?: UserId;
  display_properties?: unknown;
  debug_configuration?: DebugConfiguration;
}

export interface DebugConfiguration {
  enemy?: unknown;
  seed?: number;
  deck_override?: TestDeckName;
  dreamwell_override?: unknown;
}

export type TestDeckName = "Vanilla" | "StartingFive" | "Benchmark1" | "Core11";

export interface ConnectResponse {
  metadata: Metadata;
  commands: CommandSequence;
  response_version: string;
}

export interface PerformActionRequest {
  metadata: Metadata;
  action: GameAction;
  save_file_id?: UserId;
  last_response_version?: string;
}

export interface PerformActionResponse {
  metadata: Metadata;
  commands: CommandSequence;
}

export interface PollRequest {
  metadata: Metadata;
}

export interface PollResponse {
  metadata: Metadata;
  commands?: CommandSequence;
  response_type: PollResponseType;
  response_version?: string;
}
```

- [ ] **Step 2: Verify types compile**

Run: `cd scripts/battle_prototype && npx tsc --noEmit`

Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add scripts/battle_prototype/src/types/battle.ts
git commit -m "feat: add TypeScript types mirroring Rust display_data"
```

---

## Task 4: API Client & Command Parser

**Files:**
- Create: `scripts/battle_prototype/src/api/client.ts`
- Create: `scripts/battle_prototype/src/util/command-parser.ts`

- [ ] **Step 1: Create util/command-parser.ts**

```typescript
import type { BattleView, Command, CommandSequence } from "../types/battle";

/**
 * Extract the last BattleView from a CommandSequence.
 * Scans all groups/commands for UpdateBattle, returns the last one found.
 */
export function extractBattleView(commands: CommandSequence): BattleView | null {
  let lastBattle: BattleView | null = null;
  for (const group of commands.groups) {
    for (const cmd of group.commands) {
      if (isUpdateBattle(cmd)) {
        lastBattle = cmd.UpdateBattle.battle;
      }
    }
  }
  return lastBattle;
}

function isUpdateBattle(
  cmd: Command,
): cmd is { UpdateBattle: { battle: BattleView; update_sound?: unknown } } {
  return "UpdateBattle" in cmd;
}
```

- [ ] **Step 2: Create api/client.ts**

```typescript
import type {
  ConnectResponse,
  PerformActionResponse,
  PollResponse,
  GameAction,
  TestDeckName,
  Metadata,
} from "../types/battle";

const USER_ID = "00000000-0000-0000-0000-000000000001";

function metadata(): Metadata {
  return { user_id: USER_ID };
}

export async function connect(
  deckOverride?: TestDeckName,
): Promise<ConnectResponse> {
  const body = JSON.stringify({
    metadata: metadata(),
    persistent_data_path: "",
    streaming_assets_path: "",
    debug_configuration: deckOverride
      ? { deck_override: deckOverride }
      : undefined,
  });
  const res = await fetch("/connect", {
    method: "GET",
    headers: { "Content-Type": "application/json" },
    body,
  });
  if (!res.ok) {
    throw new Error(`connect failed: ${res.status} ${await res.text()}`);
  }
  return (await res.json()) as ConnectResponse;
}

export async function performAction(
  action: GameAction,
  lastResponseVersion?: string,
): Promise<PerformActionResponse> {
  const body = JSON.stringify({
    metadata: metadata(),
    action,
    last_response_version: lastResponseVersion,
  });
  const res = await fetch("/perform_action", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body,
  });
  if (!res.ok) {
    throw new Error(
      `perform_action failed: ${res.status} ${await res.text()}`,
    );
  }
  return (await res.json()) as PerformActionResponse;
}

export async function poll(): Promise<PollResponse> {
  const body = JSON.stringify({ metadata: metadata() });
  const res = await fetch("/poll", {
    method: "GET",
    headers: { "Content-Type": "application/json" },
    body,
  });
  if (!res.ok) {
    throw new Error(`poll failed: ${res.status} ${await res.text()}`);
  }
  return (await res.json()) as PollResponse;
}
```

**Note on GET with body:** The dev server's `connect` and `poll` handlers accept JSON body on GET requests (the Axum handler uses `body: String`). The `fetch` API supports this. If any browser restricts it, the subagent should switch to POST and update the Vite proxy config accordingly — but this matches how the Unity client works.

- [ ] **Step 3: Verify types compile**

Run: `cd scripts/battle_prototype && npx tsc --noEmit`

Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git add scripts/battle_prototype/src/api/ scripts/battle_prototype/src/util/command-parser.ts
git commit -m "feat: add API client and command parser"
```

---

## Task 5: Battle Context & App Connection

**Files:**
- Create: `scripts/battle_prototype/src/state/battle-context.tsx`
- Modify: `scripts/battle_prototype/src/App.tsx`

- [ ] **Step 1: Create state/battle-context.tsx**

```tsx
import {
  createContext,
  useCallback,
  useContext,
  useRef,
  useState,
  type ReactNode,
} from "react";
import type {
  BattleView,
  GameAction,
  TestDeckName,
} from "../types/battle";
import * as api from "../api/client";
import { extractBattleView } from "../util/command-parser";

interface BattleContextValue {
  battle: BattleView | null;
  isPolling: boolean;
  error: string | null;
  sendAction: (action: GameAction) => void;
  reconnect: (deck?: TestDeckName) => void;
}

const BattleContext = createContext<BattleContextValue | null>(null);

export function useBattle(): BattleContextValue {
  const ctx = useContext(BattleContext);
  if (!ctx) throw new Error("useBattle must be used within BattleProvider");
  return ctx;
}

const POLL_INTERVAL_MS = 200;

export function BattleProvider({ children }: { children: ReactNode }) {
  const [battle, setBattle] = useState<BattleView | null>(null);
  const [isPolling, setIsPolling] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const responseVersionRef = useRef<string | undefined>(undefined);

  const startPolling = useCallback(() => {
    setIsPolling(true);
    const interval = setInterval(() => {
      void (async () => {
        try {
          const pollRes = await api.poll();
          if (pollRes.commands) {
            const view = extractBattleView(pollRes.commands);
            if (view) setBattle(view);
          }
          if (pollRes.response_version) {
            responseVersionRef.current = pollRes.response_version;
          }
          if (pollRes.response_type === "Final") {
            clearInterval(interval);
            setIsPolling(false);
          }
        } catch (e) {
          clearInterval(interval);
          setIsPolling(false);
          setError(e instanceof Error ? e.message : "Poll failed");
        }
      })();
    }, POLL_INTERVAL_MS);
  }, []);

  const sendAction = useCallback(
    (action: GameAction) => {
      if (isPolling) return;
      void (async () => {
        try {
          setError(null);
          const res = await api.performAction(
            action,
            responseVersionRef.current,
          );
          const view = extractBattleView(res.commands);
          if (view) setBattle(view);
          startPolling();
        } catch (e) {
          setError(e instanceof Error ? e.message : "Action failed");
        }
      })();
    },
    [isPolling, startPolling],
  );

  const reconnect = useCallback(
    (deck?: TestDeckName) => {
      void (async () => {
        try {
          setError(null);
          setIsPolling(false);
          const res = await api.connect(deck);
          responseVersionRef.current = res.response_version;
          const view = extractBattleView(res.commands);
          if (view) setBattle(view);
          startPolling();
        } catch (e) {
          setError(e instanceof Error ? e.message : "Connect failed");
        }
      })();
    },
    [startPolling],
  );

  return (
    <BattleContext.Provider
      value={{ battle, isPolling, error, sendAction, reconnect }}
    >
      {children}
    </BattleContext.Provider>
  );
}
```

- [ ] **Step 2: Update App.tsx to connect on mount and show raw state**

```tsx
import { useEffect } from "react";
import { BattleProvider, useBattle } from "./state/battle-context.tsx";

function BattleApp() {
  const { battle, isPolling, error, reconnect } = useBattle();

  useEffect(() => {
    reconnect("Benchmark1");
  }, [reconnect]);

  if (error) {
    return (
      <div className="p-4">
        <p className="text-red-400">Error: {error}</p>
        <button
          className="mt-2 px-4 py-2 rounded"
          style={{ background: "var(--color-primary)" }}
          onClick={() => reconnect("Benchmark1")}
        >
          Retry
        </button>
      </div>
    );
  }

  if (!battle) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <p style={{ color: "var(--color-text-dim)" }}>Connecting...</p>
      </div>
    );
  }

  return (
    <div className="p-4">
      <h1 className="text-lg font-bold mb-2">
        Battle Prototype {isPolling ? "(polling...)" : ""}
      </h1>
      <div className="grid grid-cols-2 gap-4 mb-4">
        <div>
          <h2 className="font-bold">You</h2>
          <p>Score: {battle.user.score} | Energy: {battle.user.energy}/{battle.user.produced_energy} | Spark: {battle.user.total_spark}</p>
        </div>
        <div>
          <h2 className="font-bold">Enemy</h2>
          <p>Score: {battle.enemy.score} | Energy: {battle.enemy.energy}/{battle.enemy.produced_energy} | Spark: {battle.enemy.total_spark}</p>
        </div>
      </div>
      <p>Turn: {battle.turn_number} | Cards: {battle.cards.length}</p>
      <pre className="mt-4 text-xs overflow-auto max-h-96 p-2 rounded"
           style={{ background: "var(--color-surface)" }}>
        {JSON.stringify(battle, null, 2)}
      </pre>
    </div>
  );
}

export default function App() {
  return (
    <BattleProvider>
      <BattleApp />
    </BattleProvider>
  );
}
```

- [ ] **Step 3: Start the Rust dev server and verify connection**

Run (in a separate terminal): `cd rules_engine && just dev-server`

Then open the battle prototype in the browser. Verify: page shows "You" and "Enemy" stats, turn number, card count, and raw JSON.

**QA:** Use `agent-browser` to open the battle prototype URL. Take a screenshot. Verify the BattleView JSON is displayed with valid data (cards array is non-empty, scores are 0, energy > 0, turn_number is 1). Reload the page and take another screenshot to verify reconnection works.

- [ ] **Step 4: Commit**

```bash
git add scripts/battle_prototype/src/state/ scripts/battle_prototype/src/App.tsx
git commit -m "feat: battle context with connect, poll, and raw state display"
```

---

## Task 6: Card Display Component

**Files:**
- Create: `scripts/battle_prototype/src/components/CardDisplay.tsx`

- [ ] **Step 1: Create components/CardDisplay.tsx**

```tsx
import type { CardView, DisplayColor, GameAction } from "../types/battle";

interface CardDisplayProps {
  card: CardView;
  onAction?: (action: GameAction) => void;
  disabled?: boolean;
  compact?: boolean;
}

function colorToCSS(c: DisplayColor): string {
  return `rgba(${Math.round(c.red * 255)}, ${Math.round(c.green * 255)}, ${Math.round(c.blue * 255)}, ${c.alpha})`;
}

function getCardImageUrl(card: CardView): string | null {
  if (!card.revealed) return null;
  const img = card.revealed.image;
  if ("Sprite" in img) {
    // Sprite address format: "Cards/123" or similar — extract number
    const sprite = img.Sprite.sprite;
    const match = /(\d+)/.exec(sprite);
    if (match) {
      return `/cards/${match[1]}.webp`;
    }
  }
  return null;
}

export function CardDisplay({
  card,
  onAction,
  disabled,
  compact,
}: CardDisplayProps) {
  const revealed = card.revealed;
  const isFaceDown = card.card_facing === "FaceDown" || !revealed;

  const clickAction = revealed?.actions.can_play ?? revealed?.actions.on_click;
  const isClickable = !disabled && clickAction != null;

  const outlineColor = revealed?.outline_color
    ? colorToCSS(revealed.outline_color)
    : "var(--color-border)";

  const handleClick = () => {
    if (isClickable && clickAction && onAction) {
      onAction(clickAction);
    }
  };

  if (isFaceDown) {
    return (
      <div
        className="rounded flex items-center justify-center text-xs"
        style={{
          width: compact ? 60 : 120,
          height: compact ? 36 : 80,
          background: "var(--color-surface)",
          border: "1px solid var(--color-border)",
          color: "var(--color-text-dim)",
        }}
      >
        {card.prefab}
      </div>
    );
  }

  const imageUrl = getCardImageUrl(card);

  return (
    <div
      onClick={handleClick}
      className="rounded overflow-hidden flex flex-col"
      style={{
        width: compact ? 100 : 140,
        minHeight: compact ? 60 : 180,
        background: "var(--color-surface)",
        border: `2px solid ${outlineColor}`,
        cursor: isClickable ? "pointer" : "default",
        opacity: disabled ? 0.5 : 1,
      }}
    >
      {imageUrl && (
        <img
          src={imageUrl}
          alt={revealed.name}
          className="w-full object-cover"
          style={{ height: compact ? 40 : 80 }}
        />
      )}
      <div className="p-1 flex flex-col gap-0.5" style={{ fontSize: compact ? 9 : 11 }}>
        <div className="flex justify-between items-center">
          <span className="font-bold truncate" style={{ maxWidth: "70%" }}>
            {revealed.name}
          </span>
          {revealed.cost != null && (
            <span style={{ color: "var(--color-gold)" }}>{revealed.cost}</span>
          )}
        </div>
        <div className="flex justify-between" style={{ color: "var(--color-text-dim)", fontSize: compact ? 8 : 9 }}>
          <span>{revealed.card_type}</span>
          {revealed.spark != null && <span>⍏{revealed.spark}</span>}
        </div>
        {revealed.is_fast && (
          <span style={{ color: "var(--color-gold-light)", fontSize: compact ? 7 : 8 }}>
            ↯ Fast
          </span>
        )}
        {!compact && revealed.rules_text && (
          <div
            className="mt-1"
            style={{
              fontSize: 9,
              color: "var(--color-text-dim)",
              lineHeight: 1.3,
            }}
            dangerouslySetInnerHTML={{ __html: revealed.rules_text }}
          />
        )}
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Verify types compile**

Run: `cd scripts/battle_prototype && npx tsc --noEmit`

Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add scripts/battle_prototype/src/components/CardDisplay.tsx
git commit -m "feat: card display component with art, stats, and click handling"
```

---

## Task 7: Zone Components & Layout

**Files:**
- Create: `scripts/battle_prototype/src/components/PlayerStatus.tsx`
- Create: `scripts/battle_prototype/src/components/BattlefieldZone.tsx`
- Create: `scripts/battle_prototype/src/components/StackZone.tsx`
- Create: `scripts/battle_prototype/src/components/HandZone.tsx`
- Create: `scripts/battle_prototype/src/components/ActionBar.tsx`
- Create: `scripts/battle_prototype/src/components/BattleScreen.tsx`
- Modify: `scripts/battle_prototype/src/App.tsx`

- [ ] **Step 1: Create components/PlayerStatus.tsx**

```tsx
import type { PlayerView } from "../types/battle";

interface PlayerStatusProps {
  player: PlayerView;
  label: string;
  deckCount: number;
  voidCount: number;
  banishedCount: number;
}

export function PlayerStatus({
  player,
  label,
  deckCount,
  voidCount,
  banishedCount,
}: PlayerStatusProps) {
  return (
    <div
      className="flex items-center justify-between px-4 py-2"
      style={{
        background: "var(--color-surface)",
        borderBottom: "1px solid var(--color-border)",
      }}
    >
      <span className="font-bold">{label}</span>
      <div className="flex gap-4 text-sm">
        <span>
          Score:{" "}
          <span style={{ color: "var(--color-gold)" }}>{player.score}</span>
        </span>
        <span>
          Energy:{" "}
          <span style={{ color: "var(--color-primary-light)" }}>
            {player.energy}/{player.produced_energy}
          </span>
        </span>
        <span>
          Spark:{" "}
          <span style={{ color: "var(--color-gold-light)" }}>
            {player.total_spark}
          </span>
        </span>
        <span style={{ color: "var(--color-text-dim)" }}>
          Deck: {deckCount}
        </span>
        <span style={{ color: "var(--color-text-dim)" }}>
          Void: {voidCount}
        </span>
        {banishedCount > 0 && (
          <span style={{ color: "var(--color-text-dim)" }}>
            Banished: {banishedCount}
          </span>
        )}
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Create components/BattlefieldZone.tsx**

```tsx
import type { CardView, GameAction } from "../types/battle";
import { CardDisplay } from "./CardDisplay";

interface BattlefieldZoneProps {
  cards: CardView[];
  onAction: (action: GameAction) => void;
  disabled: boolean;
}

export function BattlefieldZone({
  cards,
  onAction,
  disabled,
}: BattlefieldZoneProps) {
  const sorted = [...cards].sort((a, b) => a.position.sorting_key - b.position.sorting_key);
  return (
    <div
      className="flex gap-2 justify-center items-center py-2 min-h-[100px]"
    >
      {sorted.length === 0 && (
        <span style={{ color: "var(--color-text-dim)", fontSize: 12 }}>
          No characters
        </span>
      )}
      {sorted.map((card) => (
        <CardDisplay
          key={card.id}
          card={card}
          onAction={onAction}
          disabled={disabled}
          compact
        />
      ))}
    </div>
  );
}
```

- [ ] **Step 3: Create components/StackZone.tsx**

```tsx
import type { CardView, GameAction } from "../types/battle";
import { CardDisplay } from "./CardDisplay";

interface StackZoneProps {
  cards: CardView[];
  onAction: (action: GameAction) => void;
  disabled: boolean;
}

export function StackZone({ cards, onAction, disabled }: StackZoneProps) {
  if (cards.length === 0) return null;

  const sorted = [...cards].sort(
    (a, b) => b.position.sorting_key - a.position.sorting_key,
  );

  return (
    <div
      className="flex gap-2 justify-center items-center py-2 px-4 mx-auto rounded"
      style={{
        border: "2px solid var(--color-gold)",
        background: "rgba(212, 160, 23, 0.05)",
        maxWidth: "80%",
      }}
    >
      <span
        className="text-xs font-bold mr-2"
        style={{ color: "var(--color-gold)" }}
      >
        STACK
      </span>
      {sorted.map((card, i) => (
        <div key={card.id} className="relative">
          {i === 0 && (
            <div
              className="absolute -top-3 left-1/2 -translate-x-1/2 text-[8px] px-1 rounded"
              style={{ background: "var(--color-gold)", color: "#000" }}
            >
              newest
            </div>
          )}
          <CardDisplay
            card={card}
            onAction={onAction}
            disabled={disabled}
            compact
          />
        </div>
      ))}
    </div>
  );
}
```

- [ ] **Step 4: Create components/HandZone.tsx**

```tsx
import type { CardView, GameAction } from "../types/battle";
import { CardDisplay } from "./CardDisplay";

interface HandZoneProps {
  cards: CardView[];
  onAction: (action: GameAction) => void;
  disabled: boolean;
}

export function HandZone({ cards, onAction, disabled }: HandZoneProps) {
  const sorted = [...cards].sort(
    (a, b) => a.position.sorting_key - b.position.sorting_key,
  );

  return (
    <div className="flex gap-2 justify-center items-end py-2 flex-wrap">
      {sorted.map((card) => (
        <CardDisplay
          key={card.id}
          card={card}
          onAction={onAction}
          disabled={disabled}
        />
      ))}
    </div>
  );
}
```

- [ ] **Step 5: Create components/ActionBar.tsx**

```tsx
import type { ButtonView, GameAction } from "../types/battle";

interface ActionBarProps {
  primaryButton?: ButtonView;
  secondaryButton?: ButtonView;
  undoButton?: ButtonView;
  devButton?: ButtonView;
  incrementButton?: ButtonView;
  decrementButton?: ButtonView;
  onAction: (action: GameAction) => void;
  disabled: boolean;
}

function ActionButton({
  button,
  onAction,
  disabled,
  primary,
}: {
  button: ButtonView;
  onAction: (action: GameAction) => void;
  disabled: boolean;
  primary?: boolean;
}) {
  const isDisabled = disabled || button.action == null;
  return (
    <button
      onClick={() => {
        if (!isDisabled && button.action != null) onAction(button.action);
      }}
      disabled={isDisabled}
      className="px-4 py-2 rounded text-sm font-bold"
      style={{
        background: primary
          ? "var(--color-primary)"
          : "var(--color-surface-light)",
        color: isDisabled ? "var(--color-text-dim)" : "var(--color-text)",
        cursor: isDisabled ? "not-allowed" : "pointer",
        opacity: isDisabled ? 0.5 : 1,
        border: "1px solid var(--color-border)",
      }}
    >
      {button.label}
    </button>
  );
}

export function ActionBar({
  primaryButton,
  secondaryButton,
  undoButton,
  devButton,
  incrementButton,
  decrementButton,
  onAction,
  disabled,
}: ActionBarProps) {
  const hasButtons =
    primaryButton ?? secondaryButton ?? undoButton ?? devButton
    ?? incrementButton ?? decrementButton;
  if (!hasButtons) return null;

  return (
    <div
      className="flex gap-2 justify-center items-center py-2 px-4"
      style={{
        background: "var(--color-surface)",
        borderTop: "1px solid var(--color-border)",
      }}
    >
      {incrementButton && (
        <ActionButton button={incrementButton} onAction={onAction} disabled={disabled} />
      )}
      {decrementButton && (
        <ActionButton button={decrementButton} onAction={onAction} disabled={disabled} />
      )}
      {secondaryButton && (
        <ActionButton button={secondaryButton} onAction={onAction} disabled={disabled} />
      )}
      {primaryButton && (
        <ActionButton
          button={primaryButton}
          onAction={onAction}
          disabled={disabled}
          primary
        />
      )}
      {undoButton && (
        <ActionButton button={undoButton} onAction={onAction} disabled={disabled} />
      )}
      {devButton && (
        <ActionButton button={devButton} onAction={onAction} disabled={disabled} />
      )}
    </div>
  );
}
```

- [ ] **Step 6: Create components/BattleScreen.tsx**

```tsx
import type { BattleView, CardView, DisplayPlayer, GameAction } from "../types/battle";
import { PlayerStatus } from "./PlayerStatus";
import { BattlefieldZone } from "./BattlefieldZone";
import { StackZone } from "./StackZone";
import { HandZone } from "./HandZone";
import { ActionBar } from "./ActionBar";

interface BattleScreenProps {
  battle: BattleView;
  onAction: (action: GameAction) => void;
  disabled: boolean;
}

function cardsByPosition(cards: CardView[], position: string, player?: DisplayPlayer): CardView[] {
  return cards.filter((c) => {
    const pos = c.position.position;
    if (typeof pos === "string") return false;
    if (position in pos) {
      if (player === undefined) return true;
      return (pos as Record<string, unknown>)[position] === player;
    }
    return false;
  });
}

function countCards(cards: CardView[], position: string, player: DisplayPlayer): number {
  return cardsByPosition(cards, position, player).length;
}

function stackCards(cards: CardView[]): CardView[] {
  return cards.filter((c) => {
    const pos = c.position.position;
    return typeof pos !== "string" && "OnStack" in pos;
  });
}

export function BattleScreen({ battle, onAction, disabled }: BattleScreenProps) {
  const ui = battle.interface;

  return (
    <div className="flex flex-col min-h-screen">
      {/* Turn info */}
      <div
        className="text-center py-1 text-sm"
        style={{
          background: "var(--color-surface)",
          borderBottom: "1px solid var(--color-border)",
          color: "var(--color-text-dim)",
        }}
      >
        Turn {battle.turn_number}
        {disabled && " — waiting..."}
      </div>

      {/* Enemy status */}
      <PlayerStatus
        player={battle.enemy}
        label="Enemy"
        deckCount={countCards(battle.cards, "InDeck", "Enemy")}
        voidCount={countCards(battle.cards, "InVoid", "Enemy")}
        banishedCount={countCards(battle.cards, "InBanished", "Enemy")}
      />

      {/* Enemy battlefield */}
      <BattlefieldZone
        cards={cardsByPosition(battle.cards, "OnBattlefield", "Enemy")}
        onAction={onAction}
        disabled={disabled}
      />

      {/* Stack */}
      <StackZone
        cards={stackCards(battle.cards)}
        onAction={onAction}
        disabled={disabled}
      />

      {/* User battlefield */}
      <BattlefieldZone
        cards={cardsByPosition(battle.cards, "OnBattlefield", "User")}
        onAction={onAction}
        disabled={disabled}
      />

      {/* User status */}
      <PlayerStatus
        player={battle.user}
        label="You"
        deckCount={countCards(battle.cards, "InDeck", "User")}
        voidCount={countCards(battle.cards, "InVoid", "User")}
        banishedCount={countCards(battle.cards, "InBanished", "User")}
      />

      {/* Hand */}
      <HandZone
        cards={cardsByPosition(battle.cards, "InHand", "User")}
        onAction={onAction}
        disabled={disabled}
      />

      {/* Action buttons */}
      <ActionBar
        primaryButton={ui.primary_action_button ?? undefined}
        secondaryButton={ui.secondary_action_button ?? undefined}
        undoButton={ui.undo_button ?? undefined}
        devButton={ui.dev_button ?? undefined}
        incrementButton={ui.increment_button ?? undefined}
        decrementButton={ui.decrement_button ?? undefined}
        onAction={onAction}
        disabled={disabled}
      />
    </div>
  );
}
```

- [ ] **Step 7: Update App.tsx to use BattleScreen**

```tsx
import { useEffect } from "react";
import { BattleProvider, useBattle } from "./state/battle-context.tsx";
import { BattleScreen } from "./components/BattleScreen.tsx";

function BattleApp() {
  const { battle, isPolling, error, sendAction, reconnect } = useBattle();

  useEffect(() => {
    reconnect("Benchmark1");
  }, [reconnect]);

  if (error) {
    return (
      <div className="p-4">
        <p className="text-red-400">Error: {error}</p>
        <button
          className="mt-2 px-4 py-2 rounded"
          style={{ background: "var(--color-primary)" }}
          onClick={() => reconnect("Benchmark1")}
        >
          Retry
        </button>
      </div>
    );
  }

  if (!battle) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <p style={{ color: "var(--color-text-dim)" }}>Connecting...</p>
      </div>
    );
  }

  return (
    <BattleScreen
      battle={battle}
      onAction={sendAction}
      disabled={isPolling}
    />
  );
}

export default function App() {
  return (
    <BattleProvider>
      <BattleApp />
    </BattleProvider>
  );
}
```

- [ ] **Step 8: Verify everything compiles and renders**

Run: `cd scripts/battle_prototype && npx tsc --noEmit`

Expected: No errors.

**QA:** Use `agent-browser` to open the app. Take a screenshot. Verify:
- Enemy status bar shows score, energy, spark, deck/void counts
- Enemy battlefield shows character cards (if any at start)
- Stack zone appears only if cards are on the stack
- Your battlefield shows your characters (if any)
- Your status bar shows your stats
- Your hand shows cards along the bottom
- Action buttons appear at the very bottom
- Card names, costs, and spark values are visible

Count cards in each zone and compare against the BattleView data. Take at least 2 screenshots.

- [ ] **Step 9: Commit**

```bash
git add scripts/battle_prototype/src/
git commit -m "feat: full battle layout with zones, cards, status bars, and action buttons"
```

---

## Task 8: FlexNode Overlay & Overlay Prompt

**Files:**
- Create: `scripts/battle_prototype/src/util/flex-node-parser.ts`
- Create: `scripts/battle_prototype/src/components/OverlayPrompt.tsx`
- Modify: `scripts/battle_prototype/src/components/BattleScreen.tsx`

- [ ] **Step 1: Create util/flex-node-parser.ts**

```typescript
import type { FlexNode, GameAction, NodeType } from "../types/battle";

export interface ExtractedButton {
  label: string;
  action: GameAction;
}

export interface ExtractedOverlay {
  texts: string[];
  buttons: ExtractedButton[];
}

function getTextFromNodeType(nodeType: NodeType): string | null {
  if ("Text" in nodeType) return nodeType.Text.label;
  if ("TypewriterTextNode" in nodeType) return nodeType.TypewriterTextNode.label;
  return null;
}

function extractFromNode(
  node: FlexNode,
  result: ExtractedOverlay,
): void {
  // Extract text from this node
  if (node.node_type) {
    const text = getTextFromNodeType(node.node_type);
    if (text && text.trim()) {
      result.texts.push(text.trim());
    }
  }

  // Extract click handler as a button
  if (node.event_handlers?.on_click) {
    // Find label from child TextNodes
    let label = "";
    for (const child of node.children) {
      if (child.node_type) {
        const text = getTextFromNodeType(child.node_type);
        if (text) {
          label = text;
          break;
        }
      }
    }
    if (label) {
      result.buttons.push({
        label,
        action: node.event_handlers.on_click,
      });
      // Don't recurse into button children (already extracted label)
      return;
    }
  }

  // Recurse into children
  for (const child of node.children) {
    extractFromNode(child, result);
  }
}

export function extractOverlayContent(
  overlay: FlexNode,
): ExtractedOverlay | null {
  const result: ExtractedOverlay = { texts: [], buttons: [] };
  extractFromNode(overlay, result);
  if (result.texts.length === 0 && result.buttons.length === 0) return null;
  return result;
}
```

- [ ] **Step 2: Create components/OverlayPrompt.tsx**

```tsx
import type { FlexNode, GameAction } from "../types/battle";
import { extractOverlayContent } from "../util/flex-node-parser";

interface OverlayPromptProps {
  overlay: FlexNode;
  onAction: (action: GameAction) => void;
  disabled: boolean;
}

export function OverlayPrompt({
  overlay,
  onAction,
  disabled,
}: OverlayPromptProps) {
  const content = extractOverlayContent(overlay);
  if (!content) return null;

  return (
    <div
      className="fixed inset-0 flex items-center justify-center z-50"
      style={{ background: "rgba(0, 0, 0, 0.7)" }}
    >
      <div
        className="rounded-lg p-6 max-w-lg w-full mx-4 flex flex-col gap-4"
        style={{
          background: "var(--color-surface)",
          border: "1px solid var(--color-border)",
        }}
      >
        {content.texts.map((text, i) => (
          <p key={i} className="text-center">
            {text}
          </p>
        ))}
        {content.buttons.length > 0 && (
          <div className="flex gap-2 justify-center flex-wrap">
            {content.buttons.map((btn, i) => (
              <button
                key={i}
                onClick={() => {
                  if (!disabled) onAction(btn.action);
                }}
                disabled={disabled}
                className="px-4 py-2 rounded text-sm font-bold"
                style={{
                  background: "var(--color-primary)",
                  color: "var(--color-text)",
                  cursor: disabled ? "not-allowed" : "pointer",
                  opacity: disabled ? 0.5 : 1,
                  border: "1px solid var(--color-border)",
                }}
              >
                {btn.label}
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
```

- [ ] **Step 3: Add OverlayPrompt to BattleScreen**

Add this import to `BattleScreen.tsx`:

```tsx
import { OverlayPrompt } from "./OverlayPrompt";
```

Add this just before the closing `</div>` of the BattleScreen component's return:

```tsx
      {/* Overlay */}
      {ui.screen_overlay && (
        <OverlayPrompt
          overlay={ui.screen_overlay}
          onAction={onAction}
          disabled={disabled}
        />
      )}
```

- [ ] **Step 4: Verify types compile**

Run: `cd scripts/battle_prototype && npx tsc --noEmit`

Expected: No errors.

- [ ] **Step 5: Commit**

```bash
git add scripts/battle_prototype/src/
git commit -m "feat: FlexNode overlay parsing and prompt rendering"
```

---

## Task 9: Debug Panel

**Files:**
- Create: `scripts/battle_prototype/src/components/DebugPanel.tsx`
- Modify: `scripts/battle_prototype/src/components/BattleScreen.tsx`

- [ ] **Step 1: Create components/DebugPanel.tsx**

```tsx
import type { GameAction, TestDeckName } from "../types/battle";
import { useState } from "react";

interface DebugPanelProps {
  onAction: (action: GameAction) => void;
  onReconnect: (deck: TestDeckName) => void;
  disabled: boolean;
}

const DECKS: TestDeckName[] = ["Vanilla", "StartingFive", "Benchmark1", "Core11"];

interface DebugButtonConfig {
  label: string;
  action: GameAction;
}

const DEBUG_BUTTONS: DebugButtonConfig[] = [
  {
    label: "99 Energy",
    action: {
      BattleAction: {
        Debug: { SetEnergy: { player: "One", energy: 99 } },
      },
    },
  },
  {
    label: "Draw Card",
    action: {
      BattleAction: {
        Debug: { DrawCard: { player: "One" } },
      },
    },
  },
  {
    label: "Enemy Character",
    action: {
      BattleAction: {
        Debug: {
          AddCardToBattlefield: {
            player: "Two",
            card: "00000000-0000-0000-0000-000000000000",
          },
        },
      },
    },
  },
  {
    label: "Opponent Continue",
    action: {
      BattleAction: {
        Debug: "OpponentContinue",
      },
    },
  },
  {
    label: "Deck → 1",
    action: {
      BattleAction: {
        Debug: { SetCardsRemainingInDeck: { player: "One", cards: 1 } },
      },
    },
  },
];

export function DebugPanel({
  onAction,
  onReconnect,
  disabled,
}: DebugPanelProps) {
  const [selectedDeck, setSelectedDeck] = useState<TestDeckName>("Benchmark1");

  return (
    <div
      className="p-4 flex flex-col gap-3"
      style={{
        background: "var(--color-surface)",
        borderTop: "2px solid var(--color-primary)",
      }}
    >
      <h3 className="font-bold text-sm" style={{ color: "var(--color-primary-light)" }}>
        Debug Panel
      </h3>

      {/* Restart with deck */}
      <div className="flex gap-2 items-center">
        <select
          value={selectedDeck}
          onChange={(e) => setSelectedDeck(e.target.value as TestDeckName)}
          className="rounded px-2 py-1 text-sm"
          style={{
            background: "var(--color-surface-light)",
            color: "var(--color-text)",
            border: "1px solid var(--color-border)",
          }}
        >
          {DECKS.map((d) => (
            <option key={d} value={d}>
              {d}
            </option>
          ))}
        </select>
        <button
          onClick={() => onReconnect(selectedDeck)}
          disabled={disabled}
          className="px-3 py-1 rounded text-sm"
          style={{
            background: "var(--color-primary)",
            color: "var(--color-text)",
            opacity: disabled ? 0.5 : 1,
            cursor: disabled ? "not-allowed" : "pointer",
          }}
        >
          Restart Battle
        </button>
      </div>

      {/* Debug action buttons */}
      <div className="flex gap-2 flex-wrap">
        {DEBUG_BUTTONS.map((btn) => (
          <button
            key={btn.label}
            onClick={() => {
              if (!disabled) onAction(btn.action);
            }}
            disabled={disabled}
            className="px-3 py-1 rounded text-xs"
            style={{
              background: "var(--color-surface-light)",
              color: "var(--color-text)",
              border: "1px solid var(--color-border)",
              opacity: disabled ? 0.5 : 1,
              cursor: disabled ? "not-allowed" : "pointer",
            }}
          >
            {btn.label}
          </button>
        ))}
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Add DebugPanel to BattleScreen**

Add the import:

```tsx
import { DebugPanel } from "./DebugPanel";
```

Add a `showDebug` state and the `onReconnect` prop:

Update the `BattleScreenProps` interface and component:

```tsx
interface BattleScreenProps {
  battle: BattleView;
  onAction: (action: GameAction) => void;
  onReconnect: (deck: TestDeckName) => void;
  disabled: boolean;
}
```

Add at the top of the component body:

```tsx
  const [showDebug, setShowDebug] = useState(false);
```

Add the import for `useState`:

```tsx
import { useState } from "react";
```

Replace the `devButton` in `ActionBar` with a custom handler. In the `ActionBar` props, remove `devButton` and instead add a manual Dev button right after the ActionBar:

```tsx
      {/* Action buttons */}
      <ActionBar
        primaryButton={ui.primary_action_button ?? undefined}
        secondaryButton={ui.secondary_action_button ?? undefined}
        undoButton={ui.undo_button ?? undefined}
        incrementButton={ui.increment_button ?? undefined}
        decrementButton={ui.decrement_button ?? undefined}
        onAction={onAction}
        disabled={disabled}
      />

      {/* Dev toggle button */}
      <div className="flex justify-center py-1">
        <button
          onClick={() => setShowDebug((prev) => !prev)}
          className="px-3 py-1 rounded text-xs"
          style={{
            background: "var(--color-surface-light)",
            color: "var(--color-text-dim)",
            border: "1px solid var(--color-border)",
          }}
        >
          {showDebug ? "Hide Debug" : "Show Debug"}
        </button>
      </div>

      {/* Debug panel */}
      {showDebug && (
        <DebugPanel
          onAction={onAction}
          onReconnect={onReconnect}
          disabled={disabled}
        />
      )}
```

- [ ] **Step 3: Update App.tsx to pass onReconnect**

In the `BattleApp` component, update the `BattleScreen` usage:

```tsx
  return (
    <BattleScreen
      battle={battle}
      onAction={sendAction}
      onReconnect={reconnect}
      disabled={isPolling}
    />
  );
```

- [ ] **Step 4: Verify types compile**

Run: `cd scripts/battle_prototype && npx tsc --noEmit`

Expected: No errors.

**QA:** Use `agent-browser` to open the app. Click "Show Debug". Take a screenshot. Verify: debug panel is visible with deck selector and action buttons. Click "99 Energy". Take a screenshot. Verify energy display shows 99. Select "Core11" and click "Restart Battle". Take a screenshot. Verify the game resets with new cards.

- [ ] **Step 5: Commit**

```bash
git add scripts/battle_prototype/src/
git commit -m "feat: debug panel with restart, energy, draw, and other debug controls"
```

---

## Task 10: Interactive QA — Milestone 4 Verification

This task has no new code. It is a dedicated QA pass using `agent-browser` to verify that the full interaction loop works end-to-end.

- [ ] **Step 1: Start the dev server and the battle prototype**

Ensure the Rust dev server is running (`cd rules_engine && just dev-server`) and the Vite dev server is running (`cd scripts/battle_prototype && npm run dev`).

- [ ] **Step 2: Full turn sequence QA**

Use `agent-browser` to:
1. Take a screenshot of the initial state. Note hand size, energy, battlefield.
2. Click a playable card (green outline). Take a screenshot after.
3. Verify: card moved from hand, energy decreased.
4. Click primary action button (Pass Priority / End Turn). Take a screenshot.
5. Wait for AI turn to complete. Take a screenshot.
6. Verify: AI played cards, turn incremented, energy reset.
7. Play through 3+ full turns. Take screenshots each turn.
8. Verify judgment phase scoring (if spark difference exists).

- [ ] **Step 3: Fix any bugs found during QA**

Fix all issues discovered. Re-run the QA steps to verify fixes. Take screenshots confirming each fix.

- [ ] **Step 4: Commit any fixes**

```bash
git add scripts/battle_prototype/src/
git commit -m "fix: address bugs found during interaction QA"
```

---

## Task 11: Card Browser & Card Order Selector

**Files:**
- Modify: `scripts/battle_prototype/src/components/BattleScreen.tsx`

The card browser (for viewing void cards) and card order selector (for Foresee) use cards with `Position::Browser` and `Position::CardOrderSelector`. We need to render these as modal overlays.

- [ ] **Step 1: Add browser and card order selector rendering to BattleScreen**

Add these helper functions at the top of BattleScreen.tsx (or in a new component):

```tsx
function browserCards(cards: CardView[]): CardView[] {
  return cards.filter((c) => c.position.position === "Browser");
}

function cardOrderCards(cards: CardView[]): CardView[] {
  return cards.filter((c) => {
    const pos = c.position.position;
    return typeof pos !== "string" && "CardOrderSelector" in pos;
  });
}
```

Add to the BattleScreen component return, before the overlay:

```tsx
      {/* Card browser (void browsing) */}
      {ui.browser && browserCards(battle.cards).length > 0 && (
        <div
          className="fixed inset-0 flex items-center justify-center z-40"
          style={{ background: "rgba(0, 0, 0, 0.7)" }}
        >
          <div
            className="rounded-lg p-4 max-w-2xl w-full mx-4"
            style={{
              background: "var(--color-surface)",
              border: "1px solid var(--color-border)",
              maxHeight: "80vh",
              overflowY: "auto",
            }}
          >
            <div className="flex justify-between items-center mb-3">
              <h3 className="font-bold">Card Browser</h3>
              {ui.browser.close_button && (
                <button
                  onClick={() => onAction(ui.browser!.close_button!)}
                  className="px-3 py-1 rounded text-sm"
                  style={{
                    background: "var(--color-surface-light)",
                    border: "1px solid var(--color-border)",
                  }}
                >
                  Close
                </button>
              )}
            </div>
            <div className="flex gap-2 flex-wrap justify-center">
              {browserCards(battle.cards).map((card) => (
                <CardDisplay
                  key={card.id}
                  card={card}
                  onAction={onAction}
                  disabled={disabled}
                />
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Card order selector (Foresee) */}
      {ui.card_order_selector && cardOrderCards(battle.cards).length > 0 && (
        <div
          className="fixed inset-0 flex items-center justify-center z-40"
          style={{ background: "rgba(0, 0, 0, 0.7)" }}
        >
          <div
            className="rounded-lg p-4 max-w-2xl w-full mx-4"
            style={{
              background: "var(--color-surface)",
              border: "1px solid var(--color-border)",
            }}
          >
            <h3 className="font-bold mb-3">Reorder Cards (click to select position)</h3>
            <div className="flex gap-2 flex-wrap justify-center">
              {cardOrderCards(battle.cards)
                .sort((a, b) => a.position.sorting_key - b.position.sorting_key)
                .map((card) => (
                  <CardDisplay
                    key={card.id}
                    card={card}
                    onAction={onAction}
                    disabled={disabled}
                  />
                ))}
            </div>
          </div>
        </div>
      )}
```

- [ ] **Step 2: Verify types compile**

Run: `cd scripts/battle_prototype && npx tsc --noEmit`

Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add scripts/battle_prototype/src/
git commit -m "feat: card browser and card order selector modals"
```

---

## Task 12: Polish Pass

**Files:**
- Modify: various components for styling improvements

- [ ] **Step 1: Improve visual polish**

Go through each component and ensure:
- Dark background `#0a0612` is applied to body (already in CSS)
- Card borders use tide colors when applicable (from `outline_color`)
- Stack zone has gold border that collapses when empty (already done)
- Disabled/polling state greys out cards and buttons (already handled via `opacity`)
- Status bars are compact and readable
- Hand cards are slightly larger than battlefield cards (already done via `compact` prop)

No specific code changes prescribed here — the subagent should review screenshots and fix any visual issues.

**QA:** Take screenshots of:
1. Initial game state — verify dark theme, readable text
2. During polling — verify disabled state is clear
3. Empty stack — verify it disappears
4. Full battlefield (8 cards) — verify they fit
5. Game end state

- [ ] **Step 2: Commit any polish changes**

```bash
git add scripts/battle_prototype/src/
git commit -m "fix: visual polish and styling improvements"
```

---

## Task 13: Extended Playtesting — Round 1-2 (Basic Flow)

No new code. This is pure QA using `agent-browser`.

- [ ] **Step 1: Play a full game with Benchmark1 vs Benchmark1**

Start a new battle. Play every card you can each turn. Take a screenshot every turn. Play until one player reaches 12 points or 50 turns. Log every bug found.

- [ ] **Step 2: Fix all bugs found in Round 1**

Fix each bug. Re-test the specific scenario. Take screenshots confirming fixes.

- [ ] **Step 3: Play a full game with Core11 vs Core11**

Same protocol. Different deck to exercise different cards/abilities.

- [ ] **Step 4: Fix all bugs found in Round 2**

Fix each bug. Re-test. Screenshots.

- [ ] **Step 5: Commit all fixes**

```bash
git add scripts/battle_prototype/src/
git commit -m "fix: bugs found during playtest rounds 1-2 (basic flow)"
```

---

## Task 14: Extended Playtesting — Round 3-4 (Prompts & Targeting)

- [ ] **Step 1: Play a game focused on prompts and targeting**

Use debug panel to set energy to 99. Deliberately play cards that require targets. Trigger choice prompts if available. Use Foresee if available. Take screenshots of every prompt interaction.

- [ ] **Step 2: Fix all bugs found in Round 3**

- [ ] **Step 3: Play another game focused on prompts with a different deck**

- [ ] **Step 4: Fix all bugs found in Round 4**

- [ ] **Step 5: Commit all fixes**

```bash
git add scripts/battle_prototype/src/
git commit -m "fix: bugs found during playtest rounds 3-4 (prompts & targeting)"
```

---

## Task 15: Extended Playtesting — Round 5-6 (Edge Cases)

- [ ] **Step 1: Test edge cases**

- Fill battlefield to 8 characters. Play a 9th. Verify abandon mechanic.
- Empty your deck. Verify behavior.
- Play with 0 energy. Verify only free cards are playable.
- Put multiple cards on the stack simultaneously.
- Take screenshots of each edge case.

- [ ] **Step 2: Fix all bugs found in Round 5**

- [ ] **Step 3: More edge case testing with different deck**

- [ ] **Step 4: Fix all bugs found in Round 6**

- [ ] **Step 5: Commit all fixes**

```bash
git add scripts/battle_prototype/src/
git commit -m "fix: bugs found during playtest rounds 5-6 (edge cases)"
```

---

## Task 16: Extended Playtesting — Round 7-8 (Full Game Arc)

- [ ] **Step 1: Play a full game start-to-finish without debug tools**

Use Vanilla deck. Play every turn carefully. Verify game ends correctly with accurate scores.

- [ ] **Step 2: Fix all bugs found in Round 7**

- [ ] **Step 3: Play another full game with StartingFive deck**

Try to reach high turn counts.

- [ ] **Step 4: Fix all bugs found in Round 8**

- [ ] **Step 5: Commit all fixes**

```bash
git add scripts/battle_prototype/src/
git commit -m "fix: bugs found during playtest rounds 7-8 (full game arc)"
```

---

## Task 17: Extended Playtesting — Round 9-10 (Stress Testing)

- [ ] **Step 1: Stress test with aggressive debug panel usage**

Set 99 energy, draw many cards, add enemy characters, restart mid-game, switch decks repeatedly. Take screenshots throughout.

- [ ] **Step 2: Fix all bugs found in Round 9**

- [ ] **Step 3: Another stress test round**

Same aggressive debug usage with a different deck.

- [ ] **Step 4: Verify Round 10 is clean (ZERO bugs)**

If any bugs are found, fix them and add additional rounds until two consecutive rounds are clean.

- [ ] **Step 5: Commit final fixes**

```bash
git add scripts/battle_prototype/src/
git commit -m "fix: final bugs from stress testing rounds 9-10"
```

---

## Task 18: Final Commit & Cleanup

- [ ] **Step 1: Run typecheck**

Run: `cd scripts/battle_prototype && npx tsc --noEmit`

Expected: No errors.

- [ ] **Step 2: Run lint**

Run: `cd scripts/battle_prototype && npm run lint`

Expected: No errors (or only acceptable warnings).

- [ ] **Step 3: Final commit**

```bash
git add scripts/battle_prototype/
git commit -m "feat: battle prototype web client — complete and playtested"
```
