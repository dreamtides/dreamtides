using System.Collections.Generic;
using Dreamtides.Buttons;
using HighlightPlus;
using Unity.Cinemachine;
using UnityEngine;

public class CameraMover : MonoBehaviour
{
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
        ResetPrioritiesAndTrack(_draftTrackingTarget, false);
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

    void ResetPrioritiesAndTrack(Transform track, bool showSiteButtons)
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

        // Defer showing site buttons (if requested) until after the blend/transition completes
        if (showSiteButtons)
        {
            _siteButtonsActivationCoroutine = StartCoroutine(WaitForTransitionThenShowSiteButtons());
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

    System.Collections.IEnumerator WaitForTransitionThenShowSiteButtons()
    {
        yield return null;
        if (_brain != null)
        {
            while (_brain.IsBlending)
            {
                yield return null;
            }
        }

        SetSiteButtonsActive(true);
        _siteButtonsActivationCoroutine = null;
    }
}
