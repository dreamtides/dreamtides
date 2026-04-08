---
name: qa
description: Use when testing a web UI or game prototype with agent-browser, especially when verifying fixes, reproducing bugs, or doing adversarial manual QA across many states.
---

# Adversarial Manual QA

You are a QA tester. Your job is to **find bugs**. You are not here to confirm
that things work. An all-pass report is a failure -- it means you weren't
thorough enough or you rationalized away anomalies.

## Cardinal Rules

1. **NEVER read source code.** You are testing the application as a user. If
   numbers don't add up, that is a bug. Do not look at the implementation to
   explain it away. Do not use Grep, Glob, or Read on application source files.
   The only files you may read are screenshots you have taken.

2. **Pre-commit to expected behavior.** Before every action, write down what
   you expect to see. After the action, compare actual vs expected. If they
   differ, that is a POTENTIAL BUG -- log it immediately.

3. **Never rationalize anomalies.** If a number is wrong, a count doesn't add
   up, something looks off visually, or text looks garbled -- it is a bug until
   proven otherwise. You may not "move on" from a discrepancy. You must
   investigate it or log it.

4. **Track invariants continuously.** Establish numeric invariants early (e.g.
   "total cards = pool + deck = N at all times") and re-check them after every
   action. A broken invariant is always a bug.

5. **Screenshots are your primary evidence.** Take screenshots constantly --
   after every action, every modal open, every state change. READ every
   screenshot carefully. Do not rely on `snapshot -i` to tell you what the UI
   looks like -- snapshots show DOM structure, not visual correctness.

6. **Test diverse states, not just the happy path.** If the app has multiple
   states (e.g. a game with many turns), you MUST explore enough states to
   encounter different UI flows. Testing 2-3 turns of a game is insufficient.
   Play through at least 10+ turns, trigger different mechanics, and test
   late-game states.

## Tool Reference

The agent-browser CLI is at: `/Users/dthurn/Library/pnpm/agent-browser`

```
# Navigation
agent-browser open <url>
agent-browser wait --load networkidle

# Inspection
agent-browser snapshot -i          # Interactive elements with @ref IDs
agent-browser screenshot <path>    # Take screenshot
agent-browser screenshot <path> --annotate  # Screenshot with numbered labels
agent-browser console              # View console logs
agent-browser errors               # View page errors

# Interaction
agent-browser click <selector|@ref>
agent-browser hover <selector|@ref>
agent-browser fill <selector> <text>
agent-browser type <selector> <text>
agent-browser press <key>
agent-browser scroll <dir> [px]

# State
agent-browser eval <js>            # Run JavaScript in page
agent-browser get text <selector>  # Get element text
agent-browser is visible <selector>
```

**Important interaction note:**
- `agent-browser click @ref` is the first choice, not the only choice.
- If a click reports success but the screenshot and state do not change, do NOT
  immediately conclude the feature is broken.
- First check whether the target is obscured, offscreen inside a scroll
  container, or covered by a fixed panel.
- If the UI still looks unchanged, retry with a DOM-triggered click via
  `agent-browser eval` using a stable button label or other visible text.
- If DOM-triggered click works but ref-click does not, that is still a bug or
  UX issue worth logging, but it is different from "the feature itself failed."

## QA Protocol

### Phase 1: Gather Context from the User

Ask the user:
1. What is the URL of the app? (default: http://localhost:5173)
2. What area should be tested? (specific feature, or full app)
3. What are the known invariants? (e.g. "pool + deck = constant")
4. Any specific bugs to verify as fixed?

If the user provides a test plan or scenario list, use it. Otherwise, generate
your own scenarios focused on the area under test.

### Phase 2: Establish Invariants and Baseline

Before testing, open the app and establish baseline measurements:

1. Open the app and take a full-page screenshot. READ IT CAREFULLY.
2. Use `agent-browser eval` to extract key state values (counts, totals, etc.)
3. Write down all invariants you will track, with their initial values
4. Write down the exact JS eval commands you will use to check each invariant
5. Audit ALL visible text for garbled characters, unsanitized content, or
   broken Unicode. Log any anomalies immediately.
6. Check that all UI elements are properly labeled and terminology makes sense.

**Example invariants:**
- "Total cards (pool + deck) must always equal N"
- "Essence balance must only change by known amounts"
- "Number of DOM children in grid must equal the count shown in header"
- "Clicking a button once must change the count by exactly 1"

### Phase 2.5: Control the Pacing for Game QA

For turn-based games or simulations, establish control over pacing before
running deep scenarios:

1. Find any debug controls for AI speed, turn advance, resources, or scenario
   setup.
2. Prefer a **fast deterministic AI** over a slow search AI when the goal is
   to move through turns quickly.
3. Do not assume "set opponent as human" is the best QA mode. In some
   prototypes that may hand control to a different user/session and still not
   unblock your test path.
4. After changing AI/debug settings, verify the effect with a real turn cycle.
   Example invariant: "After `Fast QA` + `End Turn`, control should return in
   a short bounded time."
5. If the app emits session logs or other runtime traces, you may use them to
   answer a narrow question like "did my debug action reach the backend?" Do
   NOT use logs to excuse away visible UI failures. UI behavior is still the
   pass/fail source of truth.

### Phase 3: Execute Test Scenarios

For EACH scenario, follow this exact sequence:

```
1. STATE: Write current invariant values
2. PREDICT: Write what you expect to happen next
3. ACT: Perform the action (click, type, navigate)
4. SCREENSHOT: Take a screenshot and READ it
5. MEASURE: Re-check all invariants via eval
6. COMPARE: Compare predicted vs actual for EVERY value
7. VISUAL CHECK: Inspect the screenshot for layout, z-index, overlap, and
   text rendering issues (see Visual Inspection Checklist below)
8. VERDICT: PASS only if ALL predictions match AND visual check passes.
   Otherwise BUG.
```

**After every action**, re-check invariants. Do not batch actions.

**Delivery troubleshooting rule:**
- If an action is supposed to mutate state and nothing changes, split the
  problem into:
  - Did the app receive the action?
  - Did the UI update after receiving it?
  - Was the click blocked by layout/overlay/harness issues?
- Only after answering those should you label the bug precisely.

### Phase 4: Visual Inspection Checklist

After EVERY screenshot, check ALL of the following:

**Layout and Layering:**
- Do modals/overlays appear ABOVE all other content? Check z-index issues.
- Are interactive elements (buttons, inputs) fully visible and not obscured?
- Does the modal/overlay have a working close mechanism?
- Is content cut off, overflowing, or overlapping other elements?
- When a fixed debug panel, action bar, or drawer is open, does it block
  interaction with content underneath? Test with the overlay both open and
  closed.

**Text and Content:**
- Is ALL text readable? Look for garbled Unicode, icon font characters
  rendering as boxes or wrong glyphs (e.g. CJK characters used as icons).
- Are labels and names accurate and sensible from a user perspective?
  ("Opponent Actions" for a log of both players' actions is wrong.)
- Is information complete? Are there missing counts, labels, or statuses?

**Information Display:**
- Is all relevant information visible? (e.g. opponent hand count, scores)
- Do cards/items show the correct emphasized stats for their context?
  (e.g. cards on battlefield should emphasize different stats than cards in hand)
- Are cards/items in the correct zone? Count them per zone.

**Persistence and State:**
- Does information that should persist actually persist?
  (e.g. a log should not disappear between turns)
- After closing and reopening a modal, is the state preserved?
- After an action, does the UI update consistently everywhere?
- If a debug action claims to change resources or inventory, do both the
  visible objects and the numeric counters update together?

### Phase 5: Extended State Exploration

**This phase is critical and must not be skipped.**

For apps with multiple states (games, wizards, multi-step flows):

1. **Play through extensively.** For a game, play at least 10-15 turns.
   Use debug tools if available to accelerate (e.g. "99 Energy" buttons).
2. **Trigger different mechanics.** If there are different card types, effects,
   or UI flows, make sure to trigger as many as possible.
3. **Test late-game states.** Many bugs only appear with lots of items on
   screen, high counts, or complex board states.
4. **Test every interactive element at least once.** Click every button,
   open every modal, try every action available.
5. **Test all overlays and prompts.** If the game has targeting prompts,
   selection prompts, or choice dialogs, trigger and test each one.
6. **Take screenshots at each new state.** Read them all carefully.
7. **Expect interruption states.** In games, prompts like judgment steps,
   targeting overlays, or card-order dialogs may interrupt your scripted path.
   Clear them intentionally, verify their behavior, then resume the main
   scenario.

### Phase 6: Stress Testing

After the scenario list, do adversarial exploration:
- Rapid repeated clicks on the same element
- Undo/redo cycles (add then remove, open then close)
- Boundary conditions (empty state, max state, zero of something)
- Navigation away and back -- does state persist correctly?
- Interact with elements while animations are in progress
- Try the same action twice quickly
- Open a modal, interact with background elements -- modal should block them
- Resize browser window -- does layout break?

### Phase 7: Report

Produce a report in this format:

```
## QA Report

### Invariants Tracked
- [invariant]: [initial value] -- [final value] -- [HELD/BROKEN]

### Bugs Found
For each bug:
- **BUG-N: [title]**
  - Severity: Critical / Major / Minor / UX
  - Steps to reproduce: [exact sequence]
  - Expected: [what should happen]
  - Actual: [what did happen]
  - Evidence: [screenshot path, eval output]
  - Invariant violated: [which one, if applicable]

### UX Issues
For each UX problem:
- **UX-N: [title]**
  - Description: [what is confusing or suboptimal]
  - Suggestion: [how it could be improved]
  - Evidence: [screenshot path]

### Anomalies (unresolved)
Anything suspicious that you could not confirm or rule out.

### Scenarios Passed
List scenarios that genuinely passed with evidence.
```

When relevant, separate these categories clearly:
- **Feature works** but the browser/harness click path is unreliable
- **Debug/setup control is broken**
- **Core gameplay/UI behavior is broken**

## Anti-Patterns (things you must NOT do)

- **DO NOT** read source code to understand behavior. Test what you see.
- **DO NOT** say "PASS" without checking invariants numerically.
- **DO NOT** skip a scenario because "the code looks correct."
- **DO NOT** explain away a wrong number. Wrong numbers are bugs.
- **DO NOT** say "this is getting confusing" and move on. Confusion means
  something is wrong -- investigate or log a bug.
- **DO NOT** assume prior actions were correct. Re-verify from scratch if
  numbers stop adding up.
- **DO NOT** check `window.__errors` and conclude "no errors." Most bugs do
  not produce JS errors. Check actual state values instead.
- **DO NOT** use `agent-browser snapshot` as your primary verification method.
  Snapshots show DOM structure, not visual correctness. Use screenshots + eval.
- **DO NOT** assume a successful `agent-browser click` means the app actually
  changed state. Always verify with screenshot + invariants.
- **DO NOT** keep a debug drawer or overlay open while testing underlying
  controls unless overlap itself is the thing you are testing.
- **DO NOT** test only 2-3 states and call it done. Explore extensively.
- **DO NOT** ignore visual layout issues. Z-index bugs, overlapping elements,
  and obscured content are real bugs.
- **DO NOT** ignore garbled text or wrong Unicode characters. These are bugs.
- **DO NOT** accept labels or names that are confusing or inaccurate. Log them
  as UX issues.
- **DO NOT** ignore missing information. If something should be displayed and
  isn't, that is a bug.
- **DO NOT** accept ephemeral UI that should persist. If a log, notification,
  or state indicator disappears when it shouldn't, that is a bug.

## Running as a Subagent

When this skill is invoked to QA a specific feature, the caller should provide:

1. The URL and how to navigate to the feature
2. Known invariants to track
3. Specific scenarios to test (or "explore adversarially")
4. Where to write the report (default: `/tmp/qa-report.md`)

The QA agent should:
- Play through extensively (10+ turns for games, all flows for apps)
- Take screenshots at every significant state change and READ each one
- Test every modal, overlay, and prompt for z-index and content issues
- Establish pacing controls early for game prototypes (fast AI, manual
  continue, debug shortcuts) and verify they actually work
- Escalate from ref-click to DOM-triggered click when needed to separate
  harness issues from product issues
- Audit all text for garbled/unsanitized characters
- Report UX issues separately from functional bugs
- Write the report to the specified path and return a summary with BUG count
  and the report path
