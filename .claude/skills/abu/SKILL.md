---
name: abu
description: ABU (Agent-Browser for Unity) — Python CLI + C# Unity TCP server for AI agent interaction with Dreamtides
---

# ABU Skill

## Architecture Overview

ABU lets AI agents interact with the running Unity game programmatically. The
Python CLI (`scripts/abu/abu.py`) connects to Unity over TCP, sends one NDJSON
command per process invocation, reads one response, and exits. Unity acts as the
server: `TcpServer.cs` listens on port 9999, hands commands to `AbuBridge.cs`
on the main thread via a `ConcurrentQueue`, and `SnapshotCommandHandler.cs`
dispatches the seven supported commands.

Snapshot responses carry pre-formatted ARIA-style text (produced by
`SnapshotFormatter.cs` in C#) rather than raw scene node trees. This means the
Python CLI does no formatting — it prints the `data` field of Unity's response
directly to stdout.

All C# source compiles as part of the `Dreamtides` assembly
(`client/Assets/Dreamtides/Dreamtides.asmdef`). There is no separate Abu
assembly definition. The core files use `namespace Abu`; the three
Dreamtides-specific integration files use `namespace Dreamtides.Abu` with
`using Abu;`.

## Component Map

### Python CLI

| File | Purpose |
|------|---------|
| `scripts/abu/abu.py` | Single-file CLI, stdlib only. All 7 subcommands. |
| `scripts/abu/test_abu.py` | 36 unit tests covering params, commands, responses, TCP roundtrip. |

Key functions in `abu.py`:
- `build_params(args)` — converts argparse namespace to wire params dict (line 47)
- `build_command(command, params)` — wraps params in `{id, command, params}` NDJSON (line 77)
- `send_command(command, params, port)` — one-shot TCP connect/send/recv (line 111)
- `handle_response(command, response)` — extracts output; decodes base64 for screenshot (line 87)
- `main()` — entry point; reads `ABU_PORT` env var (line 200)

### C# Transport

| File | Lines | Purpose |
|------|-------|---------|
| `client/Assets/Dreamtides/Abu/TcpServer.cs` | 248 | TCP listener, accept loop, read loop, NDJSON framing |
| `client/Assets/Dreamtides/Abu/AbuBridge.cs` | 113 | MonoBehaviour; creates TcpServer, dispatches commands on main thread |

`TcpServer` runs two background threads: an accept thread (`AcceptLoop`, line
106) and a per-connection read thread (`ReadLoop`, line 154). A new connection
replaces the previous client. `Send()` (line 63) uses a `_clientLock` to write
safely from the main thread while the accept thread may be replacing the client.

`AbuBridge.Awake()` reads `ABU_PORT` (then `ABU_WS_PORT` as fallback) to
configure the port (lines 71-84). `Update()` drains `ReceiveQueue` each frame
(line 94). `OnDestroy()` calls `TcpServer.Shutdown()` (line 109).

### C# Command Handling

| File | Lines | Purpose |
|------|-------|---------|
| `client/Assets/Dreamtides/Abu/SnapshotCommandHandler.cs` | 328 | Dispatches all 7 commands |
| `client/Assets/Dreamtides/Abu/SnapshotFormatter.cs` | 107 | DFS → ARIA text + refs dict |
| `client/Assets/Dreamtides/Abu/CommandSchema.cs` | 143 | Wire protocol types |
| `client/Assets/Dreamtides/Abu/RefRegistry.cs` | 68 | Maps e1/e2/... → action callbacks |

**`SnapshotCommandHandler`** key paths:
- `HandleSnapshot()` (line 80): clears registry, parses compact flag, calls
  `BuildSnapshotData(compact)`, responds immediately (synchronous).
- `BuildSnapshotData(compact)` (line 97): calls each `ISceneWalker.Walk()`,
  wraps results in an `application` root node, calls `SnapshotFormatter.Format()`.
- `DispatchRefAction()` (lines 169, 198): looks up ref in registry, invokes
  callback, starts `WaitForSettledThenRespond` coroutine.
- `WaitForSettledThenRespond()` (line 266): polls `IsSettled()` each frame;
  when settled, rebuilds snapshot (non-compact) and responds with `ActionSnapshotData`.
- `CaptureScreenshot()` (line 295): waits for end-of-frame, captures PNG, base64-encodes.

**`SnapshotFormatter.Format(nodes, compact)`** (line 16): DFS pre-order walk.
Per node: 2-space indent × depth, `"- {role}"`, optional `" \"{label}\""` if
non-null/non-empty, optional `" [ref=eN]"` if interactive (counter starts at 0,
pre-incremented). Compact mode omits nodes where all of: non-interactive, empty
label, no interactive descendants. Returns `SnapshotData{Snapshot, Refs}`.

**`RefRegistry`**: `Register(callbacks)` assigns the next ref string and stores
callbacks (line 43). `Clear()` resets counter to 1 (line 62). Called in the
same DFS order as `SnapshotFormatter.Walk()`, so ref strings align.

### C# Schema Types (`CommandSchema.cs`)

| Type | Wire shape | Notes |
|------|-----------|-------|
| `AbuCommand` | `{id, command, params}` | `params` is `JObject?` |
| `AbuResponse` | `{id, success, data?, error?}` | |
| `AbuSceneNode` | `{role, label?, interactive, children}` | Internal only; not in responses |
| `SnapshotData` | `{snapshot, refs}` | `refs` is `Dictionary<string, SnapshotRef>` |
| `ActionSnapshotData : SnapshotData` | `{clicked/hovered/dragged, snapshot, refs}` | Action fields merged via `JsonExtensionData` |
| `SnapshotRef` | `{role, name}` | Entry in refs dict |
| `SnapshotParams` | `{interactive?, compact?, maxDepth?}` | `maxDepth` parsed but not implemented |
| `RefParams` | `{ref}` | For click/hover |
| `DragParams` | `{source, target?}` | For drag |

### C# Interfaces

| Interface | File | Contract |
|-----------|------|---------|
| `ISceneWalker` | `ISceneWalker.cs` | `Walk(RefRegistry) → AbuSceneNode` |
| `ISettledProvider` | `ISettledProvider.cs` | `IsSettled() → bool`, `NotifyActionDispatched()` |
| `ICommandHandler` | `ICommandHandler.cs` | `HandleCommand(command, bridge, onComplete)` |

### Dreamtides Integration

| File | Namespace | Purpose |
|------|-----------|---------|
| `DreamtidesAbuSetup.cs` | `Dreamtides.Abu` | MonoBehaviour that wires walker and settled provider |
| `DreamtidesSceneWalker.cs` | `Dreamtides.Abu` | 1034-line walker for battle + non-battle scenes |
| `DreamtidesSettledProvider.cs` | `Dreamtides.Abu` | Checks `ActionService.IsProcessingCommands` + DOTween + 3-frame hold |

`DreamtidesSceneWalker` traverses: CanvasButtons (Menu/Undo/Dev/Bug),
user/opponent groups (status/deck/identity/void/battlefield/hand/dreamwell),
stack, game modifiers, action buttons, essence label, thinking indicator, and
UIToolkit overlays. In non-battle mode: UIToolkit + 3D scene.

`DreamtidesSettledProvider.IsSettled()` returns true when:
`!IsProcessingCommands` OR `WaitingForFinalResponse` (resets frame count) OR
timeout (3 seconds) OR (`!IsProcessingCommands && DOTween.TotalPlayingTweens() <= 0`
held for 3 consecutive frames).

### C# Tests

| File | Count | Coverage |
|------|-------|---------|
| `Tests/Abu/SnapshotFormatterTests.cs` | 14 | All formatter modes; port of original TypeScript test suite |
| `Tests/Abu/CommandSchemaTests.cs` | varies | `SnapshotData`/`ActionSnapshotData` serialization |
| `Tests/Abu/WebSocketMessageTests.cs` | varies | `AbuResponse` serialization |
| `Tests/Abu/RefRegistryTests.cs` | varies | Ref assignment and lookup |
| `Tests/Abu/BusyTokenTests.cs` | varies | Ref counting, dispose semantics |
| `Tests/Abu/SceneWalkerTests.cs` | varies | `DreamtidesSceneWalker` integration |
| `Tests/Abu/SettledProviderTests.cs` | varies | `DreamtidesSettledProvider` settle conditions |
| `Tests/Abu/InputSimulationTests.cs` | varies | Click/hover input simulation |

## Data Flow

```
abu.py snapshot --compact
  │
  ├─ build_params() → {"compact": true}
  ├─ send_command("snapshot", {"compact": true}, 9999)
  │     connect TCP → send {"id":"<uuid>","command":"snapshot","params":{"compact":true}}\n
  │
  │                         Unity main thread (Update())
  │                         TcpServer.ReceiveQueue.TryDequeue()
  │                         → AbuBridge → SnapshotCommandHandler.HandleSnapshot()
  │                           refRegistry.Clear()
  │                           parse SnapshotParams → compact=true
  │                           BuildSnapshotData(compact=true)
  │                             DreamtidesSceneWalker.Walk(refRegistry)
  │                               → registers interactive nodes → returns AbuSceneNode tree
  │                             wrap in application root node
  │                             SnapshotFormatter.Format([rootNode], compact=true)
  │                               DFS pre-order → lines + refs dict
  │                             → SnapshotData{Snapshot, Refs}
  │                           TcpServer.Send(AbuResponse{success:true, data:SnapshotData})
  │                             StreamWriter.WriteLine(json) + Flush()
  │
  ├─ recv one line → JSON parse → response dict
  ├─ handle_response("snapshot", response)
  │     return json.dumps(data)
  └─ print to stdout
```

For action commands (click/hover/drag), the flow diverges after callback
invocation: `WaitForSettledThenRespond` coroutine polls `IsSettled()` each
frame, then rebuilds a non-compact snapshot and responds.

## Testing Strategy

**C# tests** run via `just unity-tests`. The 14 `SnapshotFormatterTests` are
the behavioral specification for the formatter — they were ported directly from
the TypeScript test suite that defined the original format.

**Python tests** in `scripts/abu/test_abu.py` use `unittest` with a mock TCP
server (`TestSendCommand._start_mock_server`) to test full roundtrips without
Unity. Run with `python3 scripts/abu/test_abu.py` or `python3 -m pytest
scripts/abu/test_abu.py`.

The integration between Python and C# is not covered by automated tests.
Validate manually by entering Unity Play mode with `DreamtidesAbuSetup` attached
and running `python3 scripts/abu/abu.py snapshot`.

## Cross-References

- `docs/abu/abu.md` — usage guide, wire protocol reference, snapshot format
- `.claude/rules/abu.md` — namespace conventions, pitfalls, validation commands
- `docs/display_animation/display_animation.md` — how DOTween animations gate
  `DreamtidesSettledProvider.IsSettled()`
- `docs/masonry_ui_panels/masonry_ui_panels.md` — UIToolkit structure that
  `DreamtidesSceneWalker` traverses
