#nullable enable

using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using Dreamtides.Buttons;
using Dreamtides.Prototype;
using Dreamtides.Schema;
using Dreamtides.Services;
using HighlightPlus;
using Unity.Cinemachine;
using UnityEngine;

public class PrototypeQuest : Service
{
  [SerializeField]
  string _outlineColorHex = "#EF6C00";

  [SerializeField]
  CinemachineBrain _brain = null!;

  [SerializeField]
  CinemachineCamera _spaceCameraFar = null!;

  [SerializeField]
  CinemachineCamera _spaceCameraNear = null!;

  [SerializeField]
  CinemachineCamera _mapCamera = null!;

  [SerializeField]
  CinemachineCamera _draftCamera = null!;

  [SerializeField]
  Transform _draftTrackingTarget = null!;

  [SerializeField]
  CinemachineCamera _shopCamera = null!;

  [SerializeField]
  Transform _shopTrackingTarget = null!;

  [SerializeField]
  CinemachineCamera _eventCamera = null!;

  [SerializeField]
  Transform _eventTrackingTarget = null!;

  [SerializeField]
  CinemachineCamera _essenceCamera = null!;

  [SerializeField]
  Transform _essenceTrackingTarget = null!;

  [SerializeField]
  CinemachineCamera _draft2Camera = null!;

  [SerializeField]
  Transform _draft2TrackingTarget = null!;

  [SerializeField]
  CinemachineCamera _battleCamera = null!;

  [SerializeField]
  Transform _battleTrackingTarget = null!;

  [SerializeField]
  List<HighlightEffect> _highlightEffects = null!;

  [SerializeField]
  List<SiteButton> _siteButtons = null!;

  public static readonly Guid DraftSiteId = Guid.NewGuid();
  public static readonly Guid ShopSiteId = Guid.NewGuid();
  public static readonly Guid TemptingOfferSiteId = Guid.NewGuid();

  Coroutine? _siteButtonsActivationCoroutine;
  PrototypeCards _prototypeCards = new PrototypeCards();
  PrototypeQuestDraftFlow _draftFlow = null!;
  PrototypeQuestShopFlow _shopFlow = null!;
  PrototypeQuestTemptingOfferFlow _temptingOfferFlow = null!;
  List<CardOverride>? _pendingShopOverrides;
  bool _hasPendingShopOverridesUpdate;

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

  void Awake()
  {
    Application.targetFrameRate = 60;
    if (_brain == null && Camera.main != null)
    {
      _brain = Camera.main.GetComponent<CinemachineBrain>();
    }
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
        FocusMapCamera,
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

    if (name == "closeShop")
    {
      EnsureFlowsInitialized();
      _shopFlow.ClearDisplayedCards();
      FocusMapCamera();
      Registry.DreamscapeService.HideCloseSiteButton();
      Registry.DocumentService.RenderScreenAnchoredNode(
        new AnchorToScreenPositionCommand() { Node = null }
      );

      return;
    }

    if (name == "closeTemptingOffer")
    {
      FocusMapCamera();
      Registry.DreamscapeService.HideCloseSiteButton();
      Registry.DreamscapeLayout.TemptingOfferDisplay.HideAcceptButtons();
      Registry.DocumentService.RenderScreenAnchoredNode(
        new AnchorToScreenPositionCommand() { Node = null }
      );

      return;
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
      return;
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

    SetSiteButtonsActive(false);

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

    var allCards = _prototypeCards.CreateOrUpdateCards(request);
    var questCards = allCards.Take(cardCount).ToList();

    // var animation = new MoveCardsWithCustomAnimationCommand
    // {
    //   Animation = MoveCardsCustomAnimation.OpenQuestDeckBrowser,
    //   Cards = questCards,
    //   Destination = new Position { Enum = PositionEnum.QuestDeckBrowser },
    //   PauseDuration = new Milliseconds { MillisecondsValue = 0 },
    //   StaggerInterval = new Milliseconds { MillisecondsValue = 0 },
    // };

    // yield return Registry.CardAnimationService.HandleMoveCardsWithCustomAnimation(animation);

    yield return StartCoroutine(CreateOrUpdateCards(request, animate: true));
  }

  public void FocusSpaceCameraFar()
  {
    ResetPrioritiesAndTrack(null, false);
    _spaceCameraFar.Priority = 10;
  }

  public void FocusSpaceCameraNear()
  {
    ResetPrioritiesAndTrack(null, false);
    _spaceCameraNear.Priority = 10;
  }

  public void FocusMapCamera()
  {
    ResetPrioritiesAndTrack(null, true);
    _mapCamera.Priority = 10;
  }

  public void FocusDraftCamera()
  {
    EnsureFlowsInitialized();
    StartCoroutine(_draftFlow.PrepareDraftDeck());
    ResetPrioritiesAndTrack(
      _draftTrackingTarget,
      false,
      () =>
      {
        StartCoroutine(_draftFlow.RunDraftPickSequence());
      }
    );
    _draftCamera.Priority = 10;
  }

  public void FocusShopCamera()
  {
    EnsureFlowsInitialized();
    StartCoroutine(_shopFlow.PrepareShopCards());
    ResetPrioritiesAndTrack(
      _shopTrackingTarget,
      false,
      () =>
      {
        StartCoroutine(_shopFlow.RunShopDisplaySequence());
      }
    );

    _shopCamera.Priority = 10;
  }

  public void FocusEventCamera()
  {
    EnsureFlowsInitialized();
    StartCoroutine(_temptingOfferFlow.PrepareTemptingOfferCards());
    ResetPrioritiesAndTrack(
      _eventTrackingTarget,
      false,
      () =>
      {
        StartCoroutine(_temptingOfferFlow.ShowTemptingOfferCards());
      }
    );
    _eventCamera.Priority = 10;
  }

  public void FocusEssenceCamera()
  {
    ResetPrioritiesAndTrack(_essenceTrackingTarget, false);
    _essenceCamera.Priority = 10;
  }

  public void FocusDraft2Camera()
  {
    ResetPrioritiesAndTrack(_draft2TrackingTarget, false);
    _draft2Camera.Priority = 10;
  }

  public void FocusBattleCamera()
  {
    ResetPrioritiesAndTrack(_battleTrackingTarget, false);
    _battleCamera.Priority = 10;
  }

  void ResetPrioritiesAndTrack(
    Transform? track,
    bool showSiteButtons,
    Action? onCameraMoveFinished = null
  )
  {
    // Cancel any pending site-button activation and hide them immediately
    if (_siteButtonsActivationCoroutine != null)
    {
      StopCoroutine(_siteButtonsActivationCoroutine);
      _siteButtonsActivationCoroutine = null;
    }

    _spaceCameraFar.Priority = 0;
    _spaceCameraNear.Priority = 0;
    _mapCamera.Priority = 0;
    _draftCamera.Priority = 0;
    _shopCamera.Priority = 0;
    _eventCamera.Priority = 0;
    _essenceCamera.Priority = 0;
    _draft2Camera.Priority = 0;
    _battleCamera.Priority = 0;

    SetSiteButtonsActive(false);

    if (track)
    {
      _spaceCameraFar.Target.TrackingTarget = track;
      _spaceCameraNear.Target.TrackingTarget = track;
      _mapCamera.Target.TrackingTarget = track;
      _draftCamera.Target.TrackingTarget = track;
      _shopCamera.Target.TrackingTarget = track;
      _eventCamera.Target.TrackingTarget = track;
      _essenceCamera.Target.TrackingTarget = track;
      _draft2Camera.Target.TrackingTarget = track;
      _battleCamera.Target.TrackingTarget = track;
    }

    // Defer site button display and/or completion callback until after the blend/transition completes
    if (showSiteButtons || onCameraMoveFinished != null)
    {
      _siteButtonsActivationCoroutine = StartCoroutine(
        WaitForTransitionThen(() =>
        {
          if (showSiteButtons)
          {
            SetSiteButtonsActive(true);
          }
          onCameraMoveFinished?.Invoke();
        })
      );
    }
  }

  void SetSiteButtonsActive(bool active)
  {
    if (_siteButtons == null)
      return;
    foreach (var button in _siteButtons)
    {
      if (button != null)
      {
        button.gameObject.SetActive(active);
      }
    }

    foreach (var effect in _highlightEffects)
    {
      if (effect != null)
      {
        effect.highlighted = active;
      }
    }
  }

  IEnumerator WaitForTransitionThen(Action afterBlend)
  {
    // Wait a frame so Cinemachine can start the blend
    yield return null;
    if (_brain != null)
    {
      while (_brain.IsBlending)
      {
        yield return null;
      }
    }
    afterBlend?.Invoke();
    _siteButtonsActivationCoroutine = null;
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
