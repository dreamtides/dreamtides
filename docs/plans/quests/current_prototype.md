# Current Quest Prototype: Technical Reference

Technical details of the quest prototype implementation. The prototype
demonstrates client UI patterns for quests but is not definitive -- the master
design document ([quests.md](quests.md)) supersedes all prototype decisions.
Read this when implementing quest features, understanding the existing quest
architecture, or migrating prototype logic to the rules engine.

## Table of Contents

- [Prototype File Overview](#prototype-file-overview)
- [Rust-Side Quest Types](#rust-side-quest-types)
- [Command and Data Flow](#command-and-data-flow)
- [Client Layout and Site System](#client-layout-and-site-system)
- [Prototype Interaction Flows](#prototype-interaction-flows)
- [Shared Mechanisms with Battles](#shared-mechanisms-with-battles)
- [Implementation Gaps](#implementation-gaps)

## Prototype File Overview

The prototype lives in `client/Assets/Dreamtides/Prototype/` with seven C#
files. Two production classes outside this directory are also prototype quality.

**Prototype directory:**

- `PrototypeQuest.cs`: Top-level orchestrator extending `Service`. Entry point
  for the quest prototype. Initializes the quest deck, dreamsigns, and identity
  card. Routes all user interactions via a string-based debug action dispatcher
  (`OnDebugScenarioAction`). Manages camera transitions between the map and
  individual sites. Constructs `UpdateQuestCommand` snapshots and sends them to
  `DreamscapeService`.
- `PrototypeCards.cs`: Card data model and group-keyed cache. Maintains card
  groups ("quest", "draft", "shop", "tempting-offer", "dreamsigns") with
  grow-only caches. Uses deterministic pseudo-random generation from a fixed
  seed for consistent card content. Returns the union of all groups on every
  update, enabling full-state snapshots.
- `PrototypeQuestCardViewFactory.cs`: Static utility for cloning `CardView`
  objects with new positions. Used by flows to create animation targets without
  mutating cached card state.
- `PrototypeQuestDraftFlow.cs`: Draft pick mechanic. Reveals 4 cards at a time
  from a 20-card draft deck. Player picks one, the rest move offscreen, and the
  next 4 are revealed. Continues until the draft deck is exhausted.
- `PrototypeQuestShopFlow.cs`: Shop mechanic. Displays 6 items (3 regular cards,
  1 restock icon, 2 dreamsigns) with essence prices. Includes NPC merchant
  animation, typewriter dialog via the Masonry UI system, and restock
  functionality that rotates the shop group window.
- `PrototypeQuestTemptingOfferFlow.cs`: Risk/reward event. Presents two offers,
  each with a Journey card (reward) and Cost card (price). The selected offer
  triggers a multi-step animated sequence with dissolve effects, projectiles,
  and reverse-dissolve reveals of quest effect cards.
- `PrototypeQuestBattleFlow.cs`: Battle startup. Creates player and enemy
  identity cards with 3D character prefabs, repositions the camera, and displays
  the "Start Battle" button.

**Production classes (prototype quality):**

- `DreamscapeLayout.cs` in `client/Assets/Dreamtides/Layout/`: Centralized
  container of serialized references to all quest layout positions. Holds named
  `ObjectLayout` references for quest deck, draft picks, shop, dreamsigns,
  tempting offers, battle start, and more. Delegates actual positioning to
  specialized layout subclasses. Accessed globally via
  `Registry.DreamscapeLayout`.
- `DreamscapeService.cs` in `client/Assets/Dreamtides/Services/`: Handles
  `UpdateQuestCommand` dispatch. Updates essence display, screen overlay,
  buttons, tempting offer state, and delegates card updates to `CardService`.
  Manages dreamscape activation based on game mode. Provides site lookup by
  `SiteId` for animations and layout references.

## Rust-Side Quest Types

The quest system has one dedicated crate (`quest_state`) plus display types
spread across `display_data` and `core_data`.

**State types:**

- `QuestState` in `quest_state`: Holds `QuestId`, `UserState`, `Deck`, and
  `Essence`. Both human and AI players have their own quests. AI enemies build
  decks through simulated quests.
- `Deck`: A `Vec<CardDefinition>` indexed by `QuestDeckCardId(usize)`. Provides
  `insert_copies` (add N copies by `BaseCardId`) and `push_card_and_get_id`.
- `Essence`: Newtype `(pub u32)` representing quest currency.
- `QuestId` and `SiteId`: UUID-based identifiers in `core_data::identifiers`.

**Display types in `display_data`:**

- `QuestView`: The full visual state snapshot. Contains `cards` (all displayed
  `CardView` objects), `interface` (shared `InterfaceView` for UI chrome),
  `sites` (list of `SiteView` with id and icon), `essence_total`, optional
  `close_site_button`, and optional `tempting_offer`.
- `UpdateQuestCommand`: Wraps a `QuestView` plus optional `AudioClipAddress`.
  Parallel to `UpdateBattleCommand`.
- `SiteView`, `TemptingOfferView`, `TemptingOfferAction`, `CloseButtonView`:
  Supporting view types for quest-specific UI elements.

**Quest-specific `Position` enum variants** in `object_position.rs`:
`QuestDeck`, `QuestUserIdentityCard`, `QuestDeckBrowser`, `DestroyedQuestCards`,
`DreamsignDisplay`, `SiteDeck(SiteId)`, `SiteNpc(SiteId)`, `DraftPickDisplay`,
`ShopDisplay`, `JourneyDisplay`, `TemptingOfferDisplay(TemptingOfferPosition)`,
`QuestEffect`, `StartBattleCardOrigin(SiteId)`,
`StartBattleDisplay(StartBattleDisplayType)`.

**Quest-specific card prefabs**: `Dreamsign`, `Journey`, `OfferCost` in the
`CardPrefab` enum alongside battle prefabs like `Character` and `Event`.

**Quest-specific animation commands**: `ShowInDraftPickLayout`,
`ShowInShopLayout`, `HideShopLayout`, `MoveToQuestDeckOrDestroy`,
`MoveToDreamsignDisplayOrDestroy` as `MoveCardsCustomAnimation` variants.

**Quest object targets**: `QuestObjectId::EssenceTotal` and
`QuestObjectId::QuestDeck` for projectile and effect targeting.

## Command and Data Flow

**Schema generation pipeline:** Rust display types derive `Serialize`,
`Deserialize`, and `JsonSchema`. The `just schema` command generates a JSON
Schema from the Rust types, then `quicktype` converts it to C# classes in
`Schema.cs`. Both sides use JSON as the wire format (serde on Rust,
Newtonsoft.Json on C#).

**Production command flow (planned):**

- Player performs a quest action
- `ActionServiceImpl.PerformAction()` sends the action to the Rust rules engine
- The engine processes the action and returns a `CommandSequence`
- `ActionServiceImpl.ApplyGroup()` checks for `command.UpdateQuest != null` and
  routes to `DreamscapeService.HandleUpdateQuestCommand()`
- `HandleUpdateQuestCommand` updates essence, overlay, buttons, tempting offer,
  and delegates card updates to `CardService.HandleUpdateQuestCards()`
- `CardService` applies the card list through the same `ApplyUpdate()` method
  used for battles

**Prototype command flow (current):**

- User clicks a site button, triggering a `DebugAction(ApplyTestScenarioAction)`
- `ActionServiceImpl` routes this to `PrototypeQuest.OnDebugScenarioAction()` on
  the client, bypassing the Rust engine entirely
- `PrototypeQuest` constructs `UpdateQuestCommand` objects manually and sends
  them to `DreamscapeService.HandleUpdateQuestCommand()`
- All quest logic (drafting, shopping, tempting offers) runs client-side in the
  prototype flow classes

**Full-state snapshot model:** Every update rebuilds the complete list of all
cards across all groups and sends it as a single `UpdateQuestCommand`. There is
no incremental delta system. This matches the battle system's approach.

## Client Layout and Site System

**Layout hierarchy:** `DreamscapeLayout` holds references to specialized
`ObjectLayout` subclasses. The layout class hierarchy is:

- `ObjectLayout` (abstract base): `Add`, `Remove`, `ApplyLayout`,
  `ApplyTargetTransform`
  - `StandardObjectLayout`: World-space positioning with DOTween animations
    - `SitePickObjectLayout`: Grid layout for draft picks, shops, journey
      choices. Configurable rows/columns with close button positioning.
    - `TemptingOfferObjectLayout`: 2-per-row offers with per-row accept buttons
    - `StartBattleObjectLayout`: Identity card + dreamsign positioning + start
      button
  - `RenderAsChildObjectLayout`: Local-space positioning, parents children
    - `QuestDeckObjectLayout`: Pile arrangement along Z-axis
    - `DreamsignDisplayLayout`: 2-column grid for dreamsign tokens
  - `QuestDeckBrowserObjectLayout`: UI ScrollView-based grid with world-space
    mapping

**Site types:** Three concrete types extend `AbstractDreamscapeSite`:

- `CharacterSite`: NPC sites with 3 cameras (screen-left, screen-right,
  screen-top), a character model with Mecanim animator, speech position anchor,
  and character-owned objects layout.
- `DraftSite`: Draft selection sites with a single camera and a `SiteDeckLayout`
  for the site's card deck.
- `BattleSite`: Battle encounter sites with portrait/landscape layout anchors
  and a `BattleCardOrigin` layout. Aligns the battle layout to the site's
  world-space position when active.

All sites have a `SiteId` (GUID), active/inactive state, a `ButtonLabel`, and
Cinemachine camera management with priority-based activation and custom blend
curves.

**Camera system:** `DreamscapeMapCamera` frames all sites on startup by
computing a bounding volume and positioning the camera at a 50-degree downward
angle. Transitions between map and sites use Cinemachine blending with
configurable blend durations and animation curves.
`DreamscapeSiteButtonPositioner` implements a non-overlapping button placement
algorithm using cost-based search with safe area support for notched devices.

## Prototype Interaction Flows

All flows are plain C# classes (no MonoBehaviour) that receive dependencies via
constructor closure injection. They use Unity coroutine sequencing for multi-
frame operations.

**Navigation model:** The map shows site buttons positioned over 3D scene
locations. Clicking a site triggers `FocusSiteFlow()`, which runs a preparation
coroutine, animates the camera to the site's Cinemachine camera, then runs an
on-focused coroutine. Returning to the map uses
`DreamscapeMapCamera.ActivateWithTransition()`.

**Card interaction pattern:** Cards are configured with click action strings
formatted as `"action-name/cardId"`. These flow to `OnDebugScenarioAction()`
which parses and dispatches to the appropriate flow handler. Guard methods
prevent stale or invalid clicks.

**Draft flow:** Prepare 20 face-down cards in the draft group. Reveal 4 at a
time at `DraftPickDisplay` with staggered animations. Player picks one (moves to
quest deck), rejected cards move offscreen. Draft group window advances by 4.
Repeats until fewer than 4 cards remain, then returns to map.

**Shop flow:** Display 6 items at `ShopDisplay` with staggered animation. NPC
performs a Mecanim wave animation, typewriter dialog bubble appears via Masonry
UI. Purchasing a regular card animates it face-down to the quest deck with a
blue diamond projectile trail. Purchasing a dreamsign animates it to the
dreamsign display. Restocking hides current items, advances the group window by
6, and re-displays.

**Tempting offer flow:** Display 4 cards (2 Journey/Cost pairs) at
`TemptingOfferDisplay` with accept buttons per offer. Accepting triggers a
multi-step sequence: selected Journey card moves to `QuestEffect`, unselected
offer moves to destroyed, Journey dissolves with gold color and spawns quest
effect cards via reverse-dissolve, effect cards move to quest deck, Cost card
fires a blue projectile to the essence display then dissolves.

**Battle flow:** Creates player and enemy identity cards with 3D character
prefabs. Repositions camera to battle layout bounds. Animates enemy identity
card sliding into position. Shows "Start Battle" button.

## Shared Mechanisms with Battles

- **Command pipeline:** `UpdateQuest` and `UpdateBattle` are sibling variants of
  the `Command` enum. All animation commands (`FireProjectile`, `DissolveCard`,
  `MoveCardsWithCustomAnimation`, `PlayMecanimAnimation`,
  `AnchorToScreenPosition`) work for both contexts.
- **CardView:** Both systems render cards through identical `CardView` objects.
  `CardService` uses the same `ApplyUpdate()` method for quest and battle card
  diffing.
- **InterfaceView:** The same struct provides buttons, overlays, card order
  selectors, and browser state for both systems.
- **Position enum:** Quest positions and battle positions are variants of the
  same `Position` enum, so the layout system handles both uniformly.
- **Same Unity scene:** Quest and battle layouts coexist. `DreamscapeService`
  activates or deactivates quest UI based on `GameMode`.
- **Save/Load:** `SaveFile` wraps `QuestSaveFile` which contains optional
  `BattleState`. Battle card definitions are rebuilt from the quest deck on
  deserialization via `quest.deck.get_card()`.

## Implementation Gaps

The following components exist for battles but have no quest equivalents yet:

- **No `quest_mutations` or `quest_queries` crates.** Quest state is currently
  constructed directly in `game_creation` and `deserialize_save_file`. The
  mutation/query layer for drafting, shopping, essence transactions, site
  visits, and dreamscape progression has not been built.
- **No `QuestAction` variant in `GameAction`.** Quest interactions use
  `DebugAction(ApplyTestScenarioAction)` routed to client-side prototype code. A
  dedicated `QuestAction` variant with quest-specific actions (draft pick, shop
  purchase, site visit, etc.) needs to be added.
- **No quest action processing in the engine.** `handle_request_action()` in
  `engine.rs` currently only dispatches battle actions. Quest action handling
  needs to be added alongside.
- **Quest deck browser is stubbed.**
  `BattleDisplayAction::BrowseCards( CardBrowserType::QuestDeck)` has a
  `todo!()` placeholder.
- **QuestState is minimal.** No tracking of visited sites, dreamscape
  progression, dreamcaller selection, or active dreamsigns. The `Deck` type
  stores cards but has no concept of draft pools, shop inventories, or site
  encounter state.
