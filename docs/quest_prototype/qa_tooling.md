# QA Tooling for the Quest Prototype

Browser QA for the quest prototype uses `agent-browser` against the local Vite
app at `http://localhost:5173`. Do not use WebFetch for localhost, and do not
substitute Python Playwright when `agent-browser` is available in this
environment.

## Browser Automation

Confirm the tool exists before starting:

```bash
which agent-browser
```

Start the prototype, then open it in `agent-browser`:

```bash
cd /Users/dthurn/dreamtides/scripts/quest_prototype
npm install
npm run dev
```

In a second shell:

```bash
agent-browser open http://localhost:5173
agent-browser wait --load networkidle
```

Use the accessibility snapshot to inspect the current screen and find clickable
element refs:

```bash
agent-browser snapshot -i
agent-browser click @e3
agent-browser wait 500
```

## Hidden Tides Smoke Path

The hidden-tides migration smoke path is:

1. Open `http://localhost:5173` and confirm the app starts on the Dreamcaller
   selection screen.
2. Verify the quest start shows exactly 3 Dreamcaller choices.
3. Pick one Dreamcaller and confirm the run enters quest play immediately with
   no tide-pick step, no tide crystals, and no `DreamcallerDraft` screen.
4. Reach a draft site and confirm the offer shows 4 unique card names.
5. Continue far enough to see another draft offer and confirm duplicates can
   recur across the run but never inside the same offer.
6. Reach a Dreamsign surface and confirm shown Dreamsigns are spent immediately,
   including skipped ones.
7. Reach a later Dreamsign surface in the same run and confirm no previously
   shown Dreamsign repeats.
8. Open the debug surface, if present, and confirm package summary details are
   visible there without exposing package internals in the normal player flow.

## Screenshots And Inspection

Take screenshots for each major state change:

```bash
agent-browser screenshot /tmp/quest-start.png
agent-browser screenshot --annotate /tmp/quest-atlas.png
agent-browser screenshot --full /tmp/quest-draft.png
```

Useful inspection commands:

```bash
agent-browser snapshot -i
agent-browser eval "JSON.stringify(window.__questLog || [])"
agent-browser eval "JSON.stringify(window.__errors || [])"
```

Use these checks during QA:

- The heading contains `Dreamtides`.
- Normal player-facing screens do not expose `mandatoryTides`, `optionalTides`,
  card `tides`, tide crystals, or `DreamcallerDraft`.
- Draft offers remain 4-unique until fewer than 4 unique names remain in the
  pool.
- Dreamsigns do not repeat within one run.
- `window.__errors` stays empty.

## TypeScript Module Testing

To invoke TypeScript modules directly (without a browser):

```bash
# Single expression
node --experimental-strip-types -e 'import { fn } from "./src/module.ts"; ...'

# Complex script — write to a file first to avoid shell escaping problems
cat > /tmp/test.mjs << 'EOF'
import { fn } from "/abs/path/to/src/module.ts";
console.log(fn());
EOF
node --experimental-strip-types /tmp/test.mjs
```

Avoid `node -e` with shell-special characters like `!`. Write to a file and run
it instead.

Running `npx tsx` is an alternative if `node --experimental-strip-types` does
not resolve imports correctly.

## Vite SPA Fallback Behavior

Vite serves the HTML fallback document for any path that does not match a static
file. This means a `curl` request for a missing static file (e.g.,
`/tides/Wild.png`) returns HTTP 200 with `Content-Type: text/html`, not 404.
Check the `Content-Type` header rather than the status code when verifying
whether a static file exists:

```bash
curl -s -I http://localhost:5173/tides/Arc.png | grep content-type
# content-type: image/webp  → file exists
# content-type: text/html   → Vite fallback, file missing
```

## TypeScript `as const` at Runtime

TypeScript `as const` arrays are a compile-time constraint only. Testing
mutability at runtime with `.push()` will always succeed, even for correctly
typed `readonly` arrays. Verify readonly enforcement only through
`npm run typecheck`, not through runtime mutation attempts.

## Dev Server Setup

The dev server output confirms the port. If assets are missing, the setup script
logs warnings but continues and the prototype uses placeholders.
