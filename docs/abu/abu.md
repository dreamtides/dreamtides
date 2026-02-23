# ABU (Agent-Browser for Unity)

ABU enables AI agents to control the Unity Editor and interact with a running
Dreamtides game through a single Python CLI. Editor commands (refresh, play,
test, cycle) drive Unity's menu bar via Hammerspoon. In-game commands (snapshot,
click, hover, drag, screenshot) connect over TCP port 9999 using NDJSON. All C#
source lives in `client/Assets/Dreamtides/Abu/`.

## Table of Contents

- [Running ABU](#running-abu)
- [Worktree Management](#worktree-management)
- [Worktree Support](#worktree-support)
- [Adding DreamtidesAbuSetup to a Scene](#adding-dreamtidesabusetup-to-a-scene)
- [Architecture](#architecture)
- [Python CLI](#python-cli)
- [C# Transport](#c-transport)
- [C# Command Handling](#c-command-handling)
- [Wire Protocol](#wire-protocol)
- [Snapshot Format](#snapshot-format)
- [Dreamtides UI Systems](#dreamtides-ui-systems)
- [Conventions](#conventions)
- [Common Pitfalls](#common-pitfalls)
- [Adapting ABU to Another Game](#adapting-abu-to-another-game)
- [Testing and Validation](#testing-and-validation)
- [Test Save Generation](#test-save-generation)

## Running ABU

```sh
# Editor commands (require Hammerspoon)
python3 scripts/abu/abu.py refresh               # trigger asset refresh, wait for completion
python3 scripts/abu/abu.py refresh --play         # refresh then enter play mode
python3 scripts/abu/abu.py play                   # toggle play mode
python3 scripts/abu/abu.py test                   # refresh then run all Edit Mode tests
python3 scripts/abu/abu.py cycle                  # exit play → refresh → enter play
python3 scripts/abu/abu.py restart                # kill and relaunch Unity, restore scene
python3 scripts/abu/abu.py status                # show Unity state from state file and TCP probe
python3 scripts/abu/abu.py clear-save            # delete all Dreamtides save files
python3 scripts/abu/abu.py set-mode Battle       # set game mode (Quest, Battle, PrototypeQuest)

# Test save generation (no Unity required)
python3 scripts/abu/abu.py create-save --energy 99 --card "Break the Sequence"
python3 scripts/abu/abu.py create-save --energy 50 --card "Abolish" --card "Apocalypse"
python3 scripts/abu/abu.py create-save --list-cards  # list all available card names

# In-game commands (require Unity in Play mode)
python3 scripts/abu/abu.py snapshot              # ARIA-style scene tree
python3 scripts/abu/abu.py snapshot --compact    # abbreviated tree (interactive + labeled nodes)
python3 scripts/abu/abu.py snapshot --interactive # show only interactive elements
python3 scripts/abu/abu.py click e1              # click element with ref e1
python3 scripts/abu/abu.py click @e1             # leading @ is stripped automatically
python3 scripts/abu/abu.py hover e1              # hover element
python3 scripts/abu/abu.py drag e2 e5            # drag source to target
python3 scripts/abu/abu.py screenshot            # save PNG, print path

# Override the default port
ABU_PORT=9998 python3 scripts/abu/abu.py snapshot
```

Editor commands use Hammerspoon to drive Unity's menu bar via PID-targeted
lookup, so they work from both the main repo and worktrees. In-game commands
connect over TCP to Unity in Play mode. Port resolution order: `ABU_PORT` env
var > worktree `.ports.json` > default 9999. See
[Worktree Support](#worktree-support) for details.

## Worktree Management

ABU includes APFS-backed worktree lifecycle commands under the `worktree`
subcommand group. These create git worktrees with APFS copy-on-write clones of
all gitignored directories (build caches, Unity Library, etc.), enabling
near-instant setup with warm caches at negligible disk cost.

```sh
# Create a new worktree (creates branch + APFS-cloned caches)
abu worktree create my-branch
abu worktree create my-branch --base develop      # branch from develop instead of master
abu worktree create my-branch --existing           # check out existing branch
abu worktree create my-branch --dest /custom/path  # custom location
abu worktree create my-branch --dry-run            # preview without creating

# Remove a worktree
abu worktree remove my-branch
abu worktree remove my-branch --delete-branch      # also delete the git branch

# Refresh gitignored dirs from main repo (reduce COW divergence)
abu worktree refresh                              # refresh current worktree
abu worktree refresh alpha                        # refresh specific worktree
abu worktree refresh --all                        # refresh all worktrees
abu worktree refresh --build                      # cargo check first to warm cache
abu worktree refresh --dry-run                    # preview

# Activate (fast reset) an existing worktree
abu worktree activate alpha                       # reset to master
abu worktree activate alpha --base develop        # reset to develop
abu worktree activate alpha --dry-run             # preview

# List worktrees
abu worktree list
```

The justfile provides shorthand aliases: `just worktree-create`,
`just worktree-remove`, `just worktree-refresh`, `just worktree-activate`,
`just worktree-list`. These all delegate to `abu worktree`.

**Activate vs Remove+Create:** `abu worktree activate` is much faster than
removing and recreating a worktree. It uses `git reset --hard` instead of full
worktree removal/creation, and incrementally syncs gitignored directories
(comparing mtime/size) instead of performing full APFS clones.

**Port allocation:** Each worktree gets a unique TCP port (starting at 10000)
stored in `~/dreamtides-worktrees/.ports.json`. Ports are allocated on create
and deallocated on remove.

**Opening in Unity:** After creating a worktree, launch Unity with
`abu open <name>` to open it with a per-worktree log file.

## Worktree Support

ABU supports running multiple Unity editors concurrently — one per git worktree.
Each editor gets its own TCP port, state file, and log file, with no manual
configuration required.

### Port Registry

`abu worktree create <name>` allocates a unique port (starting at 10000) in
`~/dreamtides-worktrees/.ports.json`. `abu worktree remove <name>` deallocates
it. The main repo always uses port 9999.

```json
// ~/dreamtides-worktrees/.ports.json
{
  "alpha": 10000,
  "beta": 10001
}
```

### Auto-Detection

Both the Python CLI (`abu.py`) and C# bridge (`AbuBridge.cs`) auto-detect their
worktree context:

1. **Python side**: `resolve_port()` checks `ABU_PORT` env var first, then
   detects if the repo root is under `~/dreamtides-worktrees/`, reads the
   worktree name, and looks up its port in `.ports.json`.
2. **C# side**: `AbuBridge.Awake()` checks env vars first, then calls
   `ResolveWorktreePort()` which reads `Application.dataPath` to detect the
   worktree and looks up the port.

### PID-Targeted Hammerspoon

Editor commands (refresh, play, test, cycle, restart) use the `unityPid` from
`.abu-state.json` to target the correct Unity process via
`hs.application.applicationForPID(pid)`. This ensures menu commands reach the
right editor when multiple are running. Falls back to bundle-ID search when the
PID is unavailable.

### Per-Editor Log Files

Each editor writes its log file path to `.abu-state.json` (the `logFile` field).
The Python CLI reads this to poll the correct log. When `abu restart` relaunches
a worktree editor, it passes
`-logFile ~/dreamtides-worktrees/<name>/Logs/Editor.log` to Unity, ensuring each
editor has its own log. `check_log_conflict()` detects and errors if two live
editors share a log path.

### Window Tinting

Worktree Unity editors display a colored strip at the top of the window so they
are visually distinguishable from the main editor. Two components work together:

1. **C# title prefix** (`WorktreeWindowTitle.cs`): An `[InitializeOnLoad]`
   editor script that detects whether the project lives under
   `~/dreamtides-worktrees/` and, if so, prefixes the window title with the
   uppercased worktree name (e.g. `[ALPHA] Unity - ...`). The main repo editor
   is left untouched.

2. **Hammerspoon tinting** (`scripts/abu/unity_tint.lua`): A Lua module loaded
   by `~/.hammerspoon/init.lua` that watches for Unity windows whose title
   starts with `[NAME]`. It draws a 1px colored `hs.canvas` strip at the top
   edge of matching windows. Each worktree gets a stable color from a 6-color
   palette (cornflower blue, amber, violet, rose, teal, gold) based on a hash of
   the name. The strip follows window move/resize and is cleaned up on close.

To reload after changes: `hs.reload()` in the Hammerspoon console, or click
"Reload Config" from the Hammerspoon menu bar icon.

### Status Display

`abu status` shows the worktree name (or "main repo") and the resolved port:

```
Unity Editor Status
========================================
  Worktree:      alpha
  State file:    ok
  Unity PID:     12345 (running)
  Play mode:     inactive
  Game mode:     Battle
  Last updated:  2026-02-23T12:00:00Z
  TCP (:10000):  unreachable
```

## Adding DreamtidesAbuSetup to a Scene

1. Create a `GameObject` in the scene (or use the Registry prefab).
2. Add the `DreamtidesAbuSetup` component.
3. Optionally assign the `Registry` reference in the Inspector (auto-discovered
   via `FindFirstObjectByType` if left null).
4. Enter Play mode. Unity logs `[Abu] TCP server listening on port 9999` and
   then `[Abu] Client connected.` each time the CLI connects.

The component creates an `AbuBridge` GameObject with `DontDestroyOnLoad`, so it
persists across scene loads.

## Architecture

The system has two halves: a Python CLI and a set of C# classes inside Unity.

- The **Python CLI** (`scripts/abu/abu.py`) builds an NDJSON command, opens a
  one-shot TCP connection to Unity, sends the command, reads one response line,
  and exits.
- **TcpServer** (`TcpServer.cs`) runs background threads for accepting
  connections and reading data. Received commands are enqueued to a
  `ConcurrentQueue`.
- **AbuBridge** (`AbuBridge.cs`) is a `MonoBehaviour` that drains the queue each
  `Update()` frame and dispatches commands to the handler.
- **SnapshotCommandHandler** (`SnapshotCommandHandler.cs`) handles all commands
  (snapshot, click, hover, drag, screenshot). For snapshot, it runs the walker,
  formats the result, and responds synchronously. For action commands
  (click/hover/drag), it invokes the ref callback, then starts a coroutine that
  polls `IsSettled()` each frame and responds with a fresh snapshot once
  settled.
- **SnapshotFormatter** (`SnapshotFormatter.cs`) performs a DFS pre-order walk
  of the scene node tree, producing ARIA-style indented text and a refs
  dictionary.
- **RefRegistry** (`RefRegistry.cs`) assigns monotonically increasing ref
  strings (`e1`, `e2`, ...) to interactive nodes during the walker's DFS
  traversal.
- **CommandSchema** (`CommandSchema.cs`) defines the wire protocol types:
  `AbuCommand`, `AbuResponse`, `SnapshotData`, `ActionSnapshotData`,
  `SnapshotRef`, and the param types.
- **DreamtidesSceneWalker** (`DreamtidesSceneWalker.cs`) implements
  `ISceneWalker` and traverses all three Dreamtides UI systems (UI Toolkit, 3D
  Displayables, CanvasButtons).
- **DreamtidesSettledProvider** (`DreamtidesSettledProvider.cs`) implements
  `ISettledProvider` and waits for `!ActionService.IsProcessingCommands` AND
  `DOTween.TotalPlayingTweens() == 0` held for 3 consecutive frames (with a 3s
  timeout fallback).
- **HistoryRecorder** (`HistoryRecorder.cs`) implements `IHistoryProvider` and
  observes `ActionService.OnCommandProcessed` to produce per-action history
  entries: game messages (turn begins, victory/defeat), dreamwell activations,
  shuffle events, and card zone transitions derived from `UpdateBattle` diffs.
- **IHistoryProvider** (`IHistoryProvider.cs`) is the interface in
  `namespace Abu` with two methods: `ClearHistory()` (called at action dispatch)
  and `TakeHistory()` (called after settle; returns null if no events occurred).
- **DreamtidesAbuSetup** (`DreamtidesAbuSetup.cs`) is a `MonoBehaviour` that
  wires the walker, settled provider, and history recorder to the bridge on
  `Start()`. Subscribes `_historyRecorder.OnCommand` to
  `ActionService.OnCommandProcessed` and unsubscribes in `OnDestroy()`.

All C# compiles under the `Dreamtides` assembly (`Dreamtides.asmdef`). There is
no separate Abu assembly definition. Core files use `namespace Abu`; the three
Dreamtides integration files (`DreamtidesAbuSetup`, `DreamtidesSceneWalker`,
`DreamtidesSettledProvider`) use `namespace Dreamtides.Abu` with `using Abu;`.

## Python CLI

`scripts/abu/abu.py` is the main CLI entry point using only Python stdlib (no
pip dependencies). Worktree lifecycle commands live in `scripts/abu/worktree.py`
and are registered as a subcommand group. Key functions in `abu.py`:

- `build_params(args)` — converts argparse namespace to wire params dict
- `build_command(command, params)` — wraps params in `{id, command, params}`
  NDJSON
- `send_command(command, params, port)` — one-shot TCP connect/send/recv
- `handle_response(command, response)` — extracts output; decodes base64 for
  screenshot
- `run_hs(lua_code)` — execute Lua via Hammerspoon CLI
- `send_menu_item(path)` — drive Unity menu bar via Hammerspoon
- `wait_for_refresh(log_offset)` — poll Editor log for refresh completion
- `wait_for_tests(log_offset)` — poll Editor log for test run completion
- `find_unity_process()` — discover running Unity via `ps`
- `do_refresh()`, `do_test()`, `do_cycle()`, `do_restart()`, `do_status()`,
  `do_clear_save()`, `do_set_mode()`, `do_create_save()` — high-level editor
  workflows
- `main()` — entry point; dispatches editor or TCP commands

Error handling uses an `AbuError` hierarchy (`ConnectionError`, `TimeoutError`,
`EmptyResponseError`, `HammerspoonError`, `UnityNotFoundError`,
`RefreshTimeoutError`, `CompilationError`). Errors print to stderr with exit
code 1.

Python style: shebang `#!/usr/bin/env python3`, module docstring, stdlib only,
all type hints, `main() -> None`, `if __name__ == "__main__": main()`.

## C# Transport

**TcpServer** (`TcpServer.cs`) manages two background threads: an accept thread
(`AcceptLoop`) and a per-connection read thread (`ReadLoop`). A new client
connection replaces the previous one. `Send()` uses a `_clientLock` to write
safely from the main thread while the accept thread may be replacing the client.

**AbuBridge** (`AbuBridge.cs`) is a `MonoBehaviour`. `Awake()` reads `ABU_PORT`
(then `ABU_WS_PORT` as fallback) to configure the port. `Update()` drains
`ReceiveQueue` each frame. `OnDestroy()` calls `TcpServer.Shutdown()`.

## C# Command Handling

**SnapshotCommandHandler** (`SnapshotCommandHandler.cs`) dispatches all
commands:

- `HandleSnapshot()`: clears registry, parses compact flag, calls
  `BuildSnapshotData(compact)`, responds immediately (synchronous).
- `BuildSnapshotData(compact)`: calls each `ISceneWalker.Walk()`, wraps results
  in an `application` root node, calls `SnapshotFormatter.Format()`.
- `DispatchRefAction()`: looks up ref in registry, invokes callback, starts
  `WaitForSettledThenRespond` coroutine.
- `WaitForSettledThenRespond()`: polls `IsSettled()` each frame; when settled,
  rebuilds snapshot (non-compact) and responds with `ActionSnapshotData`.
- `CaptureScreenshot()`: waits for end-of-frame, captures PNG, base64-encodes.

**SnapshotFormatter** (`SnapshotFormatter.cs`): DFS pre-order walk. Per node:
2-space indent per depth level, `"- {role}"`, optional `" \"{label}\""` if
non-null/non-empty, optional `" [ref=eN]"` if interactive. Compact mode omits
nodes where all of: non-interactive, empty label, no interactive descendants.

**RefRegistry** (`RefRegistry.cs`): `Register(callbacks)` assigns the next ref
string and stores callbacks. `Clear()` resets counter to 1. Called in the same
DFS order as `SnapshotFormatter`, so ref strings align.

**CommandSchema** (`CommandSchema.cs`) defines the wire types:

| Type                 | Wire shape                            | Notes                                                                    |
| -------------------- | ------------------------------------- | ------------------------------------------------------------------------ |
| `AbuCommand`         | `{id, command, params}`               | `params` is `JObject?`                                                   |
| `AbuResponse`        | `{id, success, data?, error?}`        |                                                                          |
| `SnapshotData`       | `{snapshot, refs, history?}`          | `refs` is `Dictionary<string, SnapshotRef>`; `history` omitted when null |
| `ActionSnapshotData` | inherits `SnapshotData` + action flag | Action fields merged via `JsonExtensionData`                             |
| `SnapshotRef`        | `{role, name}`                        | Entry in refs dict                                                       |
| `SnapshotParams`     | `{interactive?, compact?, maxDepth?}` | `maxDepth` parsed but not implemented                                    |
| `RefParams`          | `{ref}`                               | For click/hover                                                          |
| `DragParams`         | `{source, target?}`                   | For drag                                                                 |

Interfaces:

| Interface          | File                  | Contract                                                              |
| ------------------ | --------------------- | --------------------------------------------------------------------- |
| `ISceneWalker`     | `ISceneWalker.cs`     | `Walk(RefRegistry) → AbuSceneNode`                                    |
| `ISettledProvider` | `ISettledProvider.cs` | `IsSettled() → bool`, `NotifyActionDispatched()`                      |
| `ICommandHandler`  | `ICommandHandler.cs`  | `HandleCommand(command, bridge, onComplete)`                          |
| `IHistoryProvider` | `IHistoryProvider.cs` | `ClearHistory()`, `TakeHistory() → List<string>?`; in `namespace Abu` |

## Wire Protocol

Each message is one JSON object on a single line (NDJSON). The CLI sends exactly
one command and reads exactly one response per process invocation.

**Request** (CLI → Unity):

```json
{"id": "<uuid4>", "command": "<action>", "params": {...}}
```

**Response** (Unity → CLI):

```json
{"id": "<uuid4>", "success": true, "data": {...}}
{"id": "<uuid4>", "success": false, "error": "<message>"}
```

**Response data shapes by command:**

| Command      | `data` shape                                                                  |
| ------------ | ----------------------------------------------------------------------------- |
| `snapshot`   | `{"snapshot": "<ARIA text>", "refs": {"e1": {"role": "...", "name": "..."}}}` |
| `click`      | `{"clicked": true, "snapshot": "...", "refs": {...}, "history": [...]}`       |
| `hover`      | `{"hovered": true, "snapshot": "...", "refs": {...}, "history": [...]}`       |
| `drag`       | `{"dragged": true, "snapshot": "...", "refs": {...}, "history": [...]}`       |
| `screenshot` | `{"base64": "<base64-encoded PNG>"}`                                          |

The `history` key is present only on action responses (click/hover/drag) and
only when game events occurred. It is absent on snapshot responses and absent
(not null) when the action produced no observable events. The CLI prints history
entries before the snapshot when present, separated by `--- History ---` / `---`
delimiters.

Action commands (click/hover/drag) return a post-action snapshot after the UI
settles — refs in this snapshot are fresh and supersede any previous snapshot.

## Snapshot Format

```
- application "Dreamtides"
  - region "UIToolkit"
    - button "End Turn" [ref=e1]
    - group "Hand"
      - button "Lightning Bolt" [ref=e2]
      - button "Shield Wall" [ref=e3]
  - region "Scene3D"
    - button "Undo" [ref=e4]
    - group "Battlefield"
      - button "Fire Elemental" [ref=e5]
```

Refs (`e1`, `e2`, ...) are monotonically assigned in DFS pre-order per snapshot
and invalidated by the next snapshot or any action command.

**Compact mode** (`--compact`): omits nodes that are non-interactive, have no
label, and have no interactive descendants. Labeled and interactive nodes are
always included.

## Dreamtides UI Systems

The `DreamtidesSceneWalker` traverses three UI systems in every snapshot:

**UI Toolkit** (Masonry renderer): Starts at
`DocumentService.RootVisualElement`. Interactive elements have
`pickingMode == PickingMode.Position`. Click is dispatched via
`Callbacks.OnClick(ClickEvent.GetPooled())`; hover via
`Callbacks.OnMouseEnter(MouseEnterEvent.GetPooled())`.

**3D Displayables**: All active `Displayable` subclasses where
`CanHandleMouseEvents()` returns true (`Card`, `ActionButton`,
`DisplayableButton`, `CardBrowserButton`). Click is simulated by injecting a
`DisplayableClickInputProvider` into `InputService.InputProvider` for a
two-frame press/release sequence. Hover calls `displayable.MouseHoverStart()`.

**CanvasButtons**: The four fixed buttons on `DocumentService` (`MenuButton`,
`UndoButton`, `DevButton`, `BugButton`). Included only when
`gameObject.activeSelf && _canvasGroup.alpha > 0`. Click via `button.OnClick()`.

**Occlusion**: When `DocumentService.HasOpenPanels` is true, the entire Scene3D
subtree is empty. UI Toolkit elements remain traversable.

In battle mode, the walker covers user/opponent groups
(status/deck/identity/void/battlefield/hand/dreamwell), stack, game modifiers,
action buttons, essence label, and thinking indicator. In non-battle mode, it
traverses UIToolkit overlays and 3D scene elements.

## Conventions

- **Two namespaces coexist**: core files use `namespace Abu`; Dreamtides
  integration files (`DreamtidesAbuSetup`, `DreamtidesSceneWalker`,
  `DreamtidesSettledProvider`, `HistoryRecorder`) use `namespace Dreamtides.Abu`
  with `using Abu;`. Game-agnostic interfaces (`IHistoryProvider`,
  `ISceneWalker`, `ISettledProvider`) live in `namespace Abu`. Do not change
  either namespace.
- **No separate asmdef**: All Abu C# files compile under `Dreamtides.asmdef`.
  Never add a nested `.asmdef` inside `client/Assets/Dreamtides/Abu/`.
- **Ref assignment is DFS pre-order**: `RefRegistry` assigns refs during
  `ISceneWalker.Walk()` and `SnapshotFormatter` assigns display refs during
  `Format()`. Both use the same DFS order on the same tree, so ref strings
  align. Never change walk order without updating both.
- **Snapshot shape**: `SnapshotData` carries `Snapshot` (string), `Refs` (dict),
  and optional `History` (list). `ActionSnapshotData` inherits `SnapshotData`
  and merges action fields via `JsonExtensionData`. Action snapshots are never
  compact. The `history` key is omitted entirely (not null) when no events
  occurred.
- **History lifecycle (reset-collect-drain)**: `ClearHistory()` is called
  immediately before `NotifyActionDispatched()`. `OnCommand()` accumulates
  entries while the action processes. `TakeHistory()` is called after
  `IsSettled()` returns true. All three steps must occur for correct per-action
  scoping.
- **MonoBehaviour lifecycle for observers**: Subscribe to delegates in `Start()`
  (not `Awake()`), after `Registry` discovery. Always unsubscribe in
  `OnDestroy()`. `DreamtidesAbuSetup` demonstrates the correct pattern.
- **Port configuration**: default port 9999 for the main repo. Worktree ports
  are allocated starting at 10000 in `~/dreamtides-worktrees/.ports.json`. Both
  `AbuBridge.cs` and `abu.py` auto-detect the worktree port. `ABU_PORT` env var
  overrides auto-detection. Legacy `ABU_WS_PORT` is also accepted in
  `AbuBridge.cs` as fallback.
- **Python style**: shebang `#!/usr/bin/env python3`, module docstring, stdlib
  only, all type hints, `main() -> None`, `if __name__ == "__main__": main()`.
- **Error handling**: `AbuError` hierarchy in `abu.py` (`ConnectionError`,
  `TimeoutError`, `EmptyResponseError`). Print to stderr, exit code 1 on error.
- **C# glob scope**: scope C# globs to `client/Assets/`, not `client/`. Unity's
  `client/Library/` directory contains package caches that pollute glob results.
- **Do not modify**: `DreamtidesSceneWalker.cs` and
  `DreamtidesSettledProvider.cs` are large and working; avoid modifications
  unless necessary.

## Common Pitfalls

- **Stale refs**: refs are invalidated after every snapshot or action command.
  Agents must re-parse `refs` from each response before making the next call.
- **Compact mode omission**: a node is omitted in compact mode only when it is
  non-interactive, has no non-empty label, AND has no interactive descendants.
  All three conditions must hold.
- **`BusyToken` scope**: acquire `BusyToken` to suppress
  `DefaultSettledProvider` during multi-step coroutines; dispose it when the
  coroutine completes.
- **TCP single-client model**: a new CLI connection replaces the previous one.
  Do not assume the connection remains open across commands.

## Adapting ABU to Another Game

ABU is game-agnostic. Two interfaces define the pluggable contract:

**`ISceneWalker`** (`client/Assets/Dreamtides/Abu/ISceneWalker.cs`): Implement
`Walk(RefRegistry) → AbuSceneNode`. Register interactive nodes with
`refRegistry.Register(callbacks)` where `RefCallbacks` provides nullable
`OnClick`, `OnHover`, and `OnDrag` actions.

**`ISettledProvider`** (`client/Assets/Dreamtides/Abu/ISettledProvider.cs`):
Implement `IsSettled() → bool` and `NotifyActionDispatched()`. The default
(`DefaultSettledProvider`) waits 3 frames using `BusyToken.IsAnyActive`.
Override to add game-specific conditions.

Wire them up in a `MonoBehaviour.Start()` by calling
`bridge.RegisterWalker(new MyGameSceneWalker())` and
`bridge.SetSettledProvider(new MyGameSettledProvider())` on the `AbuBridge`
instance.

## Testing and Validation

```sh
# Python CLI unit tests (run from the scripts/abu/ directory)
cd scripts/abu && python3 -m unittest test_abu -v

# C# bridge tests (includes SnapshotFormatterTests, CommandSchemaTests, etc.)
just cli-unity-test

# C# formatting
just fmt-csharp
```

Do not use `pytest` — it is not installed. The test module is invoked by name
(`test_abu`), not by path, because the working directory must be `scripts/abu/`
for the `from abu import ...` import in the test file to resolve.

**Python tests** (`scripts/abu/test_abu.py`): unittest-based covering both TCP
communication (mock server roundtrip tests) and editor control functions
(worktree detection, log parsing, refresh polling, parser configuration,
exception hierarchy). Worktree module tests (`scripts/abu/test_worktree.py`)
cover argument parsing, exclusion filters, port allocation/deallocation, path
resolution, and dispatch routing.

**C# tests** (`client/Assets/Dreamtides/Tests/Abu/`):

- `SnapshotFormatterTests.cs` — 14 tests covering all formatter modes; ported
  from the original TypeScript test suite
- `CommandSchemaTests.cs` — `SnapshotData`/`ActionSnapshotData` serialization
- `WebSocketMessageTests.cs` — `AbuResponse` serialization
- `RefRegistryTests.cs` — Ref assignment and lookup
- `BusyTokenTests.cs` — Ref counting, dispose semantics
- `SceneWalkerTests.cs` — `DreamtidesSceneWalker` integration
- `SettledProviderTests.cs` — `DreamtidesSettledProvider` settle conditions
- `InputSimulationTests.cs` — Click/hover input simulation

Integration between Python and C# is not covered by automated tests. Validate
manually by entering Unity Play mode with `DreamtidesAbuSetup` attached and
running `python3 scripts/abu/abu.py snapshot`.

## Test Save Generation

The `create-save` command generates save files with custom battle parameters for
debugging and testing specific cards or scenarios without playing through a game
manually.

```sh
# Create a save with 99 energy and Break the Sequence in hand
python3 scripts/abu/abu.py create-save --energy 99 --card "Break the Sequence"

# Multiple cards can be added
python3 scripts/abu/abu.py create-save --energy 50 --card "Abolish" --card "Apocalypse"

# List all available card names
python3 scripts/abu/abu.py create-save --list-cards

# Create a save with just extra energy (default 5-card hand from Core11 deck)
python3 scripts/abu/abu.py create-save --energy 20
```

**How it works:** The command builds and runs the `test_save_generator` Rust
binary (`rules_engine/src/test_save_generator/`). This binary loads the
production card database (Tabula), creates a new battle with Core11 decks and
the default dreamwell, then applies the requested modifications (energy, cards
in hand). The save file is written to the standard Dreamtides save directory.

**Behavior details:**

- Existing save files are cleared before generating a new one, so the game loads
  the test save on next connect.
- The battle starts in the Main phase with player one active. Cards are drawn
  normally (5 cards), then requested cards are moved from the deck to hand on
  top of the initial draw.
- Energy sets both `current_energy` and `produced_energy` so it persists across
  turns.
- Card names are case-insensitive. Misspelled names trigger a "did you mean?"
  suggestion.
- Cards must be in the Core11 deck to be moved to hand. Use `--list-cards` to
  see all available card names in the database.

**Typical workflow:** Clear save → create test save → enter play mode → snapshot
to verify the test state.

```sh
python3 scripts/abu/abu.py create-save --energy 99 --card "Break the Sequence"
python3 scripts/abu/abu.py play
just abu --wait 30 snapshot --compact
```

## Related Documents

- [abu_development.md](abu_development.md): Step-by-step guide to modifying the
  scene walker, adding new UI features, and interactively testing changes using
  the Unity/CLI workflow. Read when making changes to Abu or adding support for
  new game UI elements.
