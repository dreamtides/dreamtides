# ABU (Agent-Browser for Unity)

ABU enables AI agents to interact with the Dreamtides Unity client through a
Python CLI. Unity runs a TCP server on port 9999; the CLI connects one-shot per
command, sends NDJSON, reads the response, and exits. All C# source lives in
`client/Assets/Dreamtides/Abu/`.

## Table of Contents

- [Running ABU](#running-abu)
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

## Running ABU

```sh
# 1. Open Unity and enter Play mode with DreamtidesAbuSetup on a GameObject.
#    Unity logs "[Abu] TCP server listening on port 9999" when ready.

# 2. Interact with the running game
python3 scripts/abu/abu.py snapshot              # ARIA-style scene tree
python3 scripts/abu/abu.py snapshot --compact    # abbreviated tree (interactive + labeled nodes)
python3 scripts/abu/abu.py click e1              # click element with ref e1
python3 scripts/abu/abu.py click @e1             # leading @ is stripped automatically
python3 scripts/abu/abu.py hover e1              # hover element
python3 scripts/abu/abu.py drag e2 e5            # drag source to target
python3 scripts/abu/abu.py screenshot            # save PNG, print path

# Override the default port
ABU_PORT=9998 python3 scripts/abu/abu.py snapshot
```

Unity listens on port 9999 by default. Set `ABU_PORT` (or the legacy
`ABU_WS_PORT`) to override.

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
- **DreamtidesAbuSetup** (`DreamtidesAbuSetup.cs`) is a `MonoBehaviour` that
  wires the walker and settled provider to the bridge on `Start()`.

All C# compiles under the `Dreamtides` assembly (`Dreamtides.asmdef`). There is
no separate Abu assembly definition. Core files use `namespace Abu`; the three
Dreamtides integration files (`DreamtidesAbuSetup`, `DreamtidesSceneWalker`,
`DreamtidesSettledProvider`) use `namespace Dreamtides.Abu` with `using Abu;`.

## Python CLI

`scripts/abu/abu.py` is a single-file CLI using only Python stdlib (no pip
dependencies). Key functions:

- `build_params(args)` — converts argparse namespace to wire params dict
- `build_command(command, params)` — wraps params in `{id, command, params}`
  NDJSON
- `send_command(command, params, port)` — one-shot TCP connect/send/recv
- `handle_response(command, response)` — extracts output; decodes base64 for
  screenshot
- `main()` — entry point; reads `ABU_PORT` env var

Error handling uses an `AbuError` hierarchy (`ConnectionError`, `TimeoutError`,
`EmptyResponseError`). Errors print to stderr with exit code 1.

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

| Type                 | Wire shape                            | Notes                                        |
| -------------------- | ------------------------------------- | -------------------------------------------- |
| `AbuCommand`         | `{id, command, params}`               | `params` is `JObject?`                       |
| `AbuResponse`        | `{id, success, data?, error?}`        |                                              |
| `SnapshotData`       | `{snapshot, refs}`                    | `refs` is `Dictionary<string, SnapshotRef>`  |
| `ActionSnapshotData` | inherits `SnapshotData` + action flag | Action fields merged via `JsonExtensionData` |
| `SnapshotRef`        | `{role, name}`                        | Entry in refs dict                           |
| `SnapshotParams`     | `{interactive?, compact?, maxDepth?}` | `maxDepth` parsed but not implemented        |
| `RefParams`          | `{ref}`                               | For click/hover                              |
| `DragParams`         | `{source, target?}`                   | For drag                                     |

Interfaces:

| Interface          | File                  | Contract                                         |
| ------------------ | --------------------- | ------------------------------------------------ |
| `ISceneWalker`     | `ISceneWalker.cs`     | `Walk(RefRegistry) → AbuSceneNode`               |
| `ISettledProvider` | `ISettledProvider.cs` | `IsSettled() → bool`, `NotifyActionDispatched()` |
| `ICommandHandler`  | `ICommandHandler.cs`  | `HandleCommand(command, bridge, onComplete)`     |

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
| `click`      | `{"clicked": true, "snapshot": "...", "refs": {...}}`                         |
| `hover`      | `{"hovered": true, "snapshot": "...", "refs": {...}}`                         |
| `drag`       | `{"dragged": true, "snapshot": "...", "refs": {...}}`                         |
| `screenshot` | `{"base64": "<base64-encoded PNG>"}`                                          |

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
  integration files use `namespace Dreamtides.Abu` with `using Abu;`. Do not
  change either namespace.
- **No separate asmdef**: All Abu C# files compile under `Dreamtides.asmdef`.
  Never add a nested `.asmdef` inside `client/Assets/Dreamtides/Abu/`.
- **Ref assignment is DFS pre-order**: `RefRegistry` assigns refs during
  `ISceneWalker.Walk()` and `SnapshotFormatter` assigns display refs during
  `Format()`. Both use the same DFS order on the same tree, so ref strings
  align. Never change walk order without updating both.
- **Snapshot shape**: `SnapshotData` carries `Snapshot` (string) and `Refs`
  (dict). `ActionSnapshotData` inherits `SnapshotData` and merges action fields
  via `JsonExtensionData`. Action snapshots are never compact.
- **Port configuration**: default port 9999; read from `ABU_PORT` env var in
  both `AbuBridge.cs` and `abu.py`. Legacy `ABU_WS_PORT` is also accepted in
  `AbuBridge.cs` as fallback.
- **Python style**: shebang `#!/usr/bin/env python3`, module docstring, stdlib
  only, all type hints, `main() -> None`, `if __name__ == "__main__": main()`.
- **Error handling**: `AbuError` hierarchy in `abu.py` (`ConnectionError`,
  `TimeoutError`, `EmptyResponseError`). Print to stderr, exit code 1 on error.
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
# Python CLI unit tests
python3 scripts/abu/test_abu.py
# or: python3 -m pytest scripts/abu/test_abu.py

# C# bridge tests (includes SnapshotFormatterTests, CommandSchemaTests, etc.)
just unity-tests

# C# formatting
just fmt-csharp
```

**Python tests** (`scripts/abu/test_abu.py`): unittest-based with a mock TCP
server for full roundtrip testing without Unity.

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

## Related Documents

- [abu_development.md](abu_development.md): Step-by-step guide to modifying the
  scene walker, adding new UI features, and interactively testing changes using
  the Unity/CLI workflow. Read when making changes to Abu or adding support for
  new game UI elements.
