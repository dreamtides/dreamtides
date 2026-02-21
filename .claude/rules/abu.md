---
description: Conventions and validation for the ABU system (Python CLI + C# Unity TCP server)
globs:
  - scripts/abu/**/*.py
  - client/Assets/Dreamtides/Abu/**/*.cs
  - client/Assets/Dreamtides/Tests/Abu/**/*.cs
---

# ABU Rules

ABU is a Python CLI (`scripts/abu/abu.py`) that sends NDJSON commands over TCP
to a Unity TCP server. All C# lives in `client/Assets/Dreamtides/Abu/` as part
of the `Dreamtides` assembly. The Python CLI uses only stdlib; no pip deps.

## Key Conventions

- **Two namespaces coexist**: migrated core files use `namespace Abu`; Dreamtides
  integration files (`DreamtidesAbuSetup`, `DreamtidesSceneWalker`,
  `DreamtidesSettledProvider`) use `namespace Dreamtides.Abu` with `using Abu;`.
  Do not change either namespace — walkers must not be modified.
- **No separate asmdef**: All Abu C# files compile under `Dreamtides.asmdef`.
  Never add a nested `.asmdef` inside `client/Assets/Dreamtides/Abu/`.
- **Wire protocol is NDJSON**: one JSON object per line; `\n` terminates each
  message. The Python CLI sends exactly one command and reads exactly one
  response per process invocation.
- **Ref assignment is DFS pre-order**: `RefRegistry` assigns refs during
  `ISceneWalker.Walk()` and `SnapshotFormatter` assigns display refs during
  `Format()`. Both use the same DFS order on the same tree, so ref strings
  align. Never change walk order without updating both.
- **Snapshot shape**: `SnapshotData` carries `Snapshot` (string) and `Refs`
  (dict). `ActionSnapshotData` inherits `SnapshotData` and merges action fields
  via `JsonExtensionData`. Action snapshots are never compact.
- **Port configuration**: default port 9999; read from `ABU_PORT` env var in
  both `AbuBridge.cs` (line 71-74) and `abu.py` (line 205). Legacy `ABU_WS_PORT`
  is also accepted in `AbuBridge.cs` as fallback.
- **Python style**: shebang `#!/usr/bin/env python3`, module docstring, stdlib
  only, all type hints, `main() -> None`, `if __name__ == "__main__": main()`.
- **Error handling**: `AbuError` hierarchy in `abu.py` (`ConnectionError`,
  `TimeoutError`, `EmptyResponseError`). Print to stderr, exit code 1 on error.
- **Do not modify**: `DreamtidesSceneWalker.cs` and `DreamtidesSettledProvider.cs`
  are large and working; they have no tests of their own in this layer.

## Common Pitfalls

- **Stale refs**: refs are invalidated after every snapshot or action command.
  Agents must re-parse `refs` from each response before making the next call.
- **Compact mode omission**: a node is omitted in compact mode only when it is
  non-interactive, has no non-empty label, AND has no interactive descendants.
  All three conditions must hold.
- **`BusyToken` scope**: acquire `BusyToken` to suppress `DefaultSettledProvider`
  during multi-step coroutines; dispose it when the coroutine completes.
- **TCP single-client model**: a new CLI connection replaces the previous one.
  Do not assume the connection remains open across commands.

## Validating Changes

```sh
# Python tests
python3 scripts/abu/test_abu.py

# C# tests (includes SnapshotFormatterTests, CommandSchemaTests, RefRegistryTests)
just unity-tests

# Format
just fmt-csharp
```
