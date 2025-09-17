#nullable enable

using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using Dreamtides.Buttons;
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

  void Awake()
  {
    Application.targetFrameRate = 60;
    if (_brain == null && Camera.main != null)
    {
      _brain = Camera.main.GetComponent<CinemachineBrain>();
    }
  }

  protected override void OnInitialize(TestConfiguration? testConfiguration)
  {
    StartCoroutine(
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
        }
      )
    );
  }

  public void OnDebugScenarioAction(string name)
  {
    if (string.IsNullOrEmpty(name))
      return;
    var parts = name.Split('/');
    if (parts.Length != 2)
      return;
    if (parts[0] != "draft-pick")
      return;
    var clickedId = parts[1];
    if (_currentDraftPickIds == null || _currentDraftPickIds.Count != 4)
      return;
    if (!_currentDraftPickIds.Contains(clickedId))
      return;

    StartCoroutine(ResolveDraftPick(clickedId));
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
    ResetPrioritiesAndTrack(_shopTrackingTarget, false);
    _shopCamera.Priority = 10;
  }

  public void FocusEventCamera()
  {
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

  IEnumerator CreateOrUpdateCards(CreateOrUpdateCardsRequest request)
  {
    var cards = _prototypeCards.CreateOrUpdateCards(request);
    var command = new UpdateQuestCommand { Quest = new QuestView { Cards = cards } };

    var sequence = TweenUtils.Sequence("UpdateQuest");
    return Registry.CardService.HandleUpdateQuestCommand(command, sequence);
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
}
