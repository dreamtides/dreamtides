# Technical Design: Abu Quest Mode Snapshot Support

## Goal

Enable the abu snapshot system to produce a high-quality, structured
accessibility tree for quest mode, following the same site-specific approach
used by `DreamtidesSceneWalker.Battle.cs`. The quest walker should enumerate
every quest-mode layout zone (quest deck, shop, draft picks, tempting offers,
start battle, dreamsigns, etc.) with semantic grouping and card-level detail,
plus walk site navigation buttons and close buttons. Camera transitions must
also be detected by settled detection so abu waits for Cinemachine blends to
complete.

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

- `DreamtidesSceneWalker.cs` -- `Walk()` dispatch, helpers
- `SceneWalker/DreamtidesSceneWalker.Ui.cs` -- fallback walk
  (`WalkFallbackScene3D`), canvas button walking, UIToolkit walking
- `SceneWalker/DreamtidesSceneWalker.Battle.cs` -- battle mode walk
- `SceneWalker/DreamtidesSceneWalker.Input.cs` -- callback builders

The top-level `Walk()` method checks
`_registry.BattleLayout.Contents.activeSelf`. When true, it calls
`WalkBattle()`. When false (quest mode), it currently calls `WalkUiToolkit()` +
`WalkFallbackScene3D()`, which produce a flat, unstructured tree.

### Battle Walker as Template

`DreamtidesSceneWalker.Battle.cs` demonstrates the target quality level. It:

1. Creates a `"Battle"` region node as root.
2. Walks **controls** (menu, undo, dev, bug, close browser buttons).
3. Walks each **player** (user and opponent) with sub-groups: status (energy,
   score, spark breakdown, turn), browser buttons (deck, identity, void with
   card counts), battlefield cards (with rules text and annotations), hand cards
   (with playability info).
4. Walks the **stack** with position labels like `[2 of 3, top]`.
5. Walks **game modifiers**, **action buttons**, **playable cards summary**,
   **essence label**, **play zone** (drag target), **thinking indicator**.
6. Walks **card order selector** with drag-to-target support.
7. Walks the **browser** (modal card browser).
8. Walks **UIToolkit overlays** (filtered for content).

Each card node uses `BuildCardLabel()` which includes name, card type, cost,
spark, "can play" / "selected" annotations, and rules text. Interactive nodes
get `RefCallbacks` with click/hover/drag handlers.

### Quest Mode Layouts

`DreamscapeLayout` (accessed via `_registry.DreamscapeLayout`) exposes all quest
zone layouts:

| Property                | Type                           | Contents                              |
| ----------------------- | ------------------------------ | ------------------------------------- |
| `QuestDeck`             | `ObjectLayout`                 | Face-down deck cards                  |
| `QuestUserIdentityCard` | `ObjectLayout`                 | User identity card                    |
| `QuestDeckBrowser`      | `QuestDeckBrowserObjectLayout` | Deck browser (portrait/landscape)     |
| `EssenceTotal`          | `EssenceTotal`                 | Currency display with `_originalText` |
| `DraftPickLayout`       | `SitePickObjectLayout`         | Draft pick choices                    |
| `DestroyedQuestCards`   | `ObjectLayout`                 | Cards removed from game               |
| `ShopLayout`            | `StandardObjectLayout`         | Shop card offerings                   |
| `DreamsignDisplay`      | `DreamsignDisplayLayout`       | Enemy dreamsigns (2-col)              |
| `JourneyChoiceDisplay`  | `StandardObjectLayout`         | Journey/route choices                 |
| `TemptingOfferDisplay`  | `TemptingOfferObjectLayout`    | Event offer cards                     |
| `QuestEffectPosition`   | `ObjectLayout`                 | Animating effect cards                |
| `StartBattleLayout`     | `StartBattleObjectLayout`      | Pre-battle identity + dreamsigns      |

### Site Buttons and Close Buttons

Site `CanvasButton`s live in `DreamscapeMapCamera._siteButtonsBySite` (a
`Dictionary<AbstractDreamscapeSite, CanvasButton>`). They are shown on the map
and hidden when navigating to a site.

Two `CloseBrowserButton` instances exist in quest mode:

1. **`DreamscapeService.CloseSiteButton`** -- `CanvasGroup` with a
   `CloseBrowserButton` component (retrieved via `GetComponent`). Used for
   shop/tempting offer close.
2. **`QuestDeckBrowserObjectLayout._closeButton`** -- close button for the quest
   deck browser.

### Tempting Offer Accept Buttons

`TemptingOfferObjectLayout` dynamically creates `DisplayableButton` instances
(stored in `_acceptButtons`) that display "Accept" labels. These are positioned
per-row and are `Displayable` subclasses with `CanHandleMouseEvents() => true`.
The generic `WalkDisplayables` would pick these up, but in the structured quest
walker they should be walked explicitly alongside their associated offer cards
for semantic clarity.

### Start Battle Button

`StartBattleObjectLayout` creates a `DisplayableButton` instance
(`_buttonInstance`) with a "Start Battle" label. Like accept buttons, this is a
`Displayable` that should be walked explicitly in the start battle group.

### Camera Transitions and Settled Detection

Clicking a site button fires `PrototypeQuest.OnDebugScenarioAction()` which
starts a `FocusSiteFlow` coroutine. Camera transitions use Cinemachine blending
(~2 seconds) which produces zero DOTween tweens.
`DreamtidesSettledProvider.IsLocalSettled()` only checks `IsProcessingCommands`
and `DOTween.TotalPlayingTweens()`, so it reports settled immediately before the
camera finishes moving.

`BusyToken` is a ref-counted disposable. `DefaultSettledProvider` already checks
`BusyToken.IsAnyActive`; `DreamtidesSettledProvider` does not.

## Design

### New File: `DreamtidesSceneWalker.Quest.cs`

Create a new partial class file at
`client/Assets/Dreamtides/Abu/SceneWalker/DreamtidesSceneWalker.Quest.cs`
containing the quest mode walker. This follows the established pattern of one
file per mode (`.Battle.cs`, `.Ui.cs`, `.Input.cs`).

### Walk() Dispatch Change

Modify `Walk()` in `DreamtidesSceneWalker.cs` to add a quest mode branch:

```
if (_registry.BattleLayout.Contents.activeSelf)
{
  root.Children.Add(WalkBattle(refRegistry));
}
else
{
  root.Children.Add(WalkQuest(refRegistry));
}
```

The old fallback path (`WalkUiToolkit()` + `WalkFallbackScene3D()`) is replaced
by `WalkQuest()`. UIToolkit overlays will be walked as the last child of the
quest region, using `WalkUiToolkitFiltered()` (the same filtered version battle
mode uses).

### Quest Walker Tree Structure

`WalkQuest()` builds the following tree. Each section is only included when it
has content (following the battle walker's `count > 0` pattern).

```
region "Quest"
├── group "Controls"
│   ├── button "Menu"
│   ├── button "Undo"
│   ├── button "Dev"
│   ├── button "Bug Report"
│   ├── button "Close Browser"       (if quest deck browser close active)
│   └── button "Close Site"          (if site close button active)
│
├── group "Map"                      (if site buttons visible, i.e. on map)
│   ├── button "Draft"               (site button labels from ButtonLabel)
│   ├── button "Shop"
│   ├── button "Event"
│   └── ...
│
├── label "Essence: 75"
│
├── group "Quest Deck (43 cards)"
│   └── button "Browse Quest Deck"   (interactive, clicks quest deck layout)
│
├── group "Identity"
│   └── button "Card Name, ..."      (user identity card with card details)
│
├── group "Dreamsigns"               (if dreamsign display has objects)
│   ├── button "Hourglass, Dreamsign"
│   ├── button "Garlic, Dreamsign"
│   └── ...
│
├── group "Draft Picks"              (if draft pick layout has objects)
│   ├── button "Card A, Character (cost: 3, spark: 2) -- rules text"
│   ├── button "Card B, ..."
│   └── ...
│
├── group "Shop"                     (if shop layout has objects)
│   ├── button "Card A, Character (cost: 3) -- rules text"
│   ├── button "Card B, ..."
│   └── ...
│
├── group "Tempting Offer"           (if tempting offer has objects)
│   ├── button "Card A, Character -- rules text"
│   ├── button "Accept"              (accept button for row)
│   ├── button "Card B, ..."
│   ├── button "Accept"
│   └── ...
│
├── group "Start Battle"             (if start battle layout has objects)
│   ├── button "Enemy Identity, ..."
│   ├── button "Dreamsign A"
│   ├── button "Start Battle"        (the DisplayableButton)
│   └── ...
│
├── group "Journey Choices"          (if journey choice display has objects)
│   ├── button "Choice A"
│   └── ...
│
├── group "Quest Deck Browser"       (if quest deck browser has objects)
│   ├── button "Card A, Character (cost: 3, spark: 2) -- rules text"
│   ├── button "Card B, ..."
│   └── ...
│
├── group "Card Order Selector"      (reuses battle walker's WalkCardOrderSelector)
│
└── region "UIToolkit"               (filtered, only when content exists)
    └── ...
```

### Method Breakdown

**`WalkQuest(RefRegistry refRegistry) → AbuSceneNode`**

Top-level quest walker. Creates a `"Quest"` region, then conditionally adds each
section:

```csharp
var region = CreateRegionNode("Quest");
var layout = _registry.DreamscapeLayout;

// 1. Controls (always)
region.Children.Add(WalkQuestControls(refRegistry));

// 2. Map site buttons (when visible)
var mapGroup = WalkMapSiteButtons(refRegistry);
if (mapGroup != null) region.Children.Add(mapGroup);

// 3. Essence
AddEssenceLabel(region);

// 4. Quest Deck summary + browse button
AddQuestDeckSummary(region, layout, refRegistry);

// 5. Identity card
var identityGroup = WalkObjectLayoutGroup("Identity", layout.QuestUserIdentityCard, refRegistry);
if (identityGroup != null) region.Children.Add(identityGroup);

// 6. Dreamsigns
var dreamsignsGroup = WalkObjectLayoutGroup("Dreamsigns", layout.DreamsignDisplay, refRegistry);
if (dreamsignsGroup != null) region.Children.Add(dreamsignsGroup);

// 7. Draft picks
var draftGroup = WalkObjectLayoutGroup("Draft Picks", layout.DraftPickLayout, refRegistry);
if (draftGroup != null) region.Children.Add(draftGroup);

// 8. Shop
var shopGroup = WalkObjectLayoutGroup("Shop", layout.ShopLayout, refRegistry);
if (shopGroup != null) region.Children.Add(shopGroup);

// 9. Tempting offer (with accept buttons)
var offerGroup = WalkTemptingOffer(layout, refRegistry);
if (offerGroup != null) region.Children.Add(offerGroup);

// 10. Start battle (with start button)
var battleGroup = WalkStartBattle(layout, refRegistry);
if (battleGroup != null) region.Children.Add(battleGroup);

// 11. Journey choices
var journeyGroup = WalkObjectLayoutGroup("Journey Choices", layout.JourneyChoiceDisplay, refRegistry);
if (journeyGroup != null) region.Children.Add(journeyGroup);

// 12. Quest deck browser
var browserGroup = WalkObjectLayoutGroup("Quest Deck Browser", layout.QuestDeckBrowser, refRegistry);
if (browserGroup != null) region.Children.Add(browserGroup);

// 13. Card order selector (reuse from battle walker)
var cardOrderGroup = WalkCardOrderSelector(_registry.BattleLayout, refRegistry);
if (cardOrderGroup != null) region.Children.Add(cardOrderGroup);

// 14. UIToolkit overlays (filtered)
var uiOverlay = WalkUiToolkitFiltered(refRegistry);
if (uiOverlay != null) region.Children.Add(uiOverlay);

return region;
```

**`WalkQuestControls(RefRegistry refRegistry) → AbuSceneNode`**

Similar to `WalkControls()` in battle mode. Walks DocumentService buttons plus
both quest-mode close buttons:

```csharp
var group = CreateGroupNode("Controls");
var doc = _registry.DocumentService;

TryAddCanvasButton(group, refRegistry, doc.MenuButton, "Menu");
TryAddCanvasButton(group, refRegistry, doc.UndoButton, "Undo");
TryAddCanvasButton(group, refRegistry, doc.DevButton, "Dev");
TryAddCanvasButton(group, refRegistry, doc.BugButton, "Bug Report");

// Quest deck browser close button
TryAddCloseBrowserButton(group, refRegistry,
    _registry.DreamscapeLayout.QuestDeckBrowser._closeButton,
    "Close Browser");

// Site close button (on DreamscapeService._closeSiteButton CanvasGroup)
var closeSiteCanvasGroup = _registry.DreamscapeService.CloseSiteButton;
var closeSiteButton = closeSiteCanvasGroup.GetComponent<CloseBrowserButton>();
if (closeSiteButton != null)
{
    TryAddCloseBrowserButton(group, refRegistry, closeSiteButton, "Close Site");
}

return group;
```

The `TryAddCloseBrowserButton` helper is extracted from the existing battle-mode
version into a shared method with a label parameter:

```csharp
void TryAddCloseBrowserButton(
    AbuSceneNode parent, RefRegistry refRegistry,
    CloseBrowserButton button, string label)
{
    if (button == null || !button.gameObject.activeSelf) return;
    AddInteractiveNode(parent, refRegistry, "button", label,
        new RefCallbacks { OnClick = () => button.OnClick() });
}
```

The existing battle-mode `TryAddCloseBrowserButton` (no parameters beyond
parent/refRegistry) should be refactored to call this shared version with the
battle layout's `CloseBrowserButton` and `"Close Browser"` label.

**`WalkMapSiteButtons(RefRegistry refRegistry) → AbuSceneNode?`**

Discovers site buttons from `DreamscapeMapCamera._siteButtonsBySite`:

```csharp
var camera = Object.FindFirstObjectByType<DreamscapeMapCamera>(
    FindObjectsInactive.Exclude);
if (camera == null) return null;

var group = CreateGroupNode("Map");
foreach (var button in camera._siteButtonsBySite.Values)
{
    TryAddCanvasButton(group, refRegistry, button);
}
return group.Children.Count > 0 ? group : null;
```

`TryAddCanvasButton` already handles visibility checks and label extraction.
Sites with no `ButtonLabel` or inactive buttons are skipped.

**`AddQuestDeckSummary(AbuSceneNode parent, DreamscapeLayout layout, RefRegistry refRegistry)`**

Shows quest deck card count and a browse button:

```csharp
var deckCount = layout.QuestDeck.Objects.Count;
if (deckCount == 0) return;

var group = CreateGroupNode($"Quest Deck ({deckCount} cards)");
AddInteractiveNode(group, refRegistry, "button", "Browse Quest Deck",
    BuildDisplayableCallbacks(layout.QuestDeck));
parent.Children.Add(group);
```

The `BuildDisplayableCallbacks(layout.QuestDeck)` call registers a click that
simulates clicking the quest deck `Displayable`, which triggers the deck browser
via the existing prototype quest flow.

**`WalkTemptingOffer(DreamscapeLayout layout, RefRegistry refRegistry) → AbuSceneNode?`**

Walks tempting offer cards plus their per-row accept buttons:

```csharp
var offerLayout = layout.TemptingOfferDisplay;
if (offerLayout.Objects.Count == 0) return null;

var group = CreateGroupNode("Tempting Offer");
foreach (var obj in offerLayout.Objects)
{
    var cardNode = BuildCardNode(obj, "Browser", refRegistry);
    if (cardNode != null) group.Children.Add(cardNode);
}

// Walk accept buttons (DisplayableButton instances)
foreach (var button in offerLayout._acceptButtons)
{
    if (button == null || !button.gameObject.activeSelf) continue;
    var label = ToSingleLineText(button._text.text, fallback: "Accept");
    AddInteractiveNode(group, refRegistry, "button", label,
        BuildDisplayableCallbacks(button));
}

return group.Children.Count > 0 ? group : null;
```

This requires `_acceptButtons` on `TemptingOfferObjectLayout` to be accessible.
It is currently `readonly List<DisplayableButton>` (private). Change it to
`internal readonly` to match the codebase convention for fields accessed by the
scene walker (e.g., `DreamscapeMapCamera._siteButtonsBySite` is `internal`).

**`WalkStartBattle(DreamscapeLayout layout, RefRegistry refRegistry) → AbuSceneNode?`**

Walks start battle cards plus the start button:

```csharp
var startLayout = layout.StartBattleLayout;
if (startLayout.Objects.Count == 0) return null;

var group = CreateGroupNode("Start Battle");
foreach (var obj in startLayout.Objects)
{
    var cardNode = BuildCardNode(obj, "Browser", refRegistry);
    if (cardNode != null) group.Children.Add(cardNode);
}

// Walk the "Start Battle" button
if (startLayout._buttonInstance != null
    && startLayout._buttonInstance.gameObject.activeSelf)
{
    var label = ToSingleLineText(startLayout._buttonInstance._text.text,
        fallback: "Start Battle");
    AddInteractiveNode(group, refRegistry, "button", label,
        BuildDisplayableCallbacks(startLayout._buttonInstance));
}

return group.Children.Count > 0 ? group : null;
```

This requires `_buttonInstance` on `StartBattleObjectLayout` to be accessible.
Change it from `DisplayableButton?` (private) to `internal DisplayableButton?`.

### Card Label Reuse

The existing `BuildCardNode()` and `BuildCardLabel()` methods from the battle
walker handle all card types correctly. They use `zoneContext` to control which
annotations to show (cost in "Hand"/"Browser", spark in detail zones, etc.).

For quest zone cards, use `"Browser"` as the zoneContext for all quest layouts
(draft picks, shop, tempting offer, start battle, deck browser). This shows cost
and spark -- the most useful information for quest decision-making. For
dreamsigns and the quest deck (face-down), they won't have revealed data and
`BuildCardNode` returns `null` for unrevealed cards, which is correct.

Actually, dreamsigns DO have revealed data (they have names like "Hourglass").
The `BuildCardNode` method checks `card.CanHandleMouseEvents()` which returns
true for dreamsigns, so they will be included. Their `zoneContext` should be
`"Browser"` to show their name and type.

### Shared Method Extraction

Several helpers should be extracted from `.Battle.cs` to the main
`DreamtidesSceneWalker.cs` file so both battle and quest walkers can use them:

1. `WalkObjectLayoutGroup(string label, ObjectLayout layout, RefRegistry refRegistry)`
   -- already in `.Battle.cs` line 808, walks an ObjectLayout's objects as a
   card group. Move to main file.

2. `AddEssenceLabel(AbuSceneNode parent)` -- already in `.Battle.cs` line 608.
   Move to main file. It reads from `_registry.DreamscapeLayout.EssenceTotal`
   which is shared between battle and quest.

3. `WalkCardOrderSelector(BattleLayout layout, RefRegistry refRegistry)` --
   already in `.Battle.cs` line 655. Keep in `.Battle.cs` but call it from quest
   walker too (it's accessible as a partial class method).

4. `TryAddCloseBrowserButton` -- extract the shared version with label parameter
   to main file.

### Accessibility for Quest Deck

The quest deck (`layout.QuestDeck`) contains face-down cards. These won't have
`Revealed` data, so `BuildCardNode` returns `null` for them. The walker should
not walk individual deck cards -- instead it shows the count and a browse button
(via `AddQuestDeckSummary`).

However, if the deck contains any face-up cards (e.g., after a foresee effect),
they would be walked. This is the correct behavior.

### Site-Owned Layouts

Some layouts live on individual site GameObjects rather than on
`DreamscapeLayout`:

- `DraftSite._siteDeckLayout` -- cards displayed at the draft site during the
  pick animation
- `CharacterSite._characterOwnedObjects` -- cards owned by a character site
  (merchant's displayed wares)
- `BattleSite._battleCardOrigin` -- card spawn position for battle transitions

These are auxiliary animation positions. Cards in these layouts are also tracked
in the main layouts (shop, draft pick, etc.) and will be walked there. Walking
site-owned layouts separately would produce duplicates. Do not walk them.

### Settled Detection for Camera Transitions

`DreamtidesSettledProvider.IsLocalSettled()` must be extended to treat
`BusyToken.IsAnyActive` as a "not settled" signal, following the same pattern
used by `DefaultSettledProvider`: when `BusyToken.IsAnyActive` is true, return
false.

Camera transitions in `DreamscapeMapCamera` must acquire a `BusyToken` for their
duration. The token should be acquired inside the camera's transition coroutines
(`TransitionToSite` and `TransitionToMap`) and disposed when the coroutine
completes (including on early exit or error). Use `try/finally` to ensure
disposal on all exit paths.

```csharp
IEnumerator TransitionToSite(AbstractDreamscapeSite site)
{
    using var busyToken = new BusyToken();
    // ... existing body unchanged ...
}

IEnumerator TransitionToMap()
{
    using var busyToken = new BusyToken();
    // ... existing body unchanged ...
}
```

Note: C# `using var` in IEnumerator does NOT work as expected because
IEnumerator methods are compiled into state machines. The `using` block's
`Dispose` would be called when the state machine is garbage collected, not when
the coroutine completes. Instead, use explicit try/finally:

```csharp
IEnumerator TransitionToSite(AbstractDreamscapeSite site)
{
    var busyToken = new BusyToken();
    try
    {
        // ... existing body ...
    }
    finally
    {
        busyToken.Dispose();
    }
}
```

### Walk Order Consistency

Adding new interactive nodes to the quest walk path changes ref numbering for
subsequent nodes. This is by design -- `RefRegistry` and `SnapshotFormatter`
both use depth-first traversal and stay in sync as long as both see the same
tree. No special synchronization is needed.

### Tempting Offer Animation Race

`HandleTemptingOfferSelection` calls
`_registry.StartCoroutine(ResolveTemptingOfferSelection(offerNumber))`, which
runs outside the `FocusSiteFlow` yield chain. The coroutine's animations all use
DOTween (dissolves, card moves, projectiles), which the settled provider already
detects. The gap between coroutine start and the first DOTween tween creation is
covered by the 3-frame settle requirement. This does not need a BusyToken.

## Field Visibility Changes

The following fields need their visibility changed from `private` to `internal`
to be accessible by the scene walker (both are in the `Dreamtides` assembly):

1. `TemptingOfferObjectLayout._acceptButtons` --
   `readonly List<DisplayableButton>` →
   `internal readonly List<DisplayableButton>`
2. `StartBattleObjectLayout._buttonInstance` -- `DisplayableButton?` →
   `internal DisplayableButton?`

This follows the established pattern: `DreamscapeMapCamera._siteButtonsBySite`
is `internal`, `QuestDeckBrowserObjectLayout._closeButton` is `internal`, etc.

## Constraints

- **Existing battle mode must not regress.** Battle mode snapshot behavior must
  remain unchanged. The scene walker changes are gated by the existing
  battle/non-battle branch in `Walk()`.

- **Real-world QA validation.** Every change must be validated by running the
  Unity editor in quest mode, using `abu snapshot` and `abu screenshot` to
  verify that: (a) site buttons appear in the snapshot with correct labels, (b)
  clicking a site button via `abu click` waits for the camera transition to
  complete before returning, (c) the close button appears in the snapshot when
  at a site, (d) clicking close returns to the map and the snapshot shows site
  buttons again, (e) quest deck, identity, dreamsigns, shop, draft, tempting
  offer, and start battle layouts all appear correctly in snapshots.

- **BusyToken lifecycle.** Tokens must be disposed on all exit paths (normal
  completion, early return, exception). Use `try/finally` in coroutines, not
  `using var`.

- **Card node reuse.** Quest card nodes must use the same `BuildCardNode()` and
  `BuildCardLabel()` methods as battle mode to ensure consistent formatting.

## Non-Goals

- Automated integration tests for quest-mode snapshots. The quest prototype
  requires a fully initialized Unity scene.
- Walking site-owned layouts (`DraftSite._siteDeckLayout`,
  `CharacterSite._characterOwnedObjects`, `BattleSite._battleCardOrigin`). These
  are animation-intermediate positions; cards are walked via their primary quest
  layouts.
- Walking the `QuestDeckBrowserObjectLayout` filter button or scrollbar
  interactions.
- Walking `QuestEffectPosition` or `DestroyedQuestCards` layouts (these contain
  cards in transit during animations and would add noise to snapshots).

## Open Questions

None.

## References

- `client/Assets/Dreamtides/Abu/DreamtidesSceneWalker.cs` -- Walk() dispatch,
  helpers, shared methods destination
- `client/Assets/Dreamtides/Abu/SceneWalker/DreamtidesSceneWalker.Battle.cs` --
  battle walker template, methods to extract
- `client/Assets/Dreamtides/Abu/SceneWalker/DreamtidesSceneWalker.Ui.cs` --
  current fallback walk (to be replaced), TryAddCanvasButton, UIToolkit walking
- `client/Assets/Dreamtides/Abu/SceneWalker/DreamtidesSceneWalker.Input.cs` --
  callback builders
- `client/Assets/Dreamtides/Abu/DreamtidesSettledProvider.cs` -- settled
  detection (needs BusyToken check)
- `client/Assets/Dreamtides/Abu/BusyToken.cs` -- ref-counted busy token
- `client/Assets/Dreamtides/Layout/DreamscapeLayout.cs` -- all quest zone
  layouts
- `client/Assets/Dreamtides/Layout/TemptingOfferObjectLayout.cs` --
  `_acceptButtons` (needs internal)
- `client/Assets/Dreamtides/Layout/StartBattleObjectLayout.cs` --
  `_buttonInstance` (needs internal)
- `client/Assets/Dreamtides/Layout/QuestDeckBrowserObjectLayout.cs` -- quest
  deck browser, `_closeButton`
- `client/Assets/Dreamtides/Components/DreamscapeMapCamera.cs` -- site button
  dictionary, transition coroutines (need BusyToken)
- `client/Assets/Dreamtides/Services/DreamscapeService.cs` -- `CloseSiteButton`
- `client/Assets/Dreamtides/Buttons/CloseBrowserButton.cs` -- close button
  component
- `client/Assets/Dreamtides/Buttons/DisplayableButton.cs` -- accept/start
  buttons (`_text` field)
- `client/Assets/Dreamtides/Sites/AbstractDreamscapeSite.cs` -- site base class
- `client/Assets/Dreamtides/Prototype/PrototypeQuest.cs` -- quest flows,
  OnDebugScenarioAction
