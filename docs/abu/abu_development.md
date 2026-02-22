# Abu Development Guide

How to modify the Abu scene walker, add support for new UI features, and
interactively validate changes against a running Unity game.

## Table of Contents

- [Development Workflow](#development-workflow)
- [Adding a New UI Feature](#adding-a-new-ui-feature)
- [Interactive Testing](#interactive-testing)
- [Troubleshooting](#troubleshooting)

## Development Workflow

The Abu development loop has three steps: edit C# code, compile in Unity, and
validate via CLI snapshots. All three steps can be driven from the terminal
without touching the Unity Editor GUI.

**Step 1 — Edit the scene walker.** The main file is
`client/Assets/Dreamtides/Abu/DreamtidesSceneWalker.cs`. The `Walk()` method
builds a tree of `AbuSceneNode` objects representing the current UI. Battle mode
is handled in `WalkBattle()`; non-battle mode falls back to `WalkUiToolkit()`
and `WalkFallbackScene3D()`.

**Step 2 — Compile.** Run `python3 scripts/abu/abu.py refresh` to trigger asset
compilation. This drives Unity's Assets > Refresh menu via Hammerspoon and waits
for the compiler to finish. If there are C# errors, they are printed and the
script exits non-zero.

**Step 3 — Test.** Enter play mode with `python3 scripts/abu/abu.py play`, then
use `just abu --wait 30 snapshot --compact` to inspect the accessibility tree.
The `--wait` flag retries the connection for up to the specified number of
seconds, so you don't need to guess how long Unity takes to initialize. Use
`just abu click`, `just abu drag`, and `just abu hover` to interact and verify
that actions produce the expected state changes.

**Iterating:** Use `python3 scripts/abu/abu.py cycle` to automate the
edit-compile-test loop. This command detects whether play mode is active, exits
it if so, triggers a refresh (compilation), and re-enters play mode — all in one
step. Changes to C# code do not hot-reload during play mode, so this cycle is
required after every edit.

## Adding a New UI Feature

When a game UI element is not represented in Abu snapshots, add support in the
scene walker. The general pattern:

1. **Detect visibility.** Check whether the feature is active (e.g.
   `selector.IsOpen`, `layout.ThinkingIndicator.activeSelf`, a list having
   items). Return early or skip the section when the feature is inactive.

2. **Build the node tree.** Create `AbuSceneNode` objects with appropriate
   `Role` values: `"group"` for containers, `"button"` for clickable elements,
   `"label"` for read-only text, `"target"` for drag destinations.

3. **Register callbacks.** For each interactive node, call
   `refRegistry.Register(callbacks)` with a `RefCallbacks` object. The three
   callback types are `OnClick` (invoked by `just abu click`), `OnHover`
   (invoked by `just abu hover`), and `OnDrag` (invoked by `just abu drag`,
   receives the optional target ref string). The registration order must match
   the DFS order of the node tree — register each node's callbacks at the same
   point you add it to the tree.

4. **Wire actions.** Callbacks should construct and fire game actions through
   `_registry.ActionService.PerformAction()`. For click simulation on
   `Displayable` subclasses, use `BuildDisplayableCallbacks()` which injects a
   fake input provider. For drag-to-target interactions, store the target refs
   returned by `refRegistry.Register()` and look them up in the `OnDrag`
   callback to determine which action to fire.

5. **Place in the walk.** Add your new section at the appropriate point in
   `WalkBattle()` or the non-battle fallback. Consider whether the feature is
   always visible, only visible when panels are closed (`!hasOpenPanels`), or
   independent of panel state.

**Example:** The CardOrderSelector (Foresee reordering) adds deck position
targets and a void target, walks cards in both zones, and gives each card an
`OnDrag` callback that maps the target ref to a `SelectOrderForDeckCard` game
action.

## Interactive Testing

Set up the game scene so the feature you're testing is visible (e.g. trigger a
Foresee prompt for the CardOrderSelector). Then use the CLI to validate.

**Taking snapshots:**

```sh
python3 scripts/abu/abu.py play                  # enter play mode
just abu --wait 30 snapshot --compact            # wait for connection, then snapshot
```

The `--wait 30` flag retries the TCP connection for up to 30 seconds,
eliminating the need to manually sleep. The `--compact` flag omits
non-interactive unlabeled nodes for readability. Drop it to see the full tree.

**Single-command recompile and restart:**

```sh
python3 scripts/abu/abu.py cycle                 # exit play → refresh → enter play
just abu --wait 30 snapshot --compact            # snapshot after cycle completes
```

**Performing actions and verifying state changes:**

```sh
just abu click e5                     # click a button
just abu drag e10 e3                  # drag source to target
just abu hover e7                     # hover an element
```

Each action command returns a fresh snapshot after the UI settles. History
entries (game events that occurred during the action) appear before the
snapshot, bracketed by `--- History ---` / `---` lines. History is absent when
the action produced no observable events (e.g. hover over an empty zone).

**Inspecting raw JSON (for debugging history or wire protocol):**

```sh
just abu --json click e5 2>/dev/null  # suppress command echo, print raw JSON
```

The `just` wrapper echoes the underlying command to stderr; suppress it with
`2>/dev/null` when piping JSON to a parser.

**Full validation cycle:**

1. Take a snapshot; confirm the new feature's nodes appear with correct labels
   and roles.
2. Perform an action (click, drag); confirm the returned snapshot reflects the
   expected state change.
3. Perform follow-up actions to test the full interaction flow (e.g. drag card
   to void, drag back to deck, click Submit).
4. Confirm the feature disappears from the snapshot when it should (e.g. after
   submitting, the selector closes).

**C# unit tests** for the scene walker live in
`client/Assets/Dreamtides/Tests/Abu/SceneWalkerTests.cs`. Run them with
`python3 scripts/abu/abu.py test`. These tests use mock registries and do not
require play mode.

## Troubleshooting

**`No module named pytest`** — Abu Python tests use `unittest`, not `pytest`.
Run `cd scripts/abu && python3 -m unittest test_abu -v`. The module name
`test_abu` (not a file path) requires `scripts/abu/` as the working directory.

**"Could not connect to Unity on localhost:9999"** — The game is not in play
mode, or it hasn't finished initializing. Use `--wait 30` to automatically retry
instead of failing immediately.

**Snapshot is stale after code changes** — Use
`python3 scripts/abu/abu.py cycle` to exit play mode, recompile, and re-enter in
one step. C# changes do not take effect during an active play session.

**Duplicate nodes in snapshot** — Check whether a feature's nodes are being
added both in your new code and in an existing section (e.g. action buttons
appearing in both the main Actions group and a feature-specific group).

**Ref numbers shifted** — Refs are assigned in DFS order and change whenever the
tree structure changes. Always re-read the snapshot after any action before
using ref numbers.

**Refresh times out** — The `abu.py refresh` command has a 120-second timeout.
If Unity is in play mode, exit first. If compilation is genuinely slow, the
timeout may need adjustment.
