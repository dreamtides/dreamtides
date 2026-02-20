# ABU: Agent-Browser for Unity

## Goal

Build a general-purpose system that lets AI agents interact with Unity games
using the [agent-browser](https://github.com/vercel-labs/agent-browser) CLI as
the control interface. The system should work with any Unity game, not just a
specific one. The Dreamtides project serves as the testbed to prove it works in
practice: the end state is an AI agent that can launch Dreamtides, navigate its
menus, play through battles, and make strategic decisions -- all driven through
`agent-browser` CLI commands.

Ideally, only well-defined minimal changes are required for the target game. I
think this would ideally take the form of something like "just implement
`IAriaNode` on all your components and drop in our coordinator class somewhere
as a top-level GameObject and the system will just work", but obviously this is
just speculative.

I'd like code for this to live in a separate open source git repository under
~/abu. I would like to consume this as a Unity Package Manager (UPM) local
package. ~/abu should contain a Unity package layout and then we'd update
manifest.json with `"com.abu.bridge": "file:/Users/dthurn/abu"`.

We generally assume ABU is for turn-based games, or at least those without a
major reflex element.

## Why agent-browser?

Agent-browser provides a ready-made AI-to-application protocol: the AI issues
shell commands (`agent-browser snapshot`, `agent-browser click @e3`), gets back
structured text representations of the UI, and acts on them. AI agents (Claude,
GPT, etc.) already know how to use this workflow. Rather than inventing a new
protocol or MCP server, we reuse the existing CLI and daemon architecture.

## Architecture (high level)

The agent-browser CLI locates a `daemon.js` via the `AGENT_BROWSER_HOME`
environment variable. We write a **standalone TypeScript daemon** (not a fork)
that the CLI launches in place of the real browser daemon. This daemon:

1. Listens on the same Unix domain socket the CLI expects
2. Speaks the same newline-delimited JSON protocol
3. Runs a WebSocket server that Unity connects to as a client
4. Translates agent-browser commands (snapshot, click, fill, etc.) into Unity UI
   operations, and translates Unity UI state back into agent-browser snapshot
   format

On the Unity side, a C# MonoBehaviour connects to the daemon as a **WebSocket
client** using .NET's built-in `System.Net.WebSockets.ClientWebSocket` (no
third-party library required). This bridge:

1. Walks the full Unity scene to produce snapshots -- this includes UI Toolkit
   `VisualElement` trees, UGUI Canvas elements, and 3D GameObjects in the scene,
   looking for implementations of our specific interface.
2. Assigns refs (`@e1`, `@e2`, ...) to interactive elements across all systems
3. Resolves refs and executes clicks, text input, etc.
4. Returns UI state as JSON over the WebSocket connection

The server role is on the TypeScript side, where running a WebSocket server is
trivial (Node.js `ws` package). Unity only needs to be a client, which is
well-supported by the standard library.

```
AI Agent (Claude Code, etc.)
  |  shell commands
  v
agent-browser CLI (unmodified, production binary)
  |  Unix socket, JSON protocol
  v
ABU Daemon (our TypeScript project, WebSocket server)
  ^  WebSocket, JSON
  |
Unity / Dreamtides (C# MonoBehaviour, WebSocket client)
```

## Key design questions to investigate

- What is the minimal subset of agent-browser commands we need to implement? The
  full protocol has ~100 commands; a game likely only needs snapshot, click,
  type/fill, screenshot, close, and maybe a few others.

- How should the snapshot format work? Agent-browser snapshots are ARIA
  accessibility trees designed for web pages. The Unity equivalent needs to
  represent all three object systems (UI Toolkit VisualElements, UGUI Canvas
  elements, 3D GameObjects) in a unified tree format that stays close to the
  ARIA style so AI agents can reuse their web browsing intuitions. This should
  be fully general -- no game-specific semantics baked in.

- How do we walk the full Unity scene? The snapshot needs to cover UI Toolkit
  trees (via `VisualElement` hierarchy), UGUI Canvas elements (via
  `GetComponentsInChildren<Selectable>`), and 3D scene objects (via the
  `Transform` hierarchy). How do we unify these into one coherent tree from the
  perspective of the main camera?

- What's the right way to simulate input across all three systems? UI Toolkit,
  UGUI, and 3D objects each have different interaction models. Direct callback
  invocation is simplest, but synthesized events may be more robust.

- How does the AI know when the UI has settled? After an action, the game may
  play animations or transition between states. The daemon needs a default
  heuristic for detecting when the scene has stabilized (e.g. no tween updates
  for N frames). For cases where the heuristic isn't enough, there should be a
  small, well-specified API that game authors can optionally implement (e.g. an
  interface) -- but the goal is to minimize the amount of game-specific code
  required. Many games have objects that are ALWAYS moving.

- Can we reuse AltTester patterns? AltTester solves similar problems (finding
  Unity objects, simulating input, communicating over WebSocket). We should
  study its approach but build something purpose-fit for AI agents rather than
  test automation.

## Scope of the design document

The design document should cover:

1. The TypeScript daemon: project structure, socket/protocol handling, command
   routing, WebSocket server for Unity connections
2. The Unity C# bridge: WebSocket client connection, scene walking (UI Toolkit
   - UGUI + 3D GameObjects), ref assignment, input simulation, threading model
3. The snapshot format: how Unity's three object systems map into a unified
   accessibility-style tree
4. Which agent-browser commands to implement and how each maps to Unity
5. A plan for incremental development: what is the smallest thing we can demo
   end-to-end, and how do we build up from there?
6. How to test: can we write automated tests that verify the daemon speaks the
   protocol correctly, independent of Unity?

## Non-goals for the design document

- Modifying agent-browser itself (we use it as-is)
- Game-specific logic in the bridge (it should be general-purpose for any Unity
  game; Dreamtides is just the testbed)
- Exposing internal game state (the AI should learn from the UI, not a
  privileged data channel)
- Training or fine-tuning AI models (we rely on existing model capabilities)
- Supporting multiplayer or networked game scenarios

## Research: Unity UI hierarchy and controller support

Unity has no single system that produces a unified accessibility tree across its
three object systems. UI Toolkit has a `VisualElement` tree with focus rings but
no ARIA roles; UGUI has `Selectable.allSelectablesArray` with spatial navigation
graphs but no hierarchy; Unity 6.3 added
`AccessibilityHierarchy`/`AccessibilityNode` with proper roles (Button, Toggle,
Slider, etc.) but the hierarchy is entirely manual — Unity does not
auto-populate it from UI Toolkit or UGUI. The `IAriaNode` interface approach is
the right design: game components self-describe their role and label, ABU walks
the scene across all three systems, projects to screen coordinates, and
assembles the unified tree itself.
