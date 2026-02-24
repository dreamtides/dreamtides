# Technical Design: Abu Quest Mode Snapshot Support

## Goal

Enable the abu snapshot system to navigate and capture quest mode UI in the
Dreamtides Unity client. Three independent gaps prevent this today: site
navigation buttons are invisible to the scene walker, the close-site button is
invisible, and Cinemachine camera transitions are not detected by settled
detection. Fixing these makes abu able to click site buttons on the quest map,
wait for the camera to finish moving, observe the resulting site UI, and close
it -- all using general infrastructure, not site-specific logic.

## Background

### Abu Snapshot Pipeline

Abu is an Agent-Browser for Unity. It walks the scene to produce a structured
accessibility tree (`AbuSceneNode`), assigns monotonically incrementing refs
(`e1`, `e2`, ...) to interactive nodes, and serializes the tree as a text
snapshot. A Python CLI can send click commands referencing a ref; the handler
looks up the ref in `RefRegistry`, invokes its callback, calls
`NotifyActionDispatched()`, polls `IsSettled()` until true, then builds a new
snapshot.

The scene walker is a partial class split across four files:

- `client/Assets/Dreamtides/Abu/DreamtidesSceneWalker.cs` -- `Walk()` dispatch,
  helpers
- `client/Assets/Dreamtides/Abu/SceneWalker/DreamtidesSceneWalker.Ui.cs` --
  quest mode walk path (`WalkFallbackScene3D`), canvas button walking
- `client/Assets/Dreamtides/Abu/SceneWalker/DreamtidesSceneWalker.Battle.cs` --
  battle mode walk, `TryAddCloseBrowserButton`
- `client/Assets/Dreamtides/Abu/SceneWalker/DreamtidesSceneWalker.Input.cs` --
  callback builders

The top-level `Walk()` method (line 32) checks
`_registry.BattleLayout.Contents.activeSelf`. When false (quest mode), it calls
`WalkUiToolkit()` + `WalkFallbackScene3D()`.

### Quest Map and Site Buttons

Site CanvasButtons are black circle icons on the quest map. They are dynamically
created by `DreamscapeService.CreateOpenSiteButton()` and stored in
`DreamscapeMapCamera._siteButtonsBySite` (a
`Dictionary<AbstractDreamscapeSite, CanvasButton>`). They are shown/hidden via
`DreamscapeMapCamera.ShowSiteButtons()` / `HideSiteButtons()`, which toggle
`gameObject.SetActive()` on each button.

Each button gets a `ButtonView` with `Label = site.ButtonLabel` and an action
built from `site.DebugClickAction`. Sites with empty `DebugClickAction` get a
`NoOp` action.

### Close Site Button

`DreamscapeService._closeSiteButton` is a `CanvasGroup` (not a `CanvasButton`)
with a child `CloseBrowserButton` component. It becomes active when
`CloseBrowserButton.CloseAction` is set to a non-null value. Various site flows
set this (e.g., `PrototypeQuestTemptingOfferFlow.ShowTemptingOfferCards()` line
129, `PrototypeQuestShopFlow`). In battle mode, a different `CloseBrowserButton`
instance lives on `BattleLayout` and is walked by `TryAddCloseBrowserButton`.

There is also a `CloseBrowserButton` instance on
`QuestDeckBrowserObjectLayout._closeButton` (line 32), used when browsing the
quest deck. This is a second close button that needs walking.

### Camera Transitions and Settled Detection

Clicking a site button fires `ActionServiceImpl.PerformAction()`, which detects
`ApplyTestScenarioAction` and calls `PrototypeQuest.OnDebugScenarioAction()`
directly, returning immediately without setting `IsProcessingCommands = true`
(line 172-178 of `ActionServiceImpl.cs`). The camera transition runs as a
fire-and-forget coroutine via `FocusSiteFlow`.

Camera transitions use Cinemachine blending (2-second duration) which produces
zero DOTween tweens. `DreamtidesSettledProvider.IsLocalSettled()` only checks
`IsProcessingCommands` and `DOTween.TotalPlayingTweens()`, so it reports settled
immediately before the camera finishes moving.

`DreamscapeMapCamera.IsTransitioning` (line 79: `_transitionRoutine != null`)
correctly tracks whether a camera blend is in progress.

### BusyToken

`BusyToken` is a ref-counted disposable. `IsAnyActive` is a static property
returning true when any token is undisposed. `DefaultSettledProvider` already
checks `BusyToken.IsAnyActive` (line 48); `DreamtidesSettledProvider` does not.

## Design

### Site Button Discovery

`WalkFallbackScene3D` in `DreamtidesSceneWalker.Ui.cs` currently walks only four
hardcoded `DocumentService` buttons via `WalkCanvasButtons`. It needs to also
discover and walk the dynamically-created site CanvasButtons.

`DreamscapeMapCamera` is not on `Registry`, but the scene walker does not need
the camera itself -- it needs the collection of site buttons. The site buttons
are stored in `DreamscapeMapCamera._siteButtonsBySite` (a
`Dictionary<AbstractDreamscapeSite, CanvasButton>`). The `DreamscapeMapCamera`
instance can be found via `FindObjectsByType<DreamscapeMapCamera>`, which is the
established pattern in this codebase for component discovery (see
`DreamscapeService.FindCharacterSite()`, `FindDraftSite()`, `FindBattleSite()`
at lines 175-223). The `_siteButtonsBySite` dictionary is an `internal` field,
accessible from `DreamtidesSceneWalker` since both are in the `Dreamtides`
assembly. The walker should iterate the dictionary values and pass each
`CanvasButton` to `TryAddCanvasButton`.

The existing `TryAddCanvasButton` method handles visibility filtering correctly
(checks `activeSelf`, `alpha > 0`, non-empty label) and requires no changes. The
walker just needs to enumerate the site buttons and pass each to
`TryAddCanvasButton`.

Site buttons with `NoOp` actions should still be walked if they have a non-empty
label and are active/visible. `TryAddCanvasButton` already registers
`BuildCanvasButtonCallbacks` which calls `button.OnClick()`, so clicking them
will dispatch the NoOp action through normal channels. Filtering out NoOp
buttons would require the walker to inspect the button's action, coupling it to
game logic. Let the existing label/visibility filtering suffice.

### Close Site Button Discovery

Two `CloseBrowserButton` instances need walking in quest mode:

1. **`DreamscapeService.CloseSiteButton`** -- the primary close button for site
   views (shop, tempting offering, etc.).
   `_registry.DreamscapeService.CloseSiteButton` returns a `CanvasGroup`, not a
   `CloseBrowserButton`. The `CloseBrowserButton` component lives on the same
   GameObject and must be retrieved via `GetComponent<CloseBrowserButton>()` on
   the `CanvasGroup`. This is the established pattern used in quest flows (e.g.,
   `PrototypeQuestShopFlow` line 198, `PrototypeQuestTemptingOfferFlow` line
   129).

2. **`QuestDeckBrowserObjectLayout._closeButton`** -- the close button for the
   quest deck browser. Accessible via
   `_registry.DreamscapeLayout.QuestDeckBrowser._closeButton`.

The battle-mode `TryAddCloseBrowserButton` in `DreamtidesSceneWalker.Battle.cs`
(line 189) shows the pattern: check `gameObject.activeSelf`, add with "Close
Browser" label and `OnClick = () => button.OnClick()`.

The quest mode walk path (`WalkFallbackScene3D`) should apply this same pattern
for both quest-mode close buttons. The check should be generic: find the
`CloseBrowserButton` component, check if the GameObject is active, and add it.
This belongs in `DreamtidesSceneWalker.Ui.cs` as part of the
`WalkFallbackScene3D` method or a helper called from it.

Note: `CloseBrowserButton` is a plain `MonoBehaviour`, not a `Displayable` or
`CanvasButton`, so it cannot be found by `WalkDisplayables` or
`TryAddCanvasButton`. It needs its own check, identical in spirit to the battle
mode version.

### Settled Detection for Camera Transitions

`DreamtidesSettledProvider.IsLocalSettled()` must be extended to treat
`BusyToken.IsAnyActive` as a "not settled" signal, following the same pattern
used by `DefaultSettledProvider` (line 48): when `BusyToken.IsAnyActive` is
true, reset the frame counter and return false.

Camera transitions in `DreamscapeMapCamera` must acquire a `BusyToken` for their
duration. The token should be acquired inside the camera's transition coroutines
(`TransitionToSite` and `TransitionToMap`) and disposed when the coroutine
completes (including on early exit or error). This is the camera-level approach
rather than caller-level, ensuring all callers of `FocusSite()` and
`ActivateWithTransition()` are covered without modification.

The `BusyToken` is the right mechanism because:

- It is the established pattern (used by `DefaultSettledProvider`).
- It is ref-counted and disposable, fitting naturally with coroutine lifecycles.
- It decouples the camera from abu internals -- the camera only creates/disposes
  a token; it has no reference to `DreamtidesSettledProvider`.

### Tempting Offer Animation Race

`HandleTemptingOfferSelection` (line 51 of `PrototypeQuestTemptingOfferFlow.cs`)
calls `_registry.StartCoroutine(ResolveTemptingOfferSelection(offerNumber))`,
which runs outside the `FocusSiteFlow` yield chain. The coroutine's animations
all use DOTween (dissolves, card moves, projectiles), which the settled provider
already detects. The gap between coroutine start and the first DOTween tween
creation is covered by the 3-frame settle requirement. This is not a problem in
practice and does not need a BusyToken.

### Walk Order Consistency

Adding new interactive nodes to the quest walk path changes ref numbering for
subsequent nodes. This is by design -- `RefRegistry` and `SnapshotFormatter`
both use depth-first traversal and stay in sync as long as both see the same
tree. No special synchronization is needed. The coder should verify that the
snapshot formatter walks the same nodes by taking actual snapshots after
changes.

## Constraints

- **No site-specific logic in the walker.** The walker must discover and walk
  buttons generically. It must not contain references to specific sites (shop,
  draft, tempting offer) or specific action strings.

- **Real-world QA validation.** Every change must be validated by running the
  Unity editor in quest mode, using `abu snapshot` and `abu screenshot` to
  verify that: (a) site buttons appear in the snapshot with correct labels, (b)
  clicking a site button via `abu click` waits for the camera transition to
  complete before returning, (c) the close button appears in the snapshot when
  at a site, (d) clicking close returns to the map and the snapshot shows site
  buttons again. The quest prototype is stateless -- play mode must be restarted
  between visiting different sites.

- **Existing battle mode must not regress.** Battle mode snapshot behavior must
  remain unchanged. The scene walker changes are gated by the existing
  battle/non-battle branch in `Walk()`.

- **BusyToken lifecycle.** Tokens must be disposed on all exit paths (normal
  completion, early return, exception). Use `try/finally` or equivalent
  patterns.

## Non-Goals

- Automated integration tests for quest-mode snapshots. The quest prototype is
  not testable in the existing test harness (it requires a fully initialized
  Unity scene with Cinemachine cameras, site GameObjects, etc.). QA validation
  is manual via `abu snapshot` / `abu screenshot`.
- Walking any UI elements beyond site buttons, close buttons, and existing
  Displayable/UIToolkit elements. The quest prototype will change; this work
  provides the general infrastructure.
- Handling the `QuestDeckBrowserObjectLayout` filter button or scrollbar
  interactions.

## Open Questions

None.

## References

- `client/Assets/Dreamtides/Abu/DreamtidesSceneWalker.cs` -- main Walk()
  dispatch
- `client/Assets/Dreamtides/Abu/SceneWalker/DreamtidesSceneWalker.Ui.cs` --
  quest walk path, `WalkFallbackScene3D`, `TryAddCanvasButton`
- `client/Assets/Dreamtides/Abu/SceneWalker/DreamtidesSceneWalker.Battle.cs` --
  `TryAddCloseBrowserButton` pattern
- `client/Assets/Dreamtides/Abu/SceneWalker/DreamtidesSceneWalker.Input.cs` --
  `BuildCanvasButtonCallbacks`
- `client/Assets/Dreamtides/Abu/DreamtidesSettledProvider.cs` -- settled
  detection
- `client/Assets/Dreamtides/Abu/DefaultSettledProvider.cs` -- BusyToken pattern
  (line 48)
- `client/Assets/Dreamtides/Abu/BusyToken.cs` -- ref-counted busy token
- `client/Assets/Dreamtides/Components/DreamscapeMapCamera.cs` -- site button
  dictionary, `IsTransitioning`, transition coroutines
- `client/Assets/Dreamtides/Services/DreamscapeService.cs` -- `CloseSiteButton`,
  `CreateOpenSiteButton()`
- `client/Assets/Dreamtides/Buttons/CloseBrowserButton.cs` -- close button
  component
- `client/Assets/Dreamtides/Layout/QuestDeckBrowserObjectLayout.cs` -- quest
  deck browser close button
- `client/Assets/Dreamtides/Services/Registry.cs` -- service access, property
  patterns
- `client/Assets/Dreamtides/Services/ActionServiceImpl.cs` --
  `ApplyTestScenarioAction` early return (line 172)
- `client/Assets/Dreamtides/Prototype/PrototypeQuest.cs` -- `FocusSiteFlow`,
  `OnDebugScenarioAction`
- `client/Assets/Dreamtides/Prototype/PrototypeQuestTemptingOfferFlow.cs` --
  tempting offer flow
- `client/Assets/Dreamtides/Abu/SnapshotCommandHandler.cs` -- click dispatch,
  `NotifyActionDispatched` (line 261)
