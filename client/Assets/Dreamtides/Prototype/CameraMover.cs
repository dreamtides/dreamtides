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

public class CameraMover : MonoBehaviour
{
    [SerializeField] Registry _registry = null!;
    [SerializeField] string _outlineColorHex = "#EF6C00";
    [SerializeField] CinemachineBrain _brain;
    [SerializeField] CinemachineCamera _spaceCameraFar;
    [SerializeField] CinemachineCamera _spaceCameraNear;
    [SerializeField] CinemachineCamera _mapCamera;
    [SerializeField] CinemachineCamera _draftCamera;
    [SerializeField] Transform _draftTrackingTarget;
    [SerializeField] CinemachineCamera _shopCamera;
    [SerializeField] Transform _shopTrackingTarget;
    [SerializeField] CinemachineCamera _eventCamera;
    [SerializeField] Transform _eventTrackingTarget;
    [SerializeField] CinemachineCamera _essenceCamera;
    [SerializeField] Transform _essenceTrackingTarget;
    [SerializeField] CinemachineCamera _draft2Camera;
    [SerializeField] Transform _draft2TrackingTarget;
    [SerializeField] CinemachineCamera _battleCamera;
    [SerializeField] Transform _battleTrackingTarget;
    [SerializeField] List<HighlightEffect> _highlightEffects;
    [SerializeField] List<SiteButton> _siteButtons;

    Coroutine _siteButtonsActivationCoroutine;

    void Awake()
    {
        Application.targetFrameRate = 60;
        if (_brain == null && Camera.main != null)
        {
            _brain = Camera.main.GetComponent<CinemachineBrain>();
        }
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
        StartCoroutine(CreateOrUpdateCards(20, new ObjectPosition
        {
            Position = new Position
            {
                PositionClass = new PositionClass
                {
                    SiteDeck = Guid.NewGuid()
                }
            },
            SortingKey = 1,
        }, false));

        ResetPrioritiesAndTrack(_draftTrackingTarget, false, () =>
        {
            StartCoroutine(RunDraftPickSequence());
        });
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

    void ResetPrioritiesAndTrack(Transform track, bool showSiteButtons, Action onCameraMoveFinished = null)
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
            _siteButtonsActivationCoroutine = StartCoroutine(WaitForTransitionThen(() =>
            {
                if (showSiteButtons)
                {
                    SetSiteButtonsActive(true);
                }
                onCameraMoveFinished?.Invoke();
            }));
        }
    }

    void SetSiteButtonsActive(bool active)
    {
        if (_siteButtons == null) return;
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

    IEnumerator CreateOrUpdateCards(int count, ObjectPosition position, bool revealed, string outlineColorHex = null)
    {
        var cards = PrototypeCards.CreateOrUpdateCards(count, position, revealed, outlineColorHex);
        var command = new UpdateQuestCommand
        {
            Quest = new QuestView
            {
                Cards = cards,
            },
        };

        var sequence = TweenUtils.Sequence("UpdateQuest");
        return _registry.CardService.HandleUpdateQuestCommand(command, sequence);
    }

    IEnumerator RunDraftPickSequence()
    {
        var allCards = PrototypeCards.CreateOrUpdateCards(4, new ObjectPosition
        {
            Position = new Position
            {
                Enum = PositionEnum.DraftPickDisplay
            },
            SortingKey = 1,
        }, revealed: true, outlineColorHex: _outlineColorHex);

        var customAnimation = new MoveCardsWithCustomAnimationCommand
        {
            Animation = MoveCardsCustomAnimation.ShowInDraftPickLayout,
            Cards = allCards.Take(4).ToList(),
            Destination = new Position { Enum = PositionEnum.DraftPickDisplay },
            PauseDuration = new Milliseconds { MillisecondsValue = 0 },
            StaggerInterval = new Milliseconds { MillisecondsValue = 300 },
        };

        yield return StartCoroutine(_registry.CardAnimationService.HandleMoveCardsWithCustomAnimation(customAnimation));

        yield return StartCoroutine(CreateOrUpdateCards(4, new ObjectPosition
        {
            Position = new Position
            {
                Enum = PositionEnum.DraftPickDisplay
            },
            SortingKey = 1,
        }, revealed: true, outlineColorHex: _outlineColorHex));
    }
}
