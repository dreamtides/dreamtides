# QA Tooling for the Quest Prototype

Browser testing for the quest prototype requires Python Playwright. WebFetch and
`mcp__ide__executeCode` both fail on localhost URLs. This document covers the
correct tools and patterns.

## Browser Automation

**Python Playwright** is the correct tool for all UI interaction and
screenshots. It is installed system-wide.

```python
from playwright.sync_api import sync_playwright

with sync_playwright() as p:
    browser = p.chromium.launch()
    page = browser.new_page(viewport={"width": 1280, "height": 900})
    page.goto("http://localhost:5173")
    page.screenshot(path="/tmp/screenshot.png")
    browser.close()
```

Confirm availability: `which agent-browser` or
`/opt/homebrew/bin/playwright --version`.

**WebFetch does not work on localhost.** Every attempt to
`WebFetch http://localhost:5173` returns `ERROR: Invalid URL`. Do not try it. Do
not search for `mcp__ide__executeCode` — it is not available in this pipeline
context.

## Screenshot Limitations

Screenshots saved to `/tmp/` cannot be viewed inline by agents — the Read tool
returns `(no result)` for PNG files in subagent contexts. Screenshots are
evidence for human review only.

Use DOM inspection as the primary verification method:

```python
# Extract computed CSS values
color = page.evaluate(
    "getComputedStyle(document.querySelector('.hud')).backgroundColor"
)

# Read element text
text = page.locator(".essence-counter").inner_text()

# Check element dimensions
box = page.locator(".card-grid").bounding_box()
```

Combine `page.evaluate()`, `page.locator().inner_text()`, and
`page.locator().bounding_box()` to assert layout, content, and color without
relying on screenshot viewing.

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

Start the dev server before any browser QA:

```bash
cd scripts/quest_prototype
npm install   # if node_modules is missing
npm run dev   # runs setup-assets.mjs then starts Vite at localhost:5173
```

The dev server output confirms the port. If assets are missing (card images,
tide PNGs), the setup script logs warnings but continues — the prototype runs
with placeholder images.
