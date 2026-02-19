# Research Results: Unity UI Navigation and Accessibility for ABU

## Question

How do Unity's existing systems (UI Toolkit VisualElements, UGUI Canvas, EventSystem, Input System,
and the Accessibility module) handle UI navigation/hierarchy and focus? What APIs can ABU leverage
to walk all three object systems (UI Toolkit, UGUI, 3D GameObjects) and produce a unified
accessibility tree? What do these systems offer for controller/gamepad navigation?

---

## Findings

### 1. UI Toolkit VisualElement Hierarchy

**Walking the tree.** Every `VisualElement` exposes `.Children()` (logical children, respecting
`contentContainer`) and `VisualElement.Hierarchy.Children` (physical children). UQuery
(`element.Query<T>()` / `.ForEach()`) lets you traverse the whole subtree without allocation.
The visual tree is drawn depth-first, so DFS on `.Children()` gives you document order.

**Getting the root.** `UIDocument.rootVisualElement` is the entry point per document. Multiple
`UIDocument` components can share a `PanelSettings` asset; to enumerate all panels in a scene, use
`Object.FindObjectsByType<UIDocument>()` and collect `uiDoc.rootVisualElement.panel` (deduplicating
by `PanelSettings` identity).

**IPanel.** Every `VisualElement` exposes `.panel` (type `IPanel`). `IPanel` provides:
- `visualTree` — root `VisualElement` of the panel
- `focusController` — `FocusController` for the panel
- `Pick(Vector2)` / `PickAll(Vector2)` — hit-test by panel-space coordinates

**Focus / navigability.** Each panel has a `FocusController` with a `focusedElement` property
(returns the currently focused `Focusable`). Navigability is controlled per element by:
- `focusable` (bool) — whether the element can receive focus at all
- `tabIndex` (int) — negative means excluded, 0 = default DFS order, positive = prioritized
- `canGrabFocus` — computed from visibility + enabled state

**Focus ring.** The panel's focus ring uses DFS on the visual tree by default. This gives a natural
traversal order that is exactly what ABU needs for an accessibility-tree snapshot.

**Detecting interactability.** `VisualElement.pickingMode` (enum `PickingMode`: `Position` or
`Ignore`) determines whether pointer events can target the element. Elements with `PickingMode.Ignore`
pass pointer events through. Combined with `focusable` and `resolvedStyle.visibility`, this lets
ABU classify elements as interactive vs. container/decorative.

**Screen coordinates.** `VisualElement.worldBound` gives the bounding rect in panel space.
`RuntimePanelUtils.ScreenToPanel(panel, screenPos)` converts screen pixels to panel coordinates.
`RuntimePanelUtils.CameraTransformWorldToPanel` converts world-space positions to panel coordinates.
The Dreamtides codebase (DocumentService.cs) already uses `RuntimePanelUtils.ScreenToPanel` for
safe-area calculation.

**Navigation events (controller).** UI Toolkit fires `NavigationMoveEvent` (D-pad / joystick /
arrow keys), `NavigationSubmitEvent` (Enter/A button), and `NavigationCancelEvent` (Escape/B button).
These trickle-down and bubble-up through the visual tree. There is no built-in "next focusable in
direction" resolution for UI Toolkit — the game or engine must handle focus movement manually, or
intercept navigation events and call `element.Focus()` explicitly.

**Dreamtides specifics.** Dreamtides uses a single `UIDocument` with `DocumentService._document`.
Its masonry renderer creates `NodeVisualElement` / `NodeLabel` / etc. instances (custom subclasses
of `VisualElement`). The `Callbacks` inner class wires `ClickEvent`, `MouseDownEvent`, etc. on
elements. To find "clickable" elements in the UI Toolkit tree, ABU should check for `NodeVisualElement`
instances that have a `Click` callback registered (`Callbacks.HasCallback(Event.Click)`), or more
generally check `pickingMode == Position` and a registered click handler.

---

### 2. UGUI Navigation System

**Selectable as the unit of navigation.** Every UGUI interactive widget (`Button`, `Toggle`,
`Dropdown`, `InputField`, `Slider`, `Scrollbar`) inherits from `UnityEngine.UI.Selectable`.
`Selectable` has:
- `Selectable.allSelectablesArray` — static array of all currently active `Selectable` objects in
  the scene (updated automatically as objects enable/disable)
- `Selectable.allSelectableCount` — count of active selectables
- `Selectable.AllSelectablesNoAlloc(array)` — non-allocating version

**Navigation struct.** Each `Selectable` has a `navigation` property of type `Navigation`, which
controls how controller/keyboard d-pad navigation flows. Key modes:
- `Navigation.Mode.None` — disabled (no d-pad navigation)
- `Navigation.Mode.Automatic` — Unity computes nearest selectable in each direction via spatial
  raycast at runtime
- `Navigation.Mode.Explicit` — designer sets `selectOnUp`, `selectOnDown`, `selectOnLeft`,
  `selectOnRight` explicitly

In `Automatic` mode, `navigation.selectOnUp` etc. return **null** (computed on the fly by
`FindSelectableOnUp()` etc.). In `Explicit` mode, those fields hold direct references.

**FindSelectableOnUp/Down/Left/Right.** These methods compute the nearest active, interactable
selectable in a given screen-space direction, using a spatial algorithm that considers the screen
positions of all `allSelectablesArray` entries. ABU can call these at snapshot time to discover the
navigation graph edges, even in Automatic mode.

**EventSystem currentSelectedGameObject.** The `EventSystem.current.currentSelectedGameObject`
holds the currently selected `GameObject` for UGUI navigation. ABU can read this to know which UGUI
element has focus.

**UGUI/UI Toolkit focus coordination.** When an `EventSystem` exists in the scene, UI Toolkit
automatically creates `PanelRaycaster` and `PanelEventHandler` components per panel. These bridge
UGUI events into UI Toolkit. UI Toolkit uses `EventSystem.currentSelectedGameObject` to coordinate
focus: gaining panel focus sets this field; losing it when a UGUI element is selected.

---

### 3. EventSystem

The `EventSystem` (from `UnityEngine.EventSystems`) is a singleton that routes all input to the
correct handler. It handles:
- Pointer events (mouse/touch) via raycasting
- Navigation events (d-pad/keyboard) via the current input module

`EventSystem.current.currentSelectedGameObject` is the authoritative "what is focused in UGUI"
answer. For UI Toolkit elements, the equivalent is `panel.focusController.focusedElement`.

When using UI Toolkit + UGUI together: UI Toolkit auto-detects an enabled `EventSystem` and wires
`PanelRaycaster` / `PanelEventHandler` intermediaries. These components handle event routing so
that both systems coexist.

---

### 4. New Input System

Dreamtides uses the new `UnityEngine.InputSystem` package (see `InputService.cs` using
`InputSystem.actions.FindAction`). The `InputSystemUIInputModule` replaces the old
`StandaloneInputModule` for new-input-system projects.

The new Input System does NOT add any new hierarchy or focus enumeration APIs over UGUI's
`Selectable.allSelectablesArray` or UI Toolkit's focus ring. It provides:
- A `move` action (Vector2) that drives `NavigationMoveEvent` in UI Toolkit and `IMoveHandler.OnMove`
  in UGUI
- `VirtualMouseInput` component — emulates a mouse cursor driven by stick/d-pad, which lets
  controller input work with mouse-style UI

**Implication for ABU.** For controller/gamepad AI navigation, ABU has two options: (a) synthesize
navigation events directly (fire `NavigationMoveEvent` / call `Selectable.Select()`) or (b) inject
a virtual cursor position and fire pointer events. Option (a) is simpler and more robust for
turn-based games.

---

### 5. Unity's Accessibility Module

Unity 2023.2+ includes a built-in `UnityEngine.Accessibility` module with:
- `AccessibilityHierarchy` — a tree of `AccessibilityNode` objects that screen readers consume
- `AccessibilityNode` — properties: `label`, `hint`, `role` (enum), `value`, `state`, `frame`,
  `frameGetter`, `children`, `parent`, `id`, `isActive`, `isFocused`; events: `invoked`,
  `focusChanged`, `scrolled`, etc.
- `AccessibilityRole` — enum with: None, Button, Image, StaticText, SearchField, KeyboardKey,
  Header, TabBar, Slider, Toggle, Container, TextField, Dropdown, TabButton, ScrollView
- `AssistiveSupport` — assigns the active hierarchy; notifies screen reader of changes
- `AccessibilitySettings` — device accessibility preferences (font scale, bold text, etc.)

**Critical constraint.** The accessibility hierarchy is **manual and separate** from the visual
hierarchy. Unity does not automatically populate it from UI Toolkit or UGUI. Game developers
build and maintain it themselves. It is designed for OS screen readers (TalkBack, VoiceOver,
Narrator), not for ABU.

**Platform support.** Android 8+, iOS, macOS, Windows (desktop support added in Unity 6.3 /
6000.3.0a5). The hierarchy is activated via `AssistiveSupport.activeHierarchy`.

**ABU relevance.** The `AccessibilityRole` vocabulary maps well to ARIA roles and could be
reused as the role system for ABU's own snapshot format. However, because the hierarchy is manual
and must be maintained separately, it's not useful as a "free" source of accessibility data — ABU
would need to build its own tree by walking Unity's visual hierarchies directly, not by reading
from `AccessibilityHierarchy`.

If ABU implements an `IAriaNode` interface as described in the prompt, game code could populate
both the ABU tree and (optionally) Unity's `AccessibilityHierarchy` from the same data.

---

### 6. Controller Navigation: Enumerating Navigable Elements

**UGUI.** `Selectable.allSelectablesArray` gives all currently active interactive UGUI elements.
To produce a spatial order, sort by screen-space Y then X (top-left to bottom-right). To get the
navigation graph, call `FindSelectableOnUp/Down/Left/Right()` on each selectable.

**UI Toolkit.** There is no `allFocusableElements` equivalent. ABU must walk the full visual tree
(DFS from `UIDocument.rootVisualElement`) and collect elements where `focusable == true` and
`canGrabFocus == true`. These are in focus-ring order by default (DFS with `tabIndex` override).

**3D GameObjects.** No built-in navigation between 3D objects. ABU must implement its own strategy,
likely: (a) require game objects to implement an `IAriaNode` interface to self-identify, or (b)
search for `Displayable` (the Dreamtides base class) components that have `CanHandleMouseEvents()`
returning true.

**Unified enumeration approach for ABU.**
1. Call `Object.FindObjectsByType<UIDocument>()` — walk each `rootVisualElement` tree by DFS,
   collecting `NodeVisualElement` elements with click callbacks
2. Call `Selectable.allSelectablesArray` — collect all active, interactable UGUI selectables
3. Call `Object.FindObjectsByType<T>()` where T is ABU's `IAriaNode` interface (or Dreamtides'
   `Displayable`) — collect 3D interactive objects
4. Project screen coordinates for each: `VisualElement.worldBound`, `Camera.WorldToScreenPoint`
   for 3D objects, `RectTransformUtility.WorldToScreenPoint` for UGUI

---

## Connections

**Dreamtides UI architecture (directly relevant):**
- UI Toolkit tree: `DocumentService._document.rootVisualElement` is the root. The Masonry renderer
  creates `NodeVisualElement` / `NodeLabel` subclasses; `Callbacks.HasCallback(Event.Click)` is
  the hook to detect interactive elements.
- UGUI buttons: `CanvasButton` (Dreamtides class at `Buttons/CanvasButton.cs`) is a `Displayable`
  (MonoBehaviour), NOT a `Selectable`. It has a Unity `Button` component wired to `OnClick()`.
  So `Selectable.allSelectablesArray` would catch these if they have a `UnityEngine.UI.Button` component.
- 3D interactive objects: `Displayable` abstract class (at `Layout/Displayable.cs`) is the base.
  `InputService` raycasts against the "Default" layer mask and calls
  `displayable.CanHandleMouseEvents()`. ABU should similarly walk `Displayable.GetComponentsInChildren`.
- Input: Dreamtides uses the new Input System with `InputSystem.actions`. An ABU fake input provider
  would replace `UnityInputProvider` by implementing `IInputProvider`.

**Other researchers should note:**
- The Masonry/Reconciler system (FlexNode → VisualElement) is the key for UI Toolkit. To understand
  what elements are interactive, look at `MasonRenderer.cs` and `Elements.cs` (the `Callbacks` class).
- `DocumentService.ScreenPositionToElementPosition` and `RuntimePanelUtils.ScreenToPanel` are the
  coordinate-system bridges already in use.
- For testing ABU input injection: `InputService.InputProvider` is an `IInputProvider` interface —
  a test fake could be injected here directly, which is already the pattern for Dreamtides tests.

---

## Open Questions

1. Does Dreamtides' `CanvasButton` (a `Displayable`, not a `Selectable`) get a `UnityEngine.UI.Button`
   component? If so, `Selectable.allSelectablesArray` catches it; if not, ABU must handle it separately
   via the 3D/MonoBehaviour path.

2. What is the sorting order (render priority) between Dreamtides' UI Toolkit panels and any UGUI
   canvases? This determines which system "wins" hit-tests when they overlap, which matters for the
   snapshot's z-ordering.

3. For UI Toolkit navigation events (controller d-pad), does UI Toolkit automatically move focus
   between elements on `NavigationMoveEvent`, or does the game have to handle this manually? The
   documentation suggests manual handling is needed for UI Toolkit, unlike UGUI.

4. Does the Dreamtides `IInputProvider` interface get used in any existing tests as a fake? If so,
   ABU can follow the same pattern for injecting synthetic input during AI-driven play.

5. Are there any `Selectable` components in the Dreamtides scene at all, or is everything either
   UI Toolkit (Masonry) or `Displayable`-based 3D objects? If UGUI selectables are absent,
   ABU can skip the `allSelectablesArray` path for this testbed.
