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
- [Snapshot Format](#snapshot-format)
- [Test Save Generation](#test-save-generation)
- [Testing and Validation](#testing-and-validation)
- [Common Pitfalls](#common-pitfalls)

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
python3 scripts/abu/abu.py set-device iphone-se  # set device (switches to Device Simulator for mobile)

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

Both the Python CLI and C# bridge auto-detect the worktree context. The Python
side checks `ABU_PORT` first, then detects if the repo root is under
`~/dreamtides-worktrees/` and looks up the port. The C# side does the same
using `Application.dataPath`. No manual port configuration is needed.

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

### Visual Identification

Worktree Unity editors display a colored strip at the top of the window and a
`[NAME]` prefix in the title bar so they are visually distinguishable from the
main editor. Each worktree gets a stable color from a 6-color palette based on
a hash of the name.

## Adding DreamtidesAbuSetup to a Scene

1. Create a `GameObject` in the scene (or use the Registry prefab).
2. Add the `DreamtidesAbuSetup` component.
3. Optionally assign the `Registry` reference in the Inspector (auto-discovered
   via `FindFirstObjectByType` if left null).
4. Enter Play mode. Unity logs `[Abu] TCP server listening on port 9999` and
   then `[Abu] Client connected.` each time the CLI connects.

The component creates an `AbuBridge` GameObject with `DontDestroyOnLoad`, so it
persists across scene loads.

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

**Quest mode example:**

```
- application "Dreamtides"
  - region "Quest"
    - group "Controls"
      - button "Menu" [ref=e1]
      - button "Undo" [ref=e2]
    - group "Map"
      - button "Battle" [ref=e3]
      - button "Draft" [ref=e4]
      - button "Shop" [ref=e5]
    - label "Essence: 75"
    - group "Quest Deck (43 cards)"
      - button "Browse Quest Deck" [ref=e6]
    - group "Identity"
      - label "Flame Weaver (3/2)"
```

Refs (`e1`, `e2`, ...) are monotonically assigned in DFS pre-order per snapshot
and invalidated by the next snapshot or any action command.

**Compact mode** (`--compact`): omits nodes that are non-interactive, have no
label, and have no interactive descendants. Labeled and interactive nodes are
always included.

**Action responses:** Action commands (click/hover/drag) return a post-action
snapshot after the UI settles — refs in this snapshot are fresh and supersede
any previous snapshot. History entries (game events that occurred during the
action) appear before the snapshot when present, separated by
`--- History ---` / `---` delimiters.

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

Integration between Python and C# is not covered by automated tests. Validate
manually by entering Unity Play mode with `DreamtidesAbuSetup` attached and
running `python3 scripts/abu/abu.py snapshot`.

## Common Pitfalls

- **Stale refs**: refs are invalidated after every snapshot or action command.
  Agents must re-parse `refs` from each response before making the next call.
- **Compact mode omission**: a node is omitted in compact mode only when it is
  non-interactive, has no non-empty label, AND has no interactive descendants.
  All three conditions must hold.
- **TCP single-client model**: a new CLI connection replaces the previous one.
  Do not assume the connection remains open across commands.

## Related Documents

- [abu_internals.md](abu_internals.md): Architecture, C# class internals, wire
  protocol, coding conventions, and how to adapt ABU to another game. Read when
  working on ABU implementation or debugging internal behavior.
- [abu_development.md](abu_development.md): Step-by-step guide to modifying the
  scene walker, adding new UI features, and interactively testing changes using
  the Unity/CLI workflow.
