#nullable enable

using System;
using System.Collections;
using System.Collections.Generic;
using DG.Tweening;
using Dreamtides.Buttons;
using Dreamtides.Layout;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.Utils;
using Unity.Cinemachine;
using UnityEngine;

namespace Dreamtides.Components
{
  public class DreamscapeMapCamera : Displayable
  {
    [SerializeField]
    CinemachineCamera _camera = null!;

    [SerializeField]
    float _yRotation = 0f;

    [SerializeField]
    float _mapFieldOfView = 60f;

    [SerializeField]
    float _siteFieldOfView = 60f;

    [SerializeField]
    float _mapToSiteMoveDuration = 1.2f;

    [SerializeField]
    float _mapToSiteRotationDuration = 0.6f;

    [SerializeField]
    float _siteToMapMoveDuration = 1.2f;

    [SerializeField]
    float _siteToMapRotationDuration = 0.6f;

    [SerializeField]
    float _siteToMapRotationDelay = 0.35f;

    [SerializeField]
    Ease _mapToSiteMoveEase = Ease.InOutSine;

    [SerializeField]
    Ease _siteToMapMoveEase = Ease.InOutSine;

    [SerializeField]
    Ease _rotationEase = Ease.OutQuad;

    [SerializeField]
    List<DreamscapeSite> _sites = new();

    readonly Dictionary<DreamscapeSite, CanvasButton> _siteButtonsBySite = new();
    readonly Dictionary<DreamscapeSite, Vector2> _siteButtonPositions = new();

    Sequence? _transitionSequence;
    Coroutine? _positionRoutine;
    CameraPose? _mapPose;
    DreamscapeSite? _activeSite;
    bool _siteButtonsVisible;
    bool _initialFramingComplete;
    bool _hasCachedSiteButtonPositions;

    public CinemachineCamera Camera => _camera;

    public bool IsTransitioning =>
      (_transitionSequence != null && _transitionSequence.IsActive()) || _positionRoutine != null;

    public void ActivateWithTransition()
    {
      // All of this logic basically exists to make the site <-> map camera
      // transition look slightly better. We use a two-camera setup so that the
      // camera "pulls back" to the map view while keeping the site in focus.
      // It's not really necessary, but I find it satisfying to watch.

      RefreshMapPose(applyTransform: false);
      HideSiteButtons();
      StartTransitionToMap();
    }

    public void FrameSites()
    {
      RefreshMapPose(applyTransform: true);
    }

    public IEnumerator FocusSite(DreamscapeSite site)
    {
      if (site == null)
      {
        throw new InvalidOperationException("Cannot focus a null site.");
      }
      var viewport = ResolveViewport();
      RefreshSites();
      var sitePose = site.ResolveCameraPose(viewport, _siteFieldOfView);
      RefreshMapPose(applyTransform: false);
      _activeSite = site;
      site.SetActiveWithoutFocus(true);
      HideSiteButtons();
      CancelTransition();
      var sequence = BuildMapToSiteSequence(sitePose);
      _transitionSequence = sequence;
      yield return sequence.WaitForCompletion();
      _transitionSequence = null;
    }

    public void HideSiteButtons()
    {
      if (!_siteButtonsVisible)
      {
        return;
      }
      _siteButtonsVisible = false;
      foreach (var button in _siteButtonsBySite.Values)
      {
        if (button != null)
        {
          button.gameObject.SetActive(false);
        }
      }
      Registry.DreamscapeService.CloseButton.gameObject.SetActive(false);
    }

    public void ShowSiteButtons()
    {
      if (_sites.Count == 0)
      {
        throw new InvalidOperationException("No dreamscape sites were framed.");
      }
      if (_siteButtonsVisible)
      {
        return;
      }
      _siteButtonsVisible = true;
      EnsureSiteButtons();
      PositionSiteButtons();
      foreach (var button in _siteButtonsBySite.Values)
      {
        if (button != null)
        {
          button.gameObject.SetActive(true);
        }
      }
      Registry.DreamscapeService.CloseButton.gameObject.SetActive(true);
    }

    protected override void OnInitialize()
    {
      ValidateCamera();
      SetMapCameraPriority();
    }

    protected override void OnStart()
    {
      FrameSites();
      EnsureSiteButtons();
      PositionSiteButtons();
      SchedulePositionSiteButtons();
      ShowSiteButtons();
    }

    void EnsureSiteButtons()
    {
      if (_sites.Count == 0)
      {
        throw new InvalidOperationException("No dreamscape sites were framed.");
      }

      var viewport = ResolveViewport();

      var service = Registry.DreamscapeService;
      for (var i = 0; i < _sites.Count; i++)
      {
        var site = _sites[i];
        if (site == null)
        {
          continue;
        }
        if (!_siteButtonsBySite.TryGetValue(site, out var button))
        {
          button = service.CreateSiteButton();
          _siteButtonsBySite[site] = button;
          button.Initialize(Registry, Mode, TestConfiguration);
          button.StartFromRegistry();
        }
        button.gameObject.SetActive(true);
        var action = string.IsNullOrEmpty(site.DebugClickAction)
          ? new OnClickUnion { Enum = GameActionEnum.NoOp }
          : new OnClickUnion
          {
            OnClickClass = new OnClickClass
            {
              DebugAction = new DebugAction
              {
                DebugActionClass = new DebugActionClass
                {
                  ApplyTestScenarioAction = site.DebugClickAction,
                },
              },
            },
          };
        button.SetView(new ButtonView { Action = action, Label = site.ButtonLabel });
      }
    }

    void PositionSiteButtons()
    {
      if (_sites.Count == 0)
      {
        throw new InvalidOperationException("No dreamscape sites were framed.");
      }

      var viewport = ResolveViewport();

      if (
        _hasCachedSiteButtonPositions
        && _siteButtonPositions.Count == _siteButtonsBySite.Count
        && _siteButtonPositions.Count > 0
      )
      {
        foreach (var kvp in _siteButtonPositions)
        {
          if (_siteButtonsBySite.TryGetValue(kvp.Key, out var cachedButton) && cachedButton != null)
          {
            var rect = cachedButton.GetComponent<RectTransform>();
            rect.anchoredPosition = kvp.Value;
          }
        }
        return;
      }

      EnsureSiteButtons();
      var worldPositions = new List<Vector3>(_sites.Count);
      var buttonRects = new List<RectTransform>(_sites.Count);
      var orderedSites = new List<DreamscapeSite>(_sites.Count);
      for (var i = 0; i < _sites.Count; i++)
      {
        var site = _sites[i];
        if (site == null)
        {
          continue;
        }
        if (!_siteButtonsBySite.TryGetValue(site, out var button))
        {
          continue;
        }
        worldPositions.Add(site.transform.position);
        buttonRects.Add(button.GetComponent<RectTransform>());
        orderedSites.Add(site);
      }

      if (
        worldPositions.Count == 0
        || buttonRects.Count != worldPositions.Count
        || orderedSites.Count != worldPositions.Count
      )
      {
        throw new InvalidOperationException("Unable to resolve site button positions.");
      }

      var positioner = new DreamscapeSiteButtonPositioner(viewport, Registry.CanvasSafeArea);
      var resolved = positioner.PositionButtons(worldPositions, buttonRects);
      if (!_initialFramingComplete)
      {
        return;
      }
      for (var i = 0; i < resolved.Count; i++)
      {
        _siteButtonPositions[orderedSites[i]] = resolved[i];
      }
      _hasCachedSiteButtonPositions = _siteButtonPositions.Count == orderedSites.Count;
    }

    void SchedulePositionSiteButtons()
    {
      if (!Application.isPlaying)
      {
        return;
      }
      if (_positionRoutine != null)
      {
        StopCoroutine(_positionRoutine);
      }
      _positionRoutine = StartCoroutine(PositionSiteButtonsNextFrame());
    }

    IEnumerator PositionSiteButtonsNextFrame()
    {
      yield return new WaitForEndOfFrame();
      _initialFramingComplete = true;
      PositionSiteButtons();
      _positionRoutine = null;
    }

    void ValidateCamera()
    {
      if (_camera == null)
      {
        throw new InvalidOperationException("Map camera is not assigned.");
      }
    }

    IGameViewport ResolveViewport()
    {
      var viewport = Application.isPlaying ? Registry.GameViewport : RealViewport.CreateForEditor();
      return viewport ?? throw new InvalidOperationException("Viewport is not available.");
    }

    void RefreshSites()
    {
      var discoveredSites = FindObjectsByType<DreamscapeSite>(
        FindObjectsInactive.Exclude,
        FindObjectsSortMode.None
      );
      _sites.Clear();
      for (var i = 0; i < discoveredSites.Length; i++)
      {
        var site = discoveredSites[i];
        if (site != null)
        {
          _sites.Add(site);
        }
      }
      if (_sites.Count == 0)
      {
        throw new InvalidOperationException("No dreamscape sites found in the scene.");
      }
    }

    CameraPose RefreshMapPose(bool applyTransform)
    {
      ValidateCamera();
      RefreshSites();
      var viewport = ResolveViewport();
      var pose = ComputeMapPose(viewport);
      _mapPose = pose;
      if (applyTransform)
      {
        ApplyCameraPose(pose);
      }
      PositionSiteButtons();
      return pose;
    }

    CameraPose ComputeMapPose(IGameViewport viewport)
    {
      var positions = new List<Vector3>(_sites.Count);
      for (var i = 0; i < _sites.Count; i++)
      {
        var site = _sites[i];
        if (site == null)
        {
          continue;
        }
        if (!site.gameObject.scene.isLoaded || !site.gameObject.activeInHierarchy)
        {
          continue;
        }
        positions.Add(site.transform.position);
      }
      if (positions.Count == 0)
      {
        throw new InvalidOperationException("No active dreamscape site positions were found.");
      }

      var bounds = new Bounds(positions[0], Vector3.zero);
      for (var i = 1; i < positions.Count; i++)
      {
        bounds.Encapsulate(positions[i]);
      }

      var rotation = Quaternion.Euler(50f, _yRotation, 0f);
      var tanVertical = Mathf.Tan(Mathf.Deg2Rad * 30f);
      var aspect = Mathf.Approximately(viewport.ScreenHeight, 0f)
        ? 1f
        : viewport.ScreenWidth / viewport.ScreenHeight;
      var tanHorizontal = tanVertical * aspect;
      var inverseRotation = Quaternion.Inverse(rotation);
      var requiredDistance = 0f;

      for (var i = 0; i < positions.Count; i++)
      {
        var local = inverseRotation * (positions[i] - bounds.center);
        var distanceForX = Mathf.Abs(local.x) / tanHorizontal - local.z;
        var distanceForY = Mathf.Abs(local.y) / tanVertical - local.z;
        var distanceForDepth = -local.z;
        var needed = Mathf.Max(distanceForX, distanceForY, distanceForDepth, 0f);
        requiredDistance = Mathf.Max(requiredDistance, needed);
      }

      requiredDistance = Mathf.Max(requiredDistance, 0.01f);
      var position = bounds.center - rotation * (Vector3.forward * requiredDistance);
      return new CameraPose(position, rotation, _mapFieldOfView);
    }

    void ApplyCameraPose(CameraPose pose)
    {
      _camera.transform.SetPositionAndRotation(pose.Position, pose.Rotation);
      SetFieldOfView(pose.FieldOfView);
    }

    void SetFieldOfView(float fieldOfView)
    {
      var lens = _camera.Lens;
      lens.FieldOfView = fieldOfView;
      _camera.Lens = lens;
    }

    float GetFieldOfView() => _camera.Lens.FieldOfView;

    void StartTransitionToMap()
    {
      CancelTransition();
      var targetMapPose = _mapPose ?? RefreshMapPose(applyTransform: false);
      var sequence = BuildSiteToMapSequence(targetMapPose);
      _transitionSequence = sequence;
    }

    Sequence BuildMapToSiteSequence(DreamscapeSiteCameraPose sitePose)
    {
      var duration = Mathf.Max(0.01f, _mapToSiteMoveDuration);
      var rotationDuration = Mathf.Clamp(_mapToSiteRotationDuration, 0.01f, duration);
      var sequence = TweenUtils.Sequence("MapToSite");
      sequence.Join(
        _camera.transform.DOMove(sitePose.Position, duration).SetEase(_mapToSiteMoveEase)
      );
      sequence.Join(AnimateFieldOfView(sitePose.FieldOfView, duration));
      sequence.Join(
        BuildFocusLockTween(sitePose.FocusTarget, rotationDuration, duration).SetEase(_rotationEase)
      );
      sequence.OnKill(() => _transitionSequence = null);
      sequence.OnComplete(() => _transitionSequence = null);
      return sequence;
    }

    Sequence BuildSiteToMapSequence(CameraPose mapPose)
    {
      var duration = Mathf.Max(0.01f, _siteToMapMoveDuration);
      var lockDuration = Mathf.Clamp(_siteToMapRotationDelay, 0f, duration);
      var rotationDuration = Mathf.Clamp(
        _siteToMapRotationDuration,
        0.01f,
        Mathf.Max(0.01f, duration - lockDuration)
      );
      var sequence = TweenUtils.Sequence("SiteToMap");
      sequence.Join(
        _camera.transform.DOMove(mapPose.Position, duration).SetEase(_siteToMapMoveEase)
      );
      sequence.Join(AnimateFieldOfView(mapPose.FieldOfView, duration));
      var focusTarget = _activeSite != null ? _activeSite.transform.position : (Vector3?)null;
      if (focusTarget.HasValue)
      {
        _camera.transform.rotation = BuildFocusRotation(focusTarget.Value);
        sequence.Join(
          BuildFocusThenReleaseTween(
              focusTarget.Value,
              mapPose.Rotation,
              lockDuration,
              rotationDuration,
              duration
            )
            .SetEase(_rotationEase)
        );
      }
      else
      {
        sequence.Insert(
          lockDuration,
          _camera
            .transform.DORotateQuaternion(mapPose.Rotation, rotationDuration)
            .SetEase(_rotationEase)
        );
      }
      sequence.OnKill(() => _transitionSequence = null);
      sequence.OnComplete(() =>
      {
        _transitionSequence = null;
        DeactivateAllSites();
        ShowSiteButtons();
      });
      return sequence;
    }

    Tweener AnimateFieldOfView(float fieldOfView, float duration)
    {
      return DOTween
        .To(() => GetFieldOfView(), value => SetFieldOfView(value), fieldOfView, duration)
        .SetEase(_rotationEase);
    }

    Quaternion BuildFocusRotation(Vector3 focusTarget)
    {
      var direction = focusTarget - _camera.transform.position;
      return direction.sqrMagnitude < Mathf.Epsilon
        ? _camera.transform.rotation
        : Quaternion.LookRotation(direction, Vector3.up);
    }

    void CancelTransition()
    {
      if (_transitionSequence != null)
      {
        _transitionSequence.Kill();
        _transitionSequence = null;
      }
    }

    Tweener BuildFocusLockTween(Vector3 focusTarget, float rotationDuration, float totalDuration)
    {
      var startRotation = _camera.transform.rotation;
      var duration = Mathf.Max(0.01f, totalDuration);
      var rotationSeconds = Mathf.Clamp(rotationDuration, 0.01f, duration);
      return DOTween
        .To(
          () => 0f,
          t =>
          {
            var elapsed = t * duration;
            if (elapsed < rotationSeconds)
            {
              var progress = rotationSeconds <= 0f ? 1f : Mathf.Clamp01(elapsed / rotationSeconds);
              var targetRotation = BuildFocusRotation(focusTarget);
              _camera.transform.rotation = Quaternion.Slerp(
                startRotation,
                targetRotation,
                progress
              );
              return;
            }
            LockRotationOnTarget(focusTarget);
          },
          1f,
          duration
        )
        .SetEase(Ease.Linear);
    }

    Tweener BuildFocusThenReleaseTween(
      Vector3 focusTarget,
      Quaternion mapRotation,
      float lockDuration,
      float rotationDuration,
      float totalDuration
    )
    {
      var duration = Mathf.Max(0.01f, totalDuration);
      var effectiveLockDuration = Mathf.Clamp(lockDuration, 0f, duration);
      var effectiveRotationDuration = Mathf.Clamp(
        rotationDuration,
        0.01f,
        Mathf.Max(0.01f, duration - effectiveLockDuration)
      );
      var releaseStarted = false;
      var releaseStartRotation = Quaternion.identity;
      return DOTween
        .To(
          () => 0f,
          t =>
          {
            var elapsed = t * duration;
            if (elapsed < effectiveLockDuration)
            {
              LockRotationOnTarget(focusTarget);
              return;
            }
            var rotateElapsed = elapsed - effectiveLockDuration;
            var progress = Mathf.Clamp01(rotateElapsed / effectiveRotationDuration);
            if (!releaseStarted)
            {
              releaseStarted = true;
              releaseStartRotation = BuildFocusRotation(focusTarget);
            }
            _camera.transform.rotation = Quaternion.Slerp(
              releaseStartRotation,
              mapRotation,
              progress
            );
          },
          1f,
          duration
        )
        .SetEase(Ease.Linear);
    }

    void LockRotationOnTarget(Vector3 focusTarget)
    {
      _camera.transform.rotation = BuildFocusRotation(focusTarget);
    }

    void DeactivateAllSites()
    {
      for (var i = 0; i < _sites.Count; i++)
      {
        var site = _sites[i];
        if (site != null)
        {
          site.SetActiveWithoutFocus(false);
        }
      }
      _activeSite = null;
    }

    void SetMapCameraPriority()
    {
      _camera.Priority = 100;
    }

    void OnDestroy()
    {
      CancelTransition();
    }

    readonly struct CameraPose
    {
      public CameraPose(Vector3 position, Quaternion rotation, float fieldOfView)
      {
        Position = position;
        Rotation = rotation;
        FieldOfView = fieldOfView;
      }

      public Vector3 Position { get; }

      public Quaternion Rotation { get; }

      public float FieldOfView { get; }
    }
  }
}
