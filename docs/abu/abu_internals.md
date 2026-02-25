# ABU Internals

Architecture, C# class details, wire protocol, coding conventions, and the
extension points for adapting ABU to another game. For usage instructions see
[abu.md](abu.md).

## Table of Contents

- [Architecture](#architecture)
- [Python CLI Internals](#python-cli-internals)
- [C# Transport](#c-transport)
- [C# Command Handling](#c-command-handling)
- [Wire Protocol](#wire-protocol)
- [Dreamtides UI Systems](#dreamtides-ui-systems)
- [Worktree Internals](#worktree-internals)
- [Conventions](#conventions)
- [Test Suite Details](#test-suite-details)
- [Adapting ABU to Another Game](#adapting-abu-to-another-game)

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
  `ISceneWalker` and dispatches to mode-specific walkers. `Walk()` calls
  `WalkBattle()` in battle mode or `WalkQuest()` in quest mode. The walker is
  split across partial classes: `DreamtidesSceneWalker.cs` (dispatch + shared
  helpers), `DreamtidesSceneWalker.Battle.cs` (battle mode), and
  `DreamtidesSceneWalker.Quest.cs` (quest mode).
- **DreamtidesSettledProvider** (`DreamtidesSettledProvider.cs`) implements
  `ISettledProvider` and waits for three conditions:
  `!ActionService.IsProcessingCommands`, `DOTween.TotalPlayingTweens() == 0`,
  and `!BusyToken.IsAnyActive` — held for 3 consecutive frames (with a 3s
  timeout fallback). `BusyToken` wraps async operations like camera transitions
  to suppress premature settle detection.
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

## Python CLI Internals

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
  `do_clear_save()`, `do_set_mode()`, `do_set_device()`, `do_create_save()` —
  high-level editor workflows
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

In battle mode (`WalkBattle()`), the walker covers user/opponent groups
(status/deck/identity/void/battlefield/hand/dreamwell), stack, game modifiers,
action buttons, essence label, and thinking indicator.

In quest mode (`WalkQuest()`), the walker produces a semantic tree under a
`"Quest"` region with these sections: Controls (menu, undo, dev, bug report,
close buttons), Map (site buttons derived from `DebugClickAction` labels),
Essence label, Quest Deck summary (via `CardBrowserButton`), Identity card,
Dreamsigns, Draft Picks, Shop, Tempting Offer (event offers), Start Battle,
Journey Choices, Quest Deck Browser, Card Order Selector, and filtered UIToolkit
overlays. When `DocumentService.HasOpenPanels` is true, map/essence/deck/card
sections are hidden and only Controls, Card Order Selector, and UIToolkit remain
visible.

## Worktree Internals

Implementation details for the worktree multi-editor support described in
[abu.md](abu.md#worktree-support).

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

Two components provide visual identification of worktree editors:

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

### Auto-Detection Details

1. **Python side**: `resolve_port()` checks `ABU_PORT` env var first, then
   detects if the repo root is under `~/dreamtides-worktrees/`, reads the
   worktree name, and looks up its port in `.ports.json`.
2. **C# side**: `AbuBridge.Awake()` checks env vars first, then calls
   `ResolveWorktreePort()` which reads `Application.dataPath` to detect the
   worktree and looks up the port.

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
- **Partial class architecture**: `DreamtidesSceneWalker` is split across three
  files: `DreamtidesSceneWalker.cs` (dispatch + shared helpers),
  `DreamtidesSceneWalker.Battle.cs` (battle mode), and
  `DreamtidesSceneWalker.Quest.cs` (quest mode). Add battle features to the
  Battle partial and quest features to the Quest partial. Shared helpers (e.g.
  `TryAddCloseButton`) live in the Quest partial or the base file.
- **`BusyToken` scope**: acquire `BusyToken` to suppress
  `DefaultSettledProvider` during multi-step coroutines; dispose it when the
  coroutine completes.

## Test Suite Details

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
