# ABU Integration

ABU (Agent-Browser for Unity) enables AI agents to interact with the Dreamtides
Unity client through the
[agent-browser](https://github.com/vercel-labs/agent-browser) CLI. The ABU
package lives at `~/abu/`; the Dreamtides-specific integration lives at
`client/Assets/Dreamtides/Abu/`.

## Running ABU

```sh
# 1. Build the daemon (one-time setup)
cd ~/abu/daemon && pnpm install && pnpm build

# 2. Set the home directory so the CLI finds the daemon
export AGENT_BROWSER_HOME=~/abu/daemon/

# 3. Open Unity and enter Play mode with DreamtidesAbuSetup on a GameObject.
#    The daemon starts automatically when you first run a CLI command.

# 4. Interact with the running game
agent-browser snapshot              # ARIA-style scene tree
agent-browser snapshot --compact    # abbreviated tree (interactive nodes only)
agent-browser click @e3             # click element with ref e3
agent-browser hover @e1             # hover element
agent-browser drag @e2 @e5          # drag source to target
agent-browser screenshot            # save PNG, print path
```

Unity connects to the daemon via WebSocket on port 9999. Set `ABU_WS_PORT` to
use a different port (must match in both daemon env and Unity env).

## Adding DreamtidesAbuSetup to a Scene

1. Create a `GameObject` in the scene (or use the Registry prefab).
2. Add the `DreamtidesAbuSetup` component.
3. Optionally assign the `Registry` reference in the Inspector (auto-discovered
   via `FindFirstObjectByType` if left null).
4. Enter Play mode. Unity logs `[Abu] WebSocket connected.` when the daemon
   connection is established.

The component creates an `AbuBridge` GameObject with `DontDestroyOnLoad`, so it
persists across scene loads.

## Architecture

```
agent-browser CLI
      │  NDJSON / Unix socket
      ▼
~/abu/daemon/dist/daemon.js      ← TypeScript Node.js process
      │  JSON / WebSocket (port 9999)
      ▼
AbuBridge MonoBehaviour          ← ~/abu/Runtime/AbuBridge.cs
      │
      ├── DreamtidesSceneWalker  ← client/Assets/Dreamtides/Abu/DreamtidesSceneWalker.cs
      │     Walks UI Toolkit, 3D Displayables, CanvasButtons
      │
      └── DreamtidesSettledProvider  ← client/Assets/Dreamtides/Abu/DreamtidesSettledProvider.cs
            Waits for !ActionService.IsProcessingCommands
            AND DOTween.TotalPlayingTweens() == 0
            for 3 consecutive frames (or 3s timeout)
```

## Dreamtides UI Systems

The `DreamtidesSceneWalker` traverses three UI systems in every snapshot:

**UI Toolkit** (Masonry renderer): Starts at
`DocumentService.RootVisualElement`. Interactive elements have
`pickingMode == PickingMode.Position`. `Draggable` elements are always
non-interactive (the drag system is a stub). Click is dispatched via
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

## Snapshot Example

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

Refs (`e1`, `e2`, ...) are monotonically assigned per snapshot and invalidated
by the next snapshot.

## Adapting ABU to Another Game

ABU is game-agnostic. Two interfaces define the pluggable contract:

**`ISceneWalker`** (`~/abu/Runtime/ISceneWalker.cs`): Implement
`Walk(RefRegistry) → AbuSceneNode`. Register interactive nodes with
`refRegistry.Register(callbacks)` where `RefCallbacks` provides nullable
`OnClick`, `OnHover`, and `OnDrag` actions.

**`ISettledProvider`** (`~/abu/Runtime/ISettledProvider.cs`): Implement
`IsSettled() → bool` and `NotifyActionDispatched()`. The default
(`DefaultSettledProvider`) waits 3 frames. Override to add game-specific
conditions (e.g., server response processing).

Wire them up in a `MonoBehaviour.Start()`:

```csharp
var bridge = gameObject.AddComponent<AbuBridge>();
bridge.RegisterWalker(new MyGameSceneWalker());
bridge.SetSettledProvider(new MyGameSettledProvider());
```

## Package Integration

The ABU package is referenced in `client/Packages/manifest.json` as a local UPM
path:

```json
"com.abu.bridge": "file:/Users/dthurn/abu"
```

It depends only on `com.unity.nuget.newtonsoft-json:3.2.1`, which is already
present in Dreamtides. The package assembly is listed in `testables` so
`~/abu/Tests/` runs via `just unity-tests`.

## Validation

```sh
# Daemon
cd ~/abu/daemon && pnpm build && pnpm test

# C# bridge
just unity-tests
just fmt-csharp
```

For full architecture details, see `~/abu/.claude/skills/abu/SKILL.md`.
