#nullable enable

using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using Dreamtides.Buttons;
using Dreamtides.Masonry;
using Dreamtides.Prototype;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.Utils;
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

  Coroutine? _siteButtonsActivationCoroutine;
  PrototypeCards _prototypeCards = new PrototypeCards();
  List<string> _currentDraftPickIds = new List<string>(4);
  List<string> _currentShopDisplayIds = new List<string>(8);

  // Optional per-card overrides when spawning shop cards
  List<CardOverride>? _shopOverrides;

  // Public API to configure arbitrary shop card overrides (index-based)
  public void ConfigureShopOverrides(params CardOverride[] overrides)
  {
    _shopOverrides = overrides?.ToList();
  }

  public void ClearShopOverrides()
  {
    _shopOverrides = null;
  }

  void Awake()
  {
    Application.targetFrameRate = 60;
    if (_brain == null && Camera.main != null)
    {
      _brain = Camera.main.GetComponent<CinemachineBrain>();
    }
  }

  protected override void OnInitialize(GameMode _mode, TestConfiguration? testConfiguration)
  {
    StartCoroutine(InitializeQuestSequence());
  }

  IEnumerator InitializeQuestSequence()
  {
    yield return StartCoroutine(
      CreateOrUpdateCards(
        new CreateOrUpdateCardsRequest
        {
          Count = 20,
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

    if (name == "closeShop")
    {
      _currentShopDisplayIds.Clear();
      FocusMapCamera();
      Registry.DreamscapeService.HideShop();
      StartCoroutine(HideShopSequence());
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
    if (action == "draft-pick")
    {
      if (_currentDraftPickIds == null || _currentDraftPickIds.Count != 4)
      {
        return;
      }
      if (!_currentDraftPickIds.Contains(clickedId))
      {
        return;
      }
      StartCoroutine(ResolveDraftPick(clickedId));
      return;
    }
    if (action == "shop-pick")
    {
      if (_currentShopDisplayIds == null || !_currentShopDisplayIds.Contains(clickedId))
      {
        return;
      }
      StartCoroutine(ResolveShopPick(clickedId));
      return;
    }
  }

  IEnumerator ResolveDraftPick(string clickedId)
  {
    var cardsForAnimation = new List<CardView>(4);
    foreach (var id in _currentDraftPickIds)
    {
      var card = Registry.CardService.GetCard(id);
      var source = card.CardView;
      if (id == clickedId)
      {
        var sorting = Registry.DreamscapeLayout.QuestDeck.Objects.Count;
        cardsForAnimation.Add(
          CloneCardViewWithPosition(source, new Position { Enum = PositionEnum.QuestDeck }, sorting)
        );
      }
      else
      {
        cardsForAnimation.Add(source);
      }
    }

    var command = new MoveCardsWithCustomAnimationCommand
    {
      Animation = MoveCardsCustomAnimation.MoveToQuestDeckOrDestroy,
      Cards = cardsForAnimation,
      Destination = new Position { Enum = PositionEnum.QuestDeck },
      PauseDuration = new Milliseconds { MillisecondsValue = 300 },
      StaggerInterval = new Milliseconds { MillisecondsValue = 100 },
    };

    yield return StartCoroutine(
      Registry.CardAnimationService.HandleMoveCardsWithCustomAnimation(command)
    );

    var allIds = Registry.CardService.GetCardIds().ToList();
    var updateCards = new List<CardView>(allIds.Count);
    foreach (var id in allIds)
    {
      var card = Registry.CardService.GetCard(id);
      var source = card.CardView;
      if (id == clickedId)
      {
        var sorting = Registry.DreamscapeLayout.QuestDeck.Objects.Count;
        updateCards.Add(
          CloneCardViewWithPositionHidden(
            source,
            new Position { Enum = PositionEnum.QuestDeck },
            sorting
          )
        );
      }
      else if (_currentDraftPickIds.Contains(id))
      {
        updateCards.Add(
          CloneCardViewWithPosition(source, new Position { Enum = PositionEnum.Offscreen }, 0)
        );
      }
      else
      {
        updateCards.Add(source);
      }
    }

    var update = new UpdateQuestCommand { Quest = new QuestView { Cards = updateCards } };
    yield return Registry.CardService.HandleUpdateQuestCommand(update);

    // Keep the prototype's draft cache in sync so old picks don't snap back into the draft layout
    _prototypeCards.UpdateGroupCards("draft", updateCards);

    _prototypeCards.AdvanceGroupWindow("draft", 4);

    var remaining = 0;
    foreach (var cv in updateCards)
    {
      if (cv.Position.Position.PositionClass?.SiteDeck != null)
      {
        remaining++;
      }
    }

    if (remaining >= 4)
    {
      yield return StartCoroutine(RunDraftPickSequence());
    }
    else
    {
      // Draft deck exhausted; return focus to map and clear pick state
      _currentDraftPickIds.Clear();
      FocusMapCamera();
    }
  }

  IEnumerator ResolveShopPick(string clickedId)
  {
    var cardsForAnimation = new List<CardView>(1);
    var card = Registry.CardService.GetCard(clickedId);
    var source = card.CardView;
    var isRestock = source.Revealed != null && source.Revealed.CardType == "Restock";
    if (isRestock)
    {
      yield return StartCoroutine(RestockShopSequence());
      yield break;
    }
    var isDreamsign = source.Prefab == CardPrefab.Dreamsign;
    if (isDreamsign)
    {
      var sorting = Registry.DreamscapeLayout.DreamsignDisplay.Objects.Count;
      cardsForAnimation.Add(
        CloneCardViewWithPosition(
          source,
          new Position { Enum = PositionEnum.DreamsignDisplay },
          sorting
        )
      );
    }
    else
    {
      var sorting = Registry.DreamscapeLayout.QuestDeck.Objects.Count;
      cardsForAnimation.Add(
        CloneCardViewWithPosition(source, new Position { Enum = PositionEnum.QuestDeck }, sorting)
      );
    }

    var command = new MoveCardsWithCustomAnimationCommand
    {
      Animation = isDreamsign
        ? MoveCardsCustomAnimation.MoveToDreamsignDisplayOrDestroy
        : MoveCardsCustomAnimation.MoveToQuestDeckOrDestroy,
      Cards = cardsForAnimation,
      Destination = new Position
      {
        Enum = isDreamsign ? PositionEnum.DreamsignDisplay : PositionEnum.QuestDeck,
      },
      PauseDuration = new Milliseconds { MillisecondsValue = 300 },
      StaggerInterval = new Milliseconds { MillisecondsValue = 100 },
      CardTrail = new ProjectileAddress
      {
        Projectile =
          "Assets/ThirdParty/Hovl Studio/AAA Projectiles Vol 1/Prefabs/Dreamtides/Projectile 26 blue diamond.prefab",
      },
    };

    yield return StartCoroutine(
      Registry.CardAnimationService.HandleMoveCardsWithCustomAnimation(command)
    );

    var allIds = Registry.CardService.GetCardIds().ToList();
    var updateCards = new List<CardView>(allIds.Count);
    foreach (var id in allIds)
    {
      var c = Registry.CardService.GetCard(id);
      var s = c.CardView;
      if (id == clickedId)
      {
        if (isDreamsign)
        {
          var srt = Registry.DreamscapeLayout.DreamsignDisplay.Objects.Count;
          var cv = CloneCardViewWithPosition(
            s,
            new Position { Enum = PositionEnum.DreamsignDisplay },
            srt
          );
          if (cv.Revealed != null && cv.Revealed.Actions != null)
          {
            cv.Revealed.Actions.ButtonAttachment = null;
          }
          updateCards.Add(cv);
        }
        else
        {
          var srt = Registry.DreamscapeLayout.QuestDeck.Objects.Count;
          updateCards.Add(
            CloneCardViewWithPositionHidden(s, new Position { Enum = PositionEnum.QuestDeck }, srt)
          );
        }
      }
      else
      {
        updateCards.Add(s);
      }
    }

    var update = new UpdateQuestCommand { Quest = new QuestView { Cards = updateCards } };
    var sequence = TweenUtils.Sequence("UpdateQuest");
    yield return Registry.CardService.HandleUpdateQuestCommand(update, sequence);

    _prototypeCards.UpdateGroupCards("shop", updateCards);
    _currentShopDisplayIds.Remove(clickedId);
  }

  IEnumerator RestockShopSequence()
  {
    yield return StartCoroutine(HideShopSequence());

    var allIds = Registry.CardService.GetCardIds().ToList();
    var updateCards = new List<CardView>(allIds.Count);
    foreach (var id in allIds)
    {
      var c = Registry.CardService.GetCard(id);
      var s = c.CardView;
      if (_currentShopDisplayIds.Contains(id))
      {
        updateCards.Add(
          CloneCardViewWithPositionHidden(s, new Position { Enum = PositionEnum.Offscreen }, 0)
        );
      }
      else
      {
        updateCards.Add(s);
      }
    }

    var update = new UpdateQuestCommand { Quest = new QuestView { Cards = updateCards } };
    yield return Registry.CardService.HandleUpdateQuestCommand(update);

    _prototypeCards.UpdateGroupCards("shop", updateCards);
    _prototypeCards.AdvanceGroupWindow("shop", 6);

    yield return StartCoroutine(RunShopDisplaySequence());
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
    StartCoroutine(
      CreateOrUpdateCards(
        new CreateOrUpdateCardsRequest
        {
          Count = 20,
          Position = new ObjectPosition
          {
            Position = new Position
            {
              PositionClass = new PositionClass { SiteDeck = Guid.NewGuid() },
            },
            SortingKey = 1,
          },
          Revealed = false,
          GroupKey = "draft",
        }
      )
    );

    ResetPrioritiesAndTrack(
      _draftTrackingTarget,
      false,
      () =>
      {
        StartCoroutine(RunDraftPickSequence());
      }
    );
    _draftCamera.Priority = 10;
  }

  public void FocusShopCamera()
  {
    // Reset the shop cache so we create fresh cards and can apply Dreamsign overrides deterministically
    _prototypeCards.ResetGroup("shop");

    if (_shopOverrides == null)
    {
      _shopOverrides = new List<CardOverride>();
    }
    // Remove any prior override for index 4 then add our image override
    _shopOverrides.RemoveAll(o => o.Index == 4);
    _shopOverrides.Add(
      new CardOverride { Index = 0, ButtonAttachmentLabel = "550<voffset=-0.17em>\ufcec</voffset>" }
    );
    _shopOverrides.Add(
      new CardOverride { Index = 1, ButtonAttachmentLabel = "200<voffset=-0.17em>\ufcec</voffset>" }
    );
    _shopOverrides.Add(
      new CardOverride { Index = 2, ButtonAttachmentLabel = "300<voffset=-0.17em>\ufcec</voffset>" }
    );
    _shopOverrides.Add(
      new CardOverride
      {
        Index = 3,
        Name = "Restock",
        SpritePath = "Assets/ThirdParty/GameAssets/Icons/outline_refresh-cw.png",
        Rules = "Restock",
        Prefab = CardPrefab.IconCard,
        CardType = "Restock",
        ButtonAttachmentLabel = "100<voffset=-0.17em>\ufcec</voffset>",
      }
    );
    _shopOverrides.Add(
      new CardOverride
      {
        Index = 4,
        SpritePath =
          "Assets/ThirdParty/AngelinaAvgustova/WitchCraftIcons/PNG/outline_hourglass.png",
        Prefab = CardPrefab.Dreamsign,
        ButtonAttachmentLabel = "200<voffset=-0.17em>\ufcec</voffset>",
      }
    );
    _shopOverrides.Add(
      new CardOverride
      {
        Index = 5,
        SpritePath =
          "Assets/ThirdParty/AngelinaAvgustova/WitchCraftIcons/PNG/outline_goldfeather.png",
        Prefab = CardPrefab.Dreamsign,
        ButtonAttachmentLabel = "600<voffset=-0.17em>\ufcec</voffset>",
      }
    );

    StartCoroutine(
      CreateOrUpdateCards(
        new CreateOrUpdateCardsRequest
        {
          Count = 6,
          Position = new ObjectPosition
          {
            Position = new Position
            {
              PositionClass = new PositionClass { SiteNpc = Guid.NewGuid() },
            },
            SortingKey = 1,
          },
          Revealed = true,
          GroupKey = "shop",
          Overrides = _shopOverrides,
          ButtonAttachmentDebugScenario = "shop-pick",
        }
      )
    );

    ResetPrioritiesAndTrack(
      _shopTrackingTarget,
      false,
      () =>
      {
        StartCoroutine(RunShopDisplaySequence());
      }
    );

    _shopCamera.Priority = 10;
  }

  public void FocusEventCamera()
  {
    StartCoroutine(ShowTemptingOfferCards());
    ResetPrioritiesAndTrack(_eventTrackingTarget, false);
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
    var command = new UpdateQuestCommand { Quest = new QuestView { Cards = cards } };

    var sequence = TweenUtils.Sequence("UpdateQuest");
    return Registry.CardService.HandleUpdateQuestCommand(command, animate ? sequence : null);
  }

  IEnumerator RunDraftPickSequence()
  {
    var allCards = _prototypeCards.CreateOrUpdateCards(
      new CreateOrUpdateCardsRequest
      {
        Count = 4,
        Position = new ObjectPosition
        {
          Position = new Position { Enum = PositionEnum.DraftPickDisplay },
          SortingKey = 1,
        },
        Revealed = true,
        OutlineColorHex = _outlineColorHex,
        GroupKey = "draft",
        OnClickDebugScenario = "draft-pick",
      }
    );
    _currentDraftPickIds = allCards.Take(4).Select(cv => cv.Id).ToList();

    var customAnimation = new MoveCardsWithCustomAnimationCommand
    {
      Animation = MoveCardsCustomAnimation.ShowInDraftPickLayout,
      Cards = allCards.Take(4).ToList(),
      Destination = new Position { Enum = PositionEnum.DraftPickDisplay },
      PauseDuration = new Milliseconds { MillisecondsValue = 0 },
      StaggerInterval = new Milliseconds { MillisecondsValue = 300 },
    };

    yield return StartCoroutine(
      Registry.CardAnimationService.HandleMoveCardsWithCustomAnimation(customAnimation)
    );

    yield return StartCoroutine(
      CreateOrUpdateCards(
        new CreateOrUpdateCardsRequest
        {
          Count = 4,
          Position = new ObjectPosition
          {
            Position = new Position { Enum = PositionEnum.DraftPickDisplay },
            SortingKey = 1,
          },
          Revealed = true,
          OutlineColorHex = _outlineColorHex,
          GroupKey = "draft",
          OnClickDebugScenario = "draft-pick",
        }
      )
    );
  }

  IEnumerator RunShopDisplaySequence()
  {
    // Ensure the custom image override for card #4 persists during display
    const string hourglassPath =
      "Assets/ThirdParty/AngelinaAvgustova/WitchCraftIcons/PNG/outline_hourglass.png";
    if (_shopOverrides == null)
    {
      _shopOverrides = new List<CardOverride>();
    }
    var idx4 = _shopOverrides.FindIndex(o => o.Index == 4);
    if (idx4 >= 0)
    {
      _shopOverrides[idx4].SpritePath = hourglassPath;
    }
    else
    {
      _shopOverrides.Add(
        new CardOverride
        {
          Index = 4,
          SpritePath = hourglassPath,
          ButtonAttachmentLabel = "200<voffset=-0.17em>\ufcec</voffset>",
        }
      );
    }
    var allCards = _prototypeCards.CreateOrUpdateCards(
      new CreateOrUpdateCardsRequest
      {
        Count = 6,
        Position = new ObjectPosition
        {
          Position = new Position { Enum = PositionEnum.ShopDisplay },
          SortingKey = 1,
        },
        Revealed = true,
        GroupKey = "shop",
        Overrides = _shopOverrides,
        ButtonAttachmentLabel = null,
        ButtonAttachmentDebugScenario = "shop-pick",
      }
    );
    _currentShopDisplayIds = allCards.Take(6).Select(cv => cv.Id).ToList();

    yield return new WaitForSeconds(0.3f);

    var customAnimation = new MoveCardsWithCustomAnimationCommand
    {
      Animation = MoveCardsCustomAnimation.ShowInShopLayout,
      Cards = allCards.Take(6).ToList(),
      Destination = new Position { Enum = PositionEnum.ShopDisplay },
      PauseDuration = new Milliseconds { MillisecondsValue = 0 },
      StaggerInterval = new Milliseconds { MillisecondsValue = 50 },
    };

    StartCoroutine(
      Registry.DreamscapeService.HandlePlayMecanimAnimation(
        new PlayMecanimAnimationCommand
        {
          SiteId = Guid.NewGuid(),
          Parameters = new List<MecanimParameter>
          {
            new MecanimParameter { TriggerParam = new TriggerParam { Name = "WaveSmall" } },
          },
        }
      )
    );

    yield return StartCoroutine(
      Registry.CardAnimationService.HandleMoveCardsWithCustomAnimation(customAnimation)
    );

    Registry.DocumentService.RenderScreenAnchoredNode(
      new AnchorToScreenPositionCommand()
      {
        Node = ShopMerchantDialog(),
        Anchor = new ScreenAnchor { SiteCharacter = Guid.NewGuid() },
        ShowDuration = new Milliseconds { MillisecondsValue = 5000 },
      }
    );

    yield return StartCoroutine(
      CreateOrUpdateCards(
        new CreateOrUpdateCardsRequest
        {
          Count = 6,
          Position = new ObjectPosition
          {
            Position = new Position { Enum = PositionEnum.ShopDisplay },
            SortingKey = 1,
          },
          Revealed = true,
          GroupKey = "shop",
          Overrides = _shopOverrides,
          ButtonAttachmentDebugScenario = "shop-pick",
        }
      )
    );

    var button = Registry.DreamscapeService.CloseButton.GetComponent<CloseBrowserButton>();
    button.CloseAction = new GameAction
    {
      GameActionClass = new GameActionClass
      {
        DebugAction = new DebugAction
        {
          DebugActionClass = new DebugActionClass { ApplyTestScenarioAction = $"closeShop" },
        },
      },
    };
  }

  IEnumerator ShowTemptingOfferCards()
  {
    const string groupKey = "tempting-offer";
    const string spritePath =
      "Assets/ThirdParty/GameAssets/CardImages/Circular/shutterstock_1486924805.png";
    _prototypeCards.ResetGroup(groupKey);
    var cards = _prototypeCards.CreateOrUpdateCards(
      new CreateOrUpdateCardsRequest
      {
        Count = 4,
        Position = new ObjectPosition
        {
          Position = new Position
          {
            PositionClass = new PositionClass
            {
              TemptingOfferDisplay = BuildTemptingOfferPosition(0, TemptingOfferType.Cost),
            },
          },
          SortingKey = 1,
        },
        Revealed = true,
        GroupKey = groupKey,
        Overrides = BuildTemptingOfferOverrides(spritePath),
      }
    );
    ApplyTemptingOfferPresentation(cards, groupKey, spritePath);
    var command = new UpdateQuestCommand { Quest = new QuestView { Cards = cards } };
    yield return Registry.CardService.HandleUpdateQuestCommand(command);
  }

  static List<CardOverride> BuildTemptingOfferOverrides(string spritePath)
  {
    var overrides = new List<CardOverride>();
    overrides.Add(
      new CardOverride
      {
        Index = 0,
        Prefab = CardPrefab.Journey,
        Name = "Journey",
        Rules = string.Empty,
        CardType = string.Empty,
        IsFast = false,
        SpritePath = spritePath,
      }
    );
    overrides.Add(
      new CardOverride
      {
        Index = 1,
        Prefab = CardPrefab.OfferCost,
        Name = "Cost",
        Rules = string.Empty,
        CardType = string.Empty,
        IsFast = false,
        SpritePath = spritePath,
      }
    );
    overrides.Add(
      new CardOverride
      {
        Index = 2,
        Prefab = CardPrefab.Journey,
        Name = "Journey",
        Rules = string.Empty,
        CardType = string.Empty,
        IsFast = false,
        SpritePath = spritePath,
      }
    );
    overrides.Add(
      new CardOverride
      {
        Index = 3,
        Prefab = CardPrefab.OfferCost,
        Name = "Cost",
        Rules = string.Empty,
        CardType = string.Empty,
        IsFast = false,
        SpritePath = spritePath,
      }
    );
    return overrides;
  }

  void ApplyTemptingOfferPresentation(List<CardView> cards, string groupKey, string spritePath)
  {
    var groupCards = cards.Where(cv => cv.Id.StartsWith($"{groupKey}-")).Take(4).ToList();
    for (var i = 0; i < groupCards.Count; i++)
    {
      var type = i % 2 == 0 ? TemptingOfferType.Journey : TemptingOfferType.Cost;
      var name = type == TemptingOfferType.Cost ? "Cost" : "Journey";
      ConfigureTemptingOfferCard(groupCards[i], name, type, spritePath, i);
    }
  }

  static TemptingOfferPosition BuildTemptingOfferPosition(
    int sortingKey,
    TemptingOfferType displayType
  )
  {
    var offerNumber = sortingKey / 2;
    return new TemptingOfferPosition { Number = offerNumber, OfferType = displayType };
  }

  static void ConfigureTemptingOfferCard(
    CardView card,
    string name,
    TemptingOfferType displayType,
    string spritePath,
    int sortingKey
  )
  {
    card.Backless = true;
    card.CardFacing = CardFacing.FaceUp;
    card.Position = new ObjectPosition
    {
      Position = new Position
      {
        PositionClass = new PositionClass
        {
          TemptingOfferDisplay = BuildTemptingOfferPosition(sortingKey, displayType),
        },
      },
      SortingKey = sortingKey,
    };
    if (card.Revealed != null)
    {
      card.Revealed.Name = name;
      card.Revealed.CardType = string.Empty;
      card.Revealed.Cost = null;
      card.Revealed.Produced = null;
      card.Revealed.RulesText = string.Empty;
      card.Revealed.Spark = null;
      card.Revealed.IsFast = false;
      card.Revealed.Image = new DisplayImage { Sprite = new SpriteAddress { Sprite = spritePath } };
      card.Revealed.OutlineColor = null;
      card.Revealed.Actions = card.Revealed.Actions ?? new CardActions();
      card.Revealed.Actions.ButtonAttachment = null;
      card.Revealed.Actions.CanPlay = null;
      card.Revealed.Actions.CanSelectOrder = null;
      card.Revealed.Actions.OnClick = null;
      card.Revealed.Actions.OnPlaySound = null;
      card.Revealed.Actions.PlayEffectPreview = null;
    }
  }

  IEnumerator HideShopSequence()
  {
    // Gather current shop cards. Prefer the tracked IDs; if empty, fall back to layout contents.
    var cardsForAnimation = new List<CardView>(_currentShopDisplayIds.Count);
    if (_currentShopDisplayIds.Count > 0)
    {
      foreach (var id in _currentShopDisplayIds)
      {
        var card = Registry.CardService.GetCard(id);
        cardsForAnimation.Add(card.CardView);
      }
    }
    else
    {
      var shopObjects = Registry.DreamscapeLayout.ShopLayout.Objects;
      foreach (var displayable in shopObjects)
      {
        if (displayable is Dreamtides.Components.Card card)
        {
          cardsForAnimation.Add(card.CardView);
        }
      }
    }

    var command = new MoveCardsWithCustomAnimationCommand
    {
      Animation = MoveCardsCustomAnimation.HideShopLayout,
      Cards = cardsForAnimation,
      Destination = new Position { Enum = PositionEnum.ShopDisplay },
      PauseDuration = new Milliseconds { MillisecondsValue = 0 },
      StaggerInterval = new Milliseconds { MillisecondsValue = 50 },
    };

    yield return StartCoroutine(
      Registry.CardAnimationService.HandleMoveCardsWithCustomAnimation(command)
    );
  }

  static CardView CloneCardViewWithPosition(CardView source, Position position, int sortingKey) =>
    new CardView
    {
      Backless = source.Backless,
      CardFacing = source.CardFacing,
      CreatePosition = source.CreatePosition,
      CreateSound = source.CreateSound,
      DestroyPosition = source.DestroyPosition,
      Id = source.Id,
      Position = new ObjectPosition { Position = position, SortingKey = sortingKey },
      Prefab = source.Prefab,
      Revealed = source.Revealed,
      RevealedToOpponents = source.RevealedToOpponents,
    };

  static CardView CloneCardViewWithPositionHidden(
    CardView source,
    Position position,
    int sortingKey
  ) =>
    new CardView
    {
      Backless = source.Backless,
      CardFacing = CardFacing.FaceDown,
      CreatePosition = source.CreatePosition,
      CreateSound = source.CreateSound,
      DestroyPosition = source.DestroyPosition,
      Id = source.Id,
      Position = new ObjectPosition { Position = position, SortingKey = sortingKey },
      Prefab = source.Prefab,
      Revealed = null,
      RevealedToOpponents = false,
    };

  static FlexNode ShopMerchantDialog()
  {
    var style = new FlexStyle
    {
      BackgroundColor = Mason.MakeColor(Color.black),
      BorderRadius = Mason.AllBordersRadiusDip(4),
      Padding = Mason.AllPx(4),
      Inset = Mason.InsetPx(top: -8f, left: 6f),
      Color = Mason.MakeColor(Color.white),
      Position = FlexPosition.Absolute,
      FontSize = Mason.Px(8),
      Font = new FontAddress
      {
        Font = "Assets/ThirdParty/Fonts/Fira_Sans_Condensed/FiraSansCondensed-Regular.ttf",
      },
      TextAlign = TextAlign.MiddleLeft,
      AlignItems = FlexAlign.Center,
      JustifyContent = FlexJustify.Center,
      WhiteSpace = WhiteSpace.Normal,
      MaxWidth = Mason.Px(100),
    };

    var arrowTip = Mason.Column(
      "ArrowPoint",
      new FlexStyle
      {
        Width = Mason.Px(9),
        Height = Mason.Px(11.5f),
        Position = FlexPosition.Absolute,
        Inset = Mason.InsetPx(top: 0f, left: 0f),
        AlignItems = FlexAlign.Center,
        JustifyContent = FlexJustify.Center,
        BackgroundImage = new SpriteAddress
        {
          Sprite = "Assets/ThirdParty/GameAssets/speech_bubble_tip.png",
        },
      }
    );

    var text = Mason.TypewriterText(
      "Cooked up some nice grub for ya!",
      style,
      30,
      new AudioClipAddress
      {
        AudioClip = "Assets/ThirdParty/Cafofo/Fantasy Interface Sounds/UI Tight 01.wav",
      }
    );

    return Mason.Row(
      "ShopMerchantDialog",
      new FlexStyle { AlignItems = FlexAlign.FlexEnd, JustifyContent = FlexJustify.FlexStart },
      arrowTip,
      text
    );
  }
}
