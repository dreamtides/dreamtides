# ABU Integration

ABU (Agent-Browser for Unity) enables AI agents to interact with the Dreamtides
Unity client through a Python CLI. Unity runs a TCP server on port 9999; the
CLI connects one-shot per command, sends NDJSON, reads the response, and exits.
All C# source lives in `client/Assets/Dreamtides/Abu/`.

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

```
Python CLI (scripts/abu/abu.py)
      │  NDJSON / TCP port 9999 (one-shot connection per command)
      ▼
TcpServer                        ← client/Assets/Dreamtides/Abu/TcpServer.cs
      │  ConcurrentQueue<AbuCommand>  (background → main thread)
      ▼
AbuBridge MonoBehaviour          ← client/Assets/Dreamtides/Abu/AbuBridge.cs
      │
      ├── SnapshotCommandHandler  ← client/Assets/Dreamtides/Abu/SnapshotCommandHandler.cs
      │     Dispatches snapshot/click/hover/drag/screenshot/launch/close
      │     Calls SnapshotFormatter after each walker run
      │
      ├── SnapshotFormatter       ← client/Assets/Dreamtides/Abu/SnapshotFormatter.cs
      │     DFS walk → ARIA-style text + refs dict
      │
      ├── DreamtidesSceneWalker   ← client/Assets/Dreamtides/Abu/DreamtidesSceneWalker.cs
      │     Walks UI Toolkit, 3D Displayables, CanvasButtons
      │
      └── DreamtidesSettledProvider ← client/Assets/Dreamtides/Abu/DreamtidesSettledProvider.cs
            Waits for !ActionService.IsProcessingCommands
            AND DOTween.TotalPlayingTweens() == 0
            for 3 consecutive frames (or 3s timeout)
```

## Wire Protocol

Each message is one JSON object on a single line (NDJSON).

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

| Command | `data` shape |
|---------|-------------|
| `snapshot` | `{"snapshot": "<ARIA text>", "refs": {"e1": {"role": "...", "name": "..."}}}` |
| `click` | `{"clicked": true, "snapshot": "...", "refs": {...}}` |
| `hover` | `{"hovered": true, "snapshot": "...", "refs": {...}}` |
| `drag` | `{"dragged": true, "snapshot": "...", "refs": {...}}` |
| `screenshot` | `{"base64": "<base64-encoded PNG>"}` |
| `launch` | `{"launched": true}` |
| `close` | `{"closed": true}` |

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

Wire them up in a `MonoBehaviour.Start()`:

```csharp
var bridge = FindFirstObjectByType<AbuBridge>();
if (bridge == null)
{
    var bridgeObject = new GameObject("AbuBridge");
    bridge = bridgeObject.AddComponent<AbuBridge>();
}
bridge.RegisterWalker(new MyGameSceneWalker());
bridge.SetSettledProvider(new MyGameSettledProvider());
```

## Validation

```sh
# Python CLI unit tests
python3 -m pytest scripts/abu/test_abu.py
# or: python3 scripts/abu/test_abu.py

# C# bridge tests (includes SnapshotFormatterTests, CommandSchemaTests, etc.)
just unity-tests
just fmt-csharp
```
