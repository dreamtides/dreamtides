#nullable enable

using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using Dreamtides.Buttons;
using Dreamtides.Components;
using Dreamtides.Prototype;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.Sites;
using UnityEngine;

public class PrototypeQuest : Service
{
  [SerializeField]
  internal string _outlineColorHex = "#EF6C00";

  [SerializeField]
  internal DreamscapeMapCamera _mapCamera = null!;

  public static readonly Guid DraftSiteId = Guid.Parse("117b9377-e0ab-4304-9f74-6d5b4fc5c778");
  public static readonly Guid ShopSiteId = Guid.Parse("4ce46579-7d0c-455a-a404-628894cff331");
  public static readonly Guid TemptingOfferSiteId = Guid.Parse(
    "2d9b1d2c-6637-4930-b9fc-a70fa901d662"
  );

  PrototypeCards _prototypeCards = new PrototypeCards();
  PrototypeQuestDraftFlow _draftFlow = null!;
  PrototypeQuestShopFlow _shopFlow = null!;
  PrototypeQuestTemptingOfferFlow _temptingOfferFlow = null!;
  List<CardOverride>? _pendingShopOverrides;
  bool _hasPendingShopOverridesUpdate;
  bool _battleStartupApplied;

  // Public API to configure arbitrary shop card overrides (index-based)
  public void ConfigureShopOverrides(params CardOverride[] overrides)
  {
    if (_shopFlow != null)
    {
      _shopFlow.ConfigureShopOverrides(overrides);
      return;
    }
    _pendingShopOverrides = overrides?.ToList();
    _hasPendingShopOverridesUpdate = true;
  }

  public void ClearShopOverrides()
  {
    if (_shopFlow != null)
    {
      _shopFlow.ClearShopOverrides();
      return;
    }
    _pendingShopOverrides = null;
    _hasPendingShopOverridesUpdate = true;
  }

  void EnsureFlowsInitialized()
  {
    if (_draftFlow != null && _shopFlow != null && _temptingOfferFlow != null)
    {
      return;
    }
    var registry = Registry;
    var startCoroutine = new Func<IEnumerator, Coroutine>(StartCoroutine);
    if (_temptingOfferFlow == null)
    {
      _temptingOfferFlow = new PrototypeQuestTemptingOfferFlow(
        registry,
        _prototypeCards,
        (request, animate) => CreateOrUpdateCards(request, animate),
        TemptingOfferSiteId
      );
    }
    if (_draftFlow == null)
    {
      _draftFlow = new PrototypeQuestDraftFlow(
        registry,
        _prototypeCards,
        (request, animate) => CreateOrUpdateCards(request, animate),
        () => StartCoroutine(ReturnToMap()),
        () => _outlineColorHex,
        DraftSiteId
      );
    }
    if (_shopFlow == null)
    {
      _shopFlow = new PrototypeQuestShopFlow(
        registry,
        _prototypeCards,
        (request, animate) => CreateOrUpdateCards(request, animate),
        startCoroutine,
        ShopSiteId
      );
    }
    ApplyPendingShopOverrides();
  }

  protected override void OnInitialize(GameMode _mode, TestConfiguration? testConfiguration)
  {
    EnsureFlowsInitialized();
    StartCoroutine(InitializeQuestSequence());
  }

  IEnumerator InitializeQuestSequence()
  {
    yield return StartCoroutine(
      CreateOrUpdateCards(
        new CreateOrUpdateCardsRequest
        {
          Count = 43,
          Position = new ObjectPosition
          {
            Position = new Position { Enum = PositionEnum.QuestDeck },
            SortingKey = 1,
          },
          Revealed = false,
          GroupKey = "quest",
        },
        animate: false
      )
    );

    yield return StartCoroutine(
      CreateOrUpdateCards(
        new CreateOrUpdateCardsRequest
        {
          Count = 4,
          Position = new ObjectPosition
          {
            Position = new Position { Enum = PositionEnum.DreamsignDisplay },
            SortingKey = 1,
          },
          Revealed = true,
          GroupKey = "dreamsigns",
          Overrides = new List<CardOverride>
          {
            new CardOverride
            {
              Index = 0,
              Prefab = CardPrefab.Dreamsign,
              Name = "Hourglass",
              SpritePath =
                "Assets/ThirdParty/AngelinaAvgustova/WitchCraftIcons/PNG/outline_hourglass.png",
            },
            new CardOverride
            {
              Index = 1,
              Prefab = CardPrefab.Dreamsign,
              Name = "Garlic",
              SpritePath =
                "Assets/ThirdParty/AngelinaAvgustova/WitchCraftIcons/PNG/outline_garlic.png",
            },
            new CardOverride
            {
              Index = 2,
              Prefab = CardPrefab.Dreamsign,
              Name = "Claw",
              SpritePath =
                "Assets/ThirdParty/AngelinaAvgustova/WitchCraftIcons/PNG/outline_claw.png",
            },
            new CardOverride
            {
              Index = 3,
              Prefab = CardPrefab.Dreamsign,
              Name = "Tooth",
              SpritePath =
                "Assets/ThirdParty/AngelinaAvgustova/WitchCraftIcons/PNG/outline_tooth.png",
            },
          },
        },
        animate: false
      )
    );
  }

  public void OnDebugScenarioAction(string name)
  {
    if (string.IsNullOrEmpty(name))
    {
      return;
    }

    if (name == "browseQuestDeck")
    {
      StartCoroutine(BrowseQuestDeck());
      return;
    }

    if (name == "closeQuestDeck")
    {
      StartCoroutine(CloseQuestDeck());
      return;
    }

    if (name == "closeShop")
    {
      EnsureFlowsInitialized();
      StartCoroutine(CloseShop());

      return;
    }

    if (name == "closeTemptingOffer")
    {
      EnsureFlowsInitialized();
      StartCoroutine(CloseTemptingOffer());

      return;
    }

    if (name == "FocusMapCamera")
    {
      StartCoroutine(ReturnToMap());
      return;
    }

    if (name == "FocusDraftCamera")
    {
      StartCoroutine(
        FocusSiteFlow(
          "FocusDraftCamera",
          () => _draftFlow.PrepareDraftDeck(),
          () => _draftFlow.RunDraftPickSequence()
        )
      );
      return;
    }

    if (name == "FocusShopCamera")
    {
      StartCoroutine(
        FocusSiteFlow(
          "FocusShopCamera",
          () => _shopFlow.PrepareShopCards(),
          () => _shopFlow.RunShopDisplaySequence()
        )
      );
      return;
    }

    if (name == "FocusEventCamera")
    {
      StartCoroutine(
        FocusSiteFlow(
          "FocusEventCamera",
          () => _temptingOfferFlow.PrepareTemptingOfferCards(),
          () => _temptingOfferFlow.ShowTemptingOfferCards()
        )
      );
      return;
    }

    if (name == "FocusEssenceCamera")
    {
      StartCoroutine(FocusSiteFlow("FocusEssenceCamera", null, null));
      return;
    }

    if (name == "FocusDraft2Camera")
    {
      StartCoroutine(FocusSiteFlow("FocusDraft2Camera", null, null));
      return;
    }

    if (name == "FocusBattleCamera")
    {
      StartCoroutine(FocusSiteFlow("FocusBattleCamera", null, ApplyBattleStartupRoutine));
    }

    var parts = name.Split('/');
    if (parts.Length != 2)
    {
      return;
    }
    var action = parts[0];
    var clickedId = parts[1];
    EnsureFlowsInitialized();
    if (action == "draft-pick")
    {
      if (!_draftFlow.HasDraftPick(clickedId))
      {
        return;
      }
      StartCoroutine(_draftFlow.ResolveDraftPick(clickedId));
      return;
    }
    if (action == "shop-pick")
    {
      if (!_shopFlow.HasShopCard(clickedId))
      {
        return;
      }
      StartCoroutine(_shopFlow.ResolveShopPick(clickedId));
      return;
    }
    if (_temptingOfferFlow.IsTemptingOfferAction(action))
    {
      _temptingOfferFlow.HandleTemptingOfferSelection(clickedId);
    }
  }

  IEnumerator CloseShop()
  {
    EnsureFlowsInitialized();
    _shopFlow.ClearDisplayedCards();
    Registry.DreamscapeService.HideCloseSiteButton();
    Registry.DocumentService.RenderScreenAnchoredNode(
      new AnchorToScreenPositionCommand() { Node = null }
    );
    yield return StartCoroutine(ReturnToMap());
  }

  IEnumerator CloseTemptingOffer()
  {
    EnsureFlowsInitialized();
    Registry.DreamscapeService.HideCloseSiteButton();
    Registry.DreamscapeLayout.TemptingOfferDisplay.HideAcceptButtons();
    Registry.DocumentService.RenderScreenAnchoredNode(
      new AnchorToScreenPositionCommand() { Node = null }
    );
    yield return StartCoroutine(ReturnToMap());
  }

  AbstractDreamscapeSite? FindSiteForAction(string action)
  {
    var sites = FindObjectsByType<AbstractDreamscapeSite>(
      FindObjectsInactive.Exclude,
      FindObjectsSortMode.None
    );
    for (var i = 0; i < sites.Length; i++)
    {
      var site = sites[i];
      if (site != null && site.DebugClickAction == action)
      {
        return site;
      }
    }
    return null;
  }

  void DeactivateSite(string action)
  {
    var site = FindSiteForAction(action);
    if (site == null)
    {
      throw new InvalidOperationException($"No site found for action {action}");
    }
    site.SetActiveWithoutFocus(false);
  }

  IEnumerator FocusSiteFlow(string action, Func<IEnumerator>? prepare, Func<IEnumerator>? onFocused)
  {
    EnsureFlowsInitialized();
    var site = FindSiteForAction(action);
    if (site == null)
    {
      throw new InvalidOperationException($"No site found for action {action}");
    }
    if (_mapCamera == null)
    {
      throw new InvalidOperationException("Map camera is not assigned.");
    }
    if (prepare != null)
    {
      var routine = prepare();
      if (routine == null)
      {
        throw new InvalidOperationException($"Prepare routine missing for action {action}");
      }
      yield return StartCoroutine(routine);
    }
    yield return StartCoroutine(_mapCamera.FocusSite(site));
    if (onFocused != null)
    {
      var routine = onFocused();
      if (routine == null)
      {
        throw new InvalidOperationException($"OnFocused routine missing for action {action}");
      }
      yield return StartCoroutine(routine);
    }
  }

  IEnumerator ApplyBattleStartupRoutine()
  {
    var registry = Registry;
    var layout = registry.BattleLayout;
    var cameraPosition = layout.CameraPosition;
    registry.MainCamera.transform.SetPositionAndRotation(
      cameraPosition.position,
      cameraPosition.rotation
    );
    registry._cameraAdjuster.AdjustFieldOfView(layout.BattleCameraBounds);
    var active = true;
    layout.UserStatusDisplay.TotalSpark.gameObject.SetActive(active);
    layout.UserStatusDisplay.gameObject.SetActive(active);
    layout.EnemyStatusDisplay.TotalSpark.gameObject.SetActive(active);
    layout.EnemyStatusDisplay.gameObject.SetActive(active);
    layout.PrimaryActionButton.gameObject.SetActive(active);
    layout.SecondaryActionButton.gameObject.SetActive(active);
    layout.IncrementActionButton.gameObject.SetActive(active);
    layout.DecrementActionButton.gameObject.SetActive(active);
    var browserButtons = layout.GetComponentsInChildren<CardBrowserButton>();
    for (var i = 0; i < browserButtons.Length; i++)
    {
      browserButtons[i].gameObject.SetActive(active);
    }
    if (!_battleStartupApplied)
    {
      registry.ActionService.TriggerReconnect();
      _battleStartupApplied = true;
    }
    yield break;
  }

  IEnumerator ReturnToMap()
  {
    if (_mapCamera == null)
    {
      throw new InvalidOperationException("Map camera is not assigned.");
    }
    _mapCamera.ActivateWithTransition();
    while (_mapCamera.IsTransitioning)
    {
      yield return null;
    }
  }

  IEnumerator BrowseQuestDeck()
  {
    var questDeckLayout = Registry.DreamscapeLayout.QuestDeck;
    var cardCount = questDeckLayout.Objects.Count;
    if (cardCount == 0)
    {
      yield break;
    }

    if (_mapCamera != null)
    {
      _mapCamera.HideSiteButtons();
    }

    var request = new CreateOrUpdateCardsRequest
    {
      Count = cardCount,
      Position = new ObjectPosition
      {
        Position = new Position { Enum = PositionEnum.QuestDeckBrowser },
        SortingKey = 0,
      },
      Revealed = true,
      GroupKey = "quest",
    };

    _prototypeCards.CreateOrUpdateCards(request);
    yield return StartCoroutine(CreateOrUpdateCards(request, animate: true));

    Registry.DreamscapeLayout.QuestDeckBrowser.SetCloseButtonAction(
      new GameAction
      {
        GameActionClass = new GameActionClass
        {
          DebugAction = new DebugAction
          {
            DebugActionClass = new DebugActionClass { ApplyTestScenarioAction = "closeQuestDeck" },
          },
        },
      }
    );
  }

  IEnumerator CloseQuestDeck()
  {
    var browserLayout = Registry.DreamscapeLayout.QuestDeckBrowser;
    var cardCount = browserLayout.Objects.Count;
    if (cardCount == 0)
    {
      yield break;
    }

    browserLayout.SetCloseButtonAction(null);

    var request = new CreateOrUpdateCardsRequest
    {
      Count = cardCount,
      Position = new ObjectPosition
      {
        Position = new Position { Enum = PositionEnum.QuestDeck },
        SortingKey = 1,
      },
      Revealed = false,
      GroupKey = "quest",
    };

    _prototypeCards.CreateOrUpdateCards(request);
    yield return StartCoroutine(CreateOrUpdateCards(request, animate: true));

    if (_mapCamera != null)
    {
      _mapCamera.ShowSiteButtons();
    }
  }

  IEnumerator CreateOrUpdateCards(CreateOrUpdateCardsRequest request, bool animate = true)
  {
    var cards = _prototypeCards.CreateOrUpdateCards(request);
    var quest = new QuestView { Cards = cards, EssenceTotal = 75 };
    var temptingOffer = _temptingOfferFlow.BuildTemptingOfferView(request);
    if (temptingOffer != null)
    {
      quest.TemptingOffer = temptingOffer;
    }
    var command = new UpdateQuestCommand { Quest = quest };

    var coroutines = new List<Coroutine>();
    Registry.DreamscapeService.HandleUpdateQuestCommand(command, coroutines, animate);
    foreach (var coroutine in coroutines)
    {
      yield return coroutine;
    }
  }

  void ApplyPendingShopOverrides()
  {
    if (!_hasPendingShopOverridesUpdate || _shopFlow == null)
    {
      return;
    }
    if (_pendingShopOverrides != null)
    {
      _shopFlow.ConfigureShopOverrides(_pendingShopOverrides.ToArray());
    }
    else
    {
      _shopFlow.ClearShopOverrides();
    }
    _hasPendingShopOverridesUpdate = false;
  }
}
