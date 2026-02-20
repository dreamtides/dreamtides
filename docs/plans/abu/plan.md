# Technical Design: ABU (Agent-Browser for Unity)

## Goal

Build a two-component system -- a TypeScript daemon and a Unity C# bridge --
that lets AI agents interact with Unity games through the unmodified
`agent-browser` CLI. The daemon replaces the browser-based daemon, translating
CLI commands into Unity operations over WebSocket. The Unity bridge walks the
scene, builds ARIA-style snapshots, and simulates input. ABU is a
general-purpose open-source package (`~/abu`); Dreamtides
(`~/Documents/GoogleDrive/dreamtides/`) is the first testbed.

## Background

### Agent-Browser CLI Protocol

The `agent-browser` CLI communicates with a daemon via Unix domain sockets using
newline-delimited JSON. Each CLI invocation opens a **new** socket connection,
sends one JSON request line, reads one response line, and closes. This is not a
persistent connection.

**Daemon discovery**: The CLI finds `daemon.js` via
`$AGENT_BROWSER_HOME/dist/daemon.js` (among other paths). ABU sets
`AGENT_BROWSER_HOME` to point at its daemon directory so the CLI launches the
ABU daemon instead of the browser daemon.

**Daemon startup**: The CLI spawns `node <daemon_path>` as a detached process
with `AGENT_BROWSER_DAEMON=1` and `AGENT_BROWSER_SESSION=<session>` environment
variables. The daemon must write `<session>.pid` before creating
`<session>.sock` in the socket directory. The CLI polls for these files for up
to 5 seconds.

**Socket directory**: `$AGENT_BROWSER_SOCKET_DIR` >
`$XDG_RUNTIME_DIR/agent-browser` > `~/.agent-browser/` (macOS default). Created
with 0700 permissions.

**Wire format**:

```
Request:  {"id": "<uuid>", "action": "<command>", ...params}\n
Success:  {"id": "<uuid>", "success": true, "data": <payload>}\n
Error:    {"id": "<uuid>", "success": false, "error": "<message>"}\n
```

CLI timeouts: 5s write, 30s read. Retries: 5x with 200ms exponential backoff.

### Commands to Implement

| Command      | Request params                          | Response data                 | Behavior                                                                       |
| ------------ | --------------------------------------- | ----------------------------- | ------------------------------------------------------------------------------ |
| `launch`     | `headless?`, `viewport?`                | `null`                        | No-op, returns success (Unity is already running)                              |
| `snapshot`   | `interactive?`, `compact?`, `maxDepth?` | `{"snapshot": "<aria-tree>"}` | Walk scene, build ARIA text tree                                               |
| `click`      | `selector` (e.g. `"@e3"`)               | `null`                        | Resolve ref, simulate click, wait for settled                                  |
| `hover`      | `selector` (e.g. `"@e3"`)               | `null`                        | Resolve ref, simulate hover, wait for settled                                  |
| `drag`       | `selector`, `target?`                   | `null`                        | Resolve source ref, simulate drag (optionally to target ref), wait for settled |
| `screenshot` | `selector?`, `annotate?`                | `{"screenshot": "<base64>"}`  | Capture screen as PNG, encode base64                                           |
| `close`      | (none)                                  | `null`                        | No-op or disconnect                                                            |

All other commands (fill, press, type, wait, ~100 browser-specific ones) return
an error or no-op in v0.1.

### Snapshot Format

The snapshot is an indented text tree using ARIA roles. Each line has a dash,
role, quoted name, and optional attributes. Interactive elements get ephemeral
refs (`[ref=eN]`) that the CLI passes back as `@eN` for click/hover/drag.

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
      - generic "Fire Elemental"
```

Refs are monotonically incrementing per snapshot. All refs from snapshot N are
invalidated by snapshot N+1. When multiple elements share role+name, add
`[nth=N]` to disambiguate.

### Dreamtides UI Systems

Dreamtides has two active UI systems that ABU must walk:

**UI Toolkit (Masonry renderer)**: A single `UIDocument` at
`DocumentService._document`
(`client/Assets/Dreamtides/Services/DocumentService.cs:24`). Root:
`DocumentService.RootVisualElement` (line 51). Four absolute-positioned
container layers: `InfoZoomContainer`, `ScreenOverlay`, `ScreenAnchoredNode`,
`EffectPreviewOverlay` (lines 63-66). Interactive elements have
`pickingMode == PickingMode.Position`, set by `MasonRenderer.ApplyNode()` when
`PressedStyle`, `HoverStyle`, or `EventHandlers` are non-null
(`client/Assets/Dreamtides/Masonry/MasonRenderer.cs:266-274`). Click callbacks
are stored in `Callbacks._actions[Event.Click]` with a public
`OnClick(ClickEvent)` method
(`client/Assets/Dreamtides/Masonry/Elements.cs:112`).

**3D GameObjects (Displayable hierarchy)**: `Displayable`
(`client/Assets/Dreamtides/Layout/Displayable.cs:16`) is the base class for all
3D interactive objects. `CanHandleMouseEvents()` (line 196, default false) is
the interactivity predicate. Key subclasses that override it to return true:
`Card` (line 622, except Deck/DiscardPile/InfoZoom contexts), `ActionButton`
(line 246), `DisplayableButton` (line 96), `CardBrowserButton` (line 22).
`CanvasButton` (`client/Assets/Dreamtides/Buttons/CanvasButton.cs:16`) extends
`Displayable` but does NOT override `CanHandleMouseEvents()` -- it uses Unity's
UGUI `Button.OnClick()` instead (line 39). Four CanvasButton instances exist on
`DocumentService`: `_menuButton`, `_undoButton`, `_devButton`, `_bugButton`
(lines 32-45).

**Occlusion rule**: When `DocumentService.HasOpenPanels` is true (line 53) or
`MouseOverDocumentElement()` returns true, 3D clicks are blocked
(`client/Assets/Dreamtides/Services/InputService.cs:181-187`). ABU should
respect this: when a UI Toolkit overlay is open, 3D Displayable objects should
be omitted from the snapshot or marked non-interactive.

**Input simulation**: `InputService` has an injectable `IInputProvider` property
(`client/Assets/Dreamtides/Services/InputService.cs:97`). This is the primary
mechanism for simulating 3D Displayable clicks -- inject a fake provider that
returns the target Displayable, then let
`InputService.HandleDisplayableClickAndDrag()` (line 130) drive the
`MouseDown()`/`MouseUp(isSameObject: true)` sequence. This is critical because
`DisplayableButton.MouseUp()` only fires its action when `isSameObject == true`.

## Design

### TypeScript Daemon

The daemon is a standalone Node.js process that bridges the agent-browser CLI
protocol to a Unity WebSocket connection. It lives at `~/abu/daemon/`.

**Lifecycle**: On startup, the daemon reads `AGENT_BROWSER_SESSION` from the
environment, writes `<session>.pid` to the socket directory, creates a Unix
domain socket at `<session>.sock`, and starts listening. It also starts a
WebSocket server on a configurable port (default 9999, `ABU_WS_PORT` environment
variable). It accepts connections from Unity clients on the WebSocket side and
short-lived connections from the CLI on the Unix socket side.

**Command routing**: When a CLI request arrives on the Unix socket, the daemon
forwards it to Unity over the WebSocket, waits for Unity's response (up to ~25
seconds, under the CLI's 30-second timeout), formats the response, and writes it
back to the Unix socket. If no Unity client is connected, the daemon returns an
error. Each command carries an `id` field for correlation.

**Snapshot formatting**: Unity sends the scene tree as structured JSON over the
WebSocket. The daemon converts this to the ARIA-style indented text format
described above. This keeps the text-formatting logic in TypeScript where string
manipulation is natural, and keeps the Unity side focused on scene walking.

**Screenshot handling**: Unity sends a base64-encoded PNG. The daemon passes it
through to the CLI response as-is.

**Project configuration**: ESM modules (`"type": "module"` in `package.json`).
TypeScript with `target: ES2022`, `module: NodeNext`, `strict: true`. Uses
`pnpm` as the package manager. Dependencies: `ws`, `@types/ws`, `zod` (for
request validation). No playwright-core. Output compiled to `dist/daemon.js`.

**Security**: The Unix socket server should reject HTTP requests (check if the
first bytes look like `GET /` or `POST /`) to prevent accidental web access.

### Unity C# Bridge

The bridge is a MonoBehaviour in the ABU UPM package that connects to the daemon
as a WebSocket client. It lives at `~/abu/Runtime/`.

**Threading model**: The WebSocket runs on a background thread using
`System.Net.WebSockets.ClientWebSocket`. A `ConcurrentQueue<T>` pattern bridges
between the background receive thread and the Unity main thread. Background
thread receives JSON messages and enqueues them. `Update()` on the main thread
dequeues commands, executes them (all Unity API calls must happen on the main
thread), and enqueues responses. A background send thread dequeues responses and
sends them over the WebSocket.

**Pluggable scene walker interface**: ABU defines a scene walker interface that
game-specific code implements. Walkers produce a list of nodes (role, label,
interactive flag, screen bounds, invoke/hover/drag callbacks). ABU ships default
walkers for UI Toolkit (walks `VisualElement` trees, checks
`pickingMode`/`focusable`) and UGUI (`Selectable.allSelectablesArray`). Game
developers register additional walkers for custom systems. Note: Unity's
`FindObjectsByType<T>()` requires a concrete `UnityEngine.Object` subtype -- it
cannot find objects by interface. Walkers must search for concrete types (e.g.,
`Displayable`) rather than relying on interface-based discovery.

**Ref registry**: After all walkers produce nodes, ABU assigns monotonically
incrementing refs (`e1`, `e2`, ...) to interactive elements. A registry maps ref
strings to the invoke/hover/drag callbacks. The registry is rebuilt on every
snapshot and invalidated on the next snapshot.

**Click dispatch**: When a `click` command arrives with a ref like `@e3`, the
bridge looks up `e3` in the ref registry and calls the associated invoke
callback. Different element types use different invoke strategies -- this is
determined by the walker that produced the node. After invocation, the bridge
enters a settling state before responding.

**Hover dispatch**: When a `hover` command arrives with a ref, the bridge looks
up the ref and calls the associated hover callback. For UI Toolkit elements,
this fires `MouseEnterEvent` on the target. For 3D Displayables, it calls
`MouseHoverStart()` and then `MouseHover()` for a brief period (a few frames) to
trigger hover-dependent behavior (e.g., info zoom after 0.15 seconds). The
previous hover target (if any) receives the corresponding leave/end event first.
After hover effects settle, the bridge responds.

**Drag dispatch**: When a `drag` command arrives with a source ref (and optional
target ref), the bridge simulates a full drag sequence: `MouseDown()` on the
source, multiple frames of `MouseDrag()` (moving toward the target position or a
sufficient distance to trigger play thresholds), then `MouseUp()`. For UI
Toolkit `Draggable` elements, it synthesizes the corresponding pointer events.
For 3D Displayables (e.g., cards), it injects a fake `IInputProvider` that
drives `HandleDisplayableClickAndDrag()` through the press-drag-release cycle.
If no target is specified, the bridge simulates dragging far enough to trigger
the element's default action (e.g., playing a card from hand). After the drag
sequence completes and the UI settles, the bridge responds.

**Settled detection**: ABU defines a pluggable `ISettledProvider` interface.
Games register a custom predicate for "the UI has settled." ABU ships a default
implementation that checks `DOTween.TotalPlayingTweens() == 0` plus N settle
frames (3-5) plus a maximum timeout (~3 seconds). Dreamtides has no looping
DOTween tweens (confirmed: zero `SetLoops` calls in the codebase), so the
default works today.

**Screenshot**: Uses `ScreenCapture.CaptureScreenshotAsTexture()`, encodes to
PNG bytes, then base64.

**UPM package**: Name `com.abu.bridge`, minimum Unity `6000.0`. Depends on
`com.unity.nuget.newtonsoft-json:3.2.1` (already present in Dreamtides).
Assembly definition with `overrideReferences: true` referencing
`Newtonsoft.Json.dll`.

### WebSocket Message Schema

The daemon and bridge must agree on a JSON message format. Messages flow
bidirectionally over a single persistent WebSocket connection.

**Daemon to Unity (commands)**:

```json
{"id": "uuid", "command": "snapshot", "params": {"interactive": true, "compact": false}}
{"id": "uuid", "command": "click", "params": {"ref": "e3"}}
{"id": "uuid", "command": "hover", "params": {"ref": "e3"}}
{"id": "uuid", "command": "drag", "params": {"ref": "e3", "target": "e5"}}
{"id": "uuid", "command": "screenshot", "params": {}}
```

**Unity to Daemon (responses)**:

```json
{"id": "uuid", "success": true, "data": {"nodes": [...]}}
{"id": "uuid", "success": true, "data": null}
{"id": "uuid", "success": false, "error": "Ref e3 not found"}
```

The `nodes` array for snapshots is a flat or nested JSON representation of the
scene tree. Each node has: `role` (string), `label` (string or null),
`interactive` (bool), `children` (array of nodes). The daemon converts this JSON
tree to the indented ARIA text format.

Formal Zod schemas on the TypeScript side and matching C# types on the Unity
side should be defined before parallel development begins. There are roughly 8
command types, so the schema is small.

### Dreamtides Integration

**Package reference**: Add `"com.abu.bridge": "file:/Users/dthurn/abu"` to
`client/Packages/manifest.json`.

**Dreamtides-specific walker**: A single C# file in the Dreamtides codebase (not
in ABU) implements the scene walker interface for Dreamtides' systems. This
walker:

1. Walks `DocumentService.RootVisualElement` depth-first, identifying
   interactive elements by `pickingMode == PickingMode.Position`. Note: the four
   top-level container layers (`InfoZoomContainer`, `ScreenOverlay`, etc.) have
   `pickingMode = PickingMode.Ignore` (set at `DocumentService.cs:338`), so the
   walker must recurse into them even though they are not themselves
   interactive. For click invocation, it can use
   `element.SendEvent(ClickEvent.GetPooled())` or fall back to direct
   `Callbacks.OnClick()` invocation (which is public) if synthesized events do
   not propagate correctly from `Update()`. For hover, it fires
   `MouseEnterEvent` on the target element, which triggers the Masonry
   `HoverStyle` application and any `OnMouseEnter` event handler. For drag on
   `Draggable` elements, it synthesizes the pointer-down/move/up event sequence
   to drive the UI Toolkit drag-and-drop system.

2. Finds all `Displayable` objects via `FindObjectsByType<Displayable>()`,
   filters by `CanHandleMouseEvents()`, extracts labels (e.g.,
   `Card.CardView.Revealed.Name`, `DisplayableButton._text.text`). For click
   invocation, it injects a fake `IInputProvider` into
   `InputService.InputProvider` that returns the target Displayable for one
   frame, triggering the `MouseDown()`/`MouseUp(isSameObject: true)` sequence.
   For hover, it calls `MouseHoverStart()` on the target Displayable (clearing
   any previous hover via `MouseHoverEnd()` on `InputService._lastHovered`),
   then calls `MouseHover()` across several frames to trigger time-dependent
   effects like card info zoom (which requires 0.15 seconds of sustained hover).
   For drag, it injects a fake `IInputProvider` that drives the full
   `MouseDown()` → `MouseDrag()` (across multiple frames with increasing
   distance) → `MouseUp()` sequence, simulating the press-drag-release cycle.
   When no target ref is specified, the drag distance exceeds the play threshold
   (0.5m on mobile, 1m+ on desktop) to trigger the card's play action.

3. Detects the four `CanvasButton` instances via `DocumentService.MenuButton`,
   `DocumentService.UndoButton`, etc. For click invocation, calls
   `CanvasButton.OnClick()` directly (respecting the 0.5-second debounce at
   `CanvasButton.cs:44`).

4. Respects the occlusion rule: when `DocumentService.HasOpenPanels` is true, 3D
   Displayables and CanvasButtons are omitted or marked non-interactive.

5. Registers a settled provider that checks
   `ActionServiceImpl.IsProcessingCommands == false`
   (`client/Assets/Dreamtides/Services/ActionServiceImpl.cs:78`) in addition to
   the default DOTween check.

**MonoBehaviour placement**: The ABU bridge MonoBehaviour needs to be added to
the scene (likely on the Registry prefab or as a separate `DontDestroyOnLoad`
object). The Dreamtides walker registers itself with ABU during initialization.

### Critical Problem: User Input Simulation

Sending clicks, hovers, and drags to Unity is the hardest problem in this
project. ABU must correctly deliver input to **three distinct systems** -- UI
Toolkit VisualElements, 3D Displayable GameObjects, and UGUI CanvasButtons --
each of which has its own event model, threading constraints, and edge cases.

We do **not** need to have a complete solution before starting, but we need to
treat this as the highest-risk area and **empirically validate** each input
pathway as early as possible via Unity editor tests (`just unity-tests`). The
plan for each input system is our best hypothesis; the editor tests are how we
confirm or refute it. Specifically:

- **UI Toolkit**: Can `element.SendEvent(ClickEvent.GetPooled())` be called from
  `Update()` and have it propagate correctly through the UI Toolkit event
  system? Or must we fall back to `Callbacks.OnClick()` direct invocation? We
  need an editor test that creates a VisualElement, registers a click callback,
  fires a synthesized ClickEvent, and asserts the callback was invoked.

- **3D Displayables**: Does injecting a fake `IInputProvider` into
  `InputService` and letting `HandleDisplayableClickAndDrag()` drive the
  `MouseDown()`/`MouseUp(isSameObject: true)` sequence actually work from a test
  context? We need an editor test that instantiates a `Displayable` subclass,
  simulates the click sequence, and asserts the expected side effect.

- **CanvasButtons**: Does calling `CanvasButton.OnClick()` directly work in
  editor tests, or does UGUI require a full EventSystem? We need an editor test
  that verifies this.

These three input validation tests should be among the **very first things
built** in the project. If any of the hypothesized approaches fail, we need to
discover that immediately and course-correct, not after building thousands of
lines of code on top of a broken assumption.

### Testing Strategy: Continuous Validation

**Core principle: Every step of this project must produce real-world evidence of
correctness.** It is *not* acceptable to write large amounts of C# code and
defer testing to the end. Every task that produces C# code must also produce
editor tests that run via `just unity-tests` and demonstrate the code actually
works. If a piece of C# code cannot be validated by an editor test, that is a
design problem to be solved before proceeding, not a testing gap to be accepted.

This means:

- No task is "done" until its editor tests pass via `just unity-tests`.
- If an approach cannot be tested in EditMode, redesign it so it can be, or
  escalate the problem.
- Agent implementors must run `just unity-tests` after every meaningful C#
  change, not just at milestones.

#### Verification Tools Available to Agents

| Tool                                  | What it checks                   | Latency      | Autonomous? |
| ------------------------------------- | -------------------------------- | ------------ | ----------- |
| `tsc --noEmit`                        | TypeScript type errors in daemon | ~2s          | Yes         |
| `pnpm test`                           | Daemon unit/integration tests    | ~5s          | Yes         |
| VS Code `getDiagnostics`              | C# type errors per file          | Instant      | Yes         |
| Unity batch mode (`-batchmode -quit`) | Full C# compilation              | ~60s         | Yes         |
| **`just unity-tests`**                | **EditMode NUnit tests**         | **~1-3 min** | **Yes**     |
| `just fmt-csharp`                     | C# formatting (csharpier)        | ~5s          | Yes         |
| ABU itself (once bootstrapped)        | End-to-end UI interaction        | ~5s/command  | Yes         |

Agents should run the fastest applicable check after every edit.
**`just unity-tests` is the primary correctness gate for all C# code** and must
pass after every task. The full suite (`just unity-tests` + `pnpm test`) is the
gate before each milestone.

#### Daemon Tests (Node.js)

Test the daemon independently with mock WebSocket clients using `pnpm test`.
Verify: PID file and socket creation, NDJSON request/response round-trips,
snapshot JSON-to-ARIA-text formatting, error handling for missing Unity client,
timeout behavior. These run without Unity.

#### Bridge Tests (Unity EditMode)

EditMode tests can programmatically create VisualElement trees and GameObjects
without a running scene. The ABU UPM package and the Dreamtides integration each
get their own test assembly (`.asmdef` with `UNITY_INCLUDE_TESTS` define,
editor-only platform). All tests run via `just unity-tests`.

**Gate 0 — Input simulation validation (build this FIRST):**

- UI Toolkit click: create a `VisualElement`, register a click callback, fire
  `SendEvent(ClickEvent.GetPooled())`, assert the callback was invoked. If this
  fails, test direct `Callbacks.OnClick()` invocation as a fallback.
- UI Toolkit hover: create a `VisualElement`, register a MouseEnter callback,
  fire `SendEvent(MouseEnterEvent.GetPooled())`, assert the callback was
  invoked.
- Displayable click: instantiate a `Displayable` subclass, inject a fake
  `IInputProvider`, trigger `HandleDisplayableClickAndDrag()`, assert the
  expected `MouseUp(isSameObject: true)` side effect fires.
- Displayable hover: instantiate a `Displayable` subclass, call
  `MouseHoverStart()` and `MouseHover()`, assert the expected hover side effects
  fire (e.g., info zoom display after the hover delay).
- Displayable drag: instantiate a `Displayable` subclass (e.g., `Card`), inject
  a fake `IInputProvider`, drive the `MouseDown()` → `MouseDrag()` → `MouseUp()`
  sequence across multiple frames, assert the expected drag completion effect
  fires (e.g., card play action).
- CanvasButton click: instantiate a `CanvasButton`, call `OnClick()` directly,
  assert the expected side effect.
- **This gate is the single most important deliverable in the project.** If any
  of these tests fail, stop and redesign before building anything else.

**Gate 1 — After WebSocket message types / command schema:**

- Round-trip serialization tests: serialize a command to JSON, deserialize it,
  assert field equality.
- Response serialization: build a success/error response, serialize, verify JSON
  structure.

**Gate 2 — After ref registry implementation:**

- Assign refs to a list of mock nodes, verify monotonic `e1`, `e2`, ... naming.
- Look up by `@eN` string, verify correct node returned.
- Invalidation: build a registry, call invalidate, verify all lookups fail.

**Gate 3 — After UI Toolkit walker:**

- Create a `VisualElement` tree in code: root with children, some with
  `pickingMode = PickingMode.Position`, others with `PickingMode.Ignore`.
- Run the walker, assert it finds exactly the interactive elements.
- Verify that containers with `pickingMode.Ignore` are recursed into but not
  themselves marked interactive.
- Verify roles and labels are extracted correctly.

**Gate 4 — After Displayable walker:**

- Instantiate GameObjects with `Displayable` subclass components (e.g., a
  minimal `ActionButton`).
- Run the walker, verify discovery via `FindObjectsByType<Displayable>()`.
- Verify `CanHandleMouseEvents()` filtering: add a Displayable that returns
  false, assert it is excluded.

**Gate 5 — After CanvasButton walker:**

- Instantiate a GameObject with a `CanvasButton` component.
- Run the walker, verify it is discovered and marked interactive.

**Gate 6 — After occlusion logic:**

- Set `HasOpenPanels = true` (or mock the condition), run the full walker
  pipeline, verify 3D Displayable and CanvasButton elements are excluded while
  UI Toolkit elements remain.

**Gate 7 — After click/hover/drag dispatch:**

- Register a mock callback via the ref registry, dispatch a click command to its
  ref, assert the callback was invoked.
- For UI Toolkit: create a `VisualElement` with a registered click handler,
  dispatch via the walker's invoke callback, verify the handler fired.
- Dispatch a hover command to a ref, verify the hover callback was invoked and
  any previous hover target received a leave/end event.
- Dispatch a drag command with source ref (and optionally target ref), verify
  the drag sequence callback was invoked and completed.

**Gate 8 — After settled detection:**

- Test with `DOTween.TotalPlayingTweens() == 0`: assert settled immediately.
- If testable: mock a non-zero tween count, verify the bridge waits before
  reporting settled.

**Gate 9 — After ConcurrentQueue threading model:**

- Enqueue a command on a background thread, verify it is dequeued and executed
  on the main thread in `Update()`.
- Enqueue a response from the main thread, verify it is dequeued on the
  background send thread.

#### End-to-End (Self-Hosting Milestone)

Once the basic `snapshot` command works end-to-end, ABU becomes self-testing.
Run the actual `agent-browser` CLI against the daemon with Unity running.
Verify: `agent-browser snapshot` returns a valid ARIA tree,
`agent-browser click @eN` activates the correct element,
`agent-browser hover @eN` triggers hover effects, `agent-browser drag @eN`
completes a drag action, `agent-browser screenshot` returns a valid PNG. After
this milestone, agents can use ABU to verify their own subsequent changes to
ABU.

## Constraints

- The `agent-browser` CLI is used unmodified. ABU must be fully
  protocol-compatible.
- All Unity API calls happen on the main thread. No Unity API calls from
  background WebSocket threads.
- ABU must be game-agnostic. All Dreamtides-specific code lives in the
  Dreamtides codebase, not in the ABU package.
- The ABU UPM package depends only on `com.unity.nuget.newtonsoft-json`. No
  other third-party Unity dependencies.
- The TypeScript daemon depends on `ws`, `@types/ws`, and `zod`. No
  playwright-core or browser dependencies. Use `pnpm` for all Node.js package
  management.
- ABU is designed for turn-based games or games without a major reflex element.
  Real-time twitch gameplay is out of scope.
- The `~/abu` repository does not exist yet. It must be created from scratch.
- WebSocket port is configurable via `ABU_WS_PORT` (default 9999). Fixed port
  for v0.1; no dynamic negotiation.
- All C# code must be continuously validated via `just unity-tests`. No task is
  complete without passing editor tests that demonstrate correctness.

## Non-Goals

- Modifying the `agent-browser` CLI or its source code.
- Supporting `fill`, `press`, `type`, or `wait` commands in v0.1 (they return
  errors or no-ops).
- Per-element occlusion testing (coarse `HasOpenPanels` flag is sufficient for
  v0.1; `IPanel.Pick()` deferred).
- Controller/gamepad input simulation.
- Dynamic port negotiation between daemon and Unity.
- Multiplayer or networked game support.
- Exposing internal game state to the AI (it interacts only through the UI).

## Open Questions

- **Input simulation across all three UI systems**: This is the project's
  highest-risk area. Each of the three UI systems (UI Toolkit, 3D Displayables,
  UGUI CanvasButtons) has a different event model, and our planned approaches
  for simulating click, hover, and drag input into each are hypotheses that must
  be validated empirically via editor tests before building on top of them. See
  the "Critical Problem: User Input Simulation" section above. Gate 0 exists
  specifically to validate or refute these hypotheses as the first deliverable.
- **Synthesized ClickEvent propagation**: Whether
  `element.SendEvent(ClickEvent.GetPooled())` works correctly when called from
  `Update()` outside the UI Toolkit event dispatch cycle needs to be verified
  via editor test. If it does not work, fall back to direct
  `Callbacks.OnClick()` invocation.
- **Drag simulation across multiple frames**: Drag requires driving
  `MouseDown()` → `MouseDrag()` (repeated) → `MouseUp()` across multiple
  `Update()` frames, unlike click which can fire in a single frame. The fake
  `IInputProvider` must report `IsPointerPressed() == true` for the correct
  number of frames and then switch to `false` to trigger release. The required
  drag distance varies: cards use 0.5m (mobile) or visual threshold (desktop)
  for play, and 0.25m to clear info zoom. We need to determine empirically how
  many frames and what positions the fake provider must report to reliably
  trigger drag-to-play.
- **Hover timing for 3D Displayables**: Card info zoom requires 0.15 seconds of
  sustained hover via `MouseHover()` calls. The hover simulation must span
  enough frames to exceed this threshold. The bridge needs to either wait the
  required duration before responding, or trigger the hover effect immediately
  by bypassing the timer check.
- **CanvasButton detection**: Whether to use `FindObjectsByType<CanvasButton>()`
  or `DocumentService`'s four direct references. Both work; the latter is more
  explicit for the four known instances.

## References

- Agent-browser CLI: https://github.com/vercel-labs/agent-browser
- Project prompt:
  `/Users/dthurn/Documents/GoogleDrive/dreamtides/docs/plans/abu/prompt.md`
- Unity UI research:
  `/Users/dthurn/Documents/GoogleDrive/dreamtides/docs/plans/abu/unity_ui_research.md`
- `DocumentService`:
  `/Users/dthurn/Documents/GoogleDrive/dreamtides/client/Assets/Dreamtides/Services/DocumentService.cs`
- `InputService` and `IInputProvider`:
  `/Users/dthurn/Documents/GoogleDrive/dreamtides/client/Assets/Dreamtides/Services/InputService.cs`
- `Displayable` base class:
  `/Users/dthurn/Documents/GoogleDrive/dreamtides/client/Assets/Dreamtides/Layout/Displayable.cs`
- `MasonRenderer` (pickingMode logic):
  `/Users/dthurn/Documents/GoogleDrive/dreamtides/client/Assets/Dreamtides/Masonry/MasonRenderer.cs`
- `Callbacks` (click handlers):
  `/Users/dthurn/Documents/GoogleDrive/dreamtides/client/Assets/Dreamtides/Masonry/Elements.cs`
- `CanvasButton`:
  `/Users/dthurn/Documents/GoogleDrive/dreamtides/client/Assets/Dreamtides/Buttons/CanvasButton.cs`
- `DisplayableButton`:
  `/Users/dthurn/Documents/GoogleDrive/dreamtides/client/Assets/Dreamtides/Buttons/DisplayableButton.cs`
- `ActionServiceImpl.IsProcessingCommands`:
  `/Users/dthurn/Documents/GoogleDrive/dreamtides/client/Assets/Dreamtides/Services/ActionServiceImpl.cs:78`
- `manifest.json` (UPM packages):
  `/Users/dthurn/Documents/GoogleDrive/dreamtides/client/Packages/manifest.json`
