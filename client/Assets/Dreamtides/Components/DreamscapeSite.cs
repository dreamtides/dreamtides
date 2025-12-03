#nullable enable

using System.Collections;
using Dreamtides.Layout;
using Unity.Cinemachine;
using UnityEngine;

namespace Dreamtides.Components
{
  public enum LandscapeCameraTargetSide
  {
    Left,
    Right,
  }

  public class DreamscapeSite : Displayable
  {
    [SerializeField]
    CinemachineCamera _targetScreenLeftCamera = null!;

    [SerializeField]
    CinemachineCamera _targetScreenRightCamera = null!;

    [SerializeField]
    CinemachineCamera _targetScreenTopCamera = null!;

    [SerializeField]
    LandscapeCameraTargetSide _landscapeCameraTargetSide = LandscapeCameraTargetSide.Left;

    [SerializeField]
    float _landscapeCameraDistanceModifier = 0f;

    [SerializeField]
    float _portraitCameraDistanceModifier = 0f;

    [SerializeField]
    bool _isActive = true;

    [SerializeField]
    string _buttonLabel = string.Empty;

    DreamscapeMapCamera? _mapCamera;
    Coroutine? _activationRoutine;
    bool _hasCameraDefaults;
    Vector3 _targetScreenLeftBaseDirection;
    Vector3 _targetScreenRightBaseDirection;
    Vector3 _targetScreenTopBaseDirection;
    float _targetScreenLeftBaseDistance;
    float _targetScreenRightBaseDistance;
    float _targetScreenTopBaseDistance;
    CinemachineCamera? _activeCamera;

    public bool IsActive => _isActive;

    public string ButtonLabel => _buttonLabel;

    public void SetActive(bool isActive)
    {
      EnsureCameraDefaults();
      _isActive = isActive;
      if (_isActive && _hasCameraDefaults)
      {
        if (_activationRoutine != null)
        {
          StopCoroutine(_activationRoutine);
        }
        if (!TryStartFocusTransition())
        {
          ApplyCameraState();
        }
      }
      else if (!_isActive)
      {
        if (_activationRoutine != null)
        {
          StopCoroutine(_activationRoutine);
          _activationRoutine = null;
        }
        ResetCameraPriorities();
      }
    }

    public void Activate()
    {
      EnsureCameraDefaults();
      SetActive(isActive: true);
    }

    public void SetMapCamera(DreamscapeMapCamera mapCamera)
    {
      _mapCamera = mapCamera;
    }

    protected override void OnInitialize()
    {
      EnsureCameraDefaults();
    }

    protected override void OnUpdate()
    {
      if (!_isActive)
      {
        return;
      }
      if (_activationRoutine != null)
      {
        return;
      }
      ApplyCameraState();
    }

    void EnsureCameraDefaults()
    {
      if (_hasCameraDefaults)
      {
        return;
      }
      CacheCameraDefaults(
        _targetScreenLeftCamera,
        out _targetScreenLeftBaseDirection,
        out _targetScreenLeftBaseDistance
      );
      CacheCameraDefaults(
        _targetScreenRightCamera,
        out _targetScreenRightBaseDirection,
        out _targetScreenRightBaseDistance
      );
      CacheCameraDefaults(
        _targetScreenTopCamera,
        out _targetScreenTopBaseDirection,
        out _targetScreenTopBaseDistance
      );
      _hasCameraDefaults = true;
    }

    void ApplyCameraState()
    {
      if (!_hasCameraDefaults)
      {
        return;
      }
      var viewport = Registry.GameViewport;
      ApplyDistanceModifier(
        _targetScreenLeftCamera,
        _targetScreenLeftBaseDirection,
        _targetScreenLeftBaseDistance,
        _landscapeCameraDistanceModifier
      );
      ApplyDistanceModifier(
        _targetScreenRightCamera,
        _targetScreenRightBaseDirection,
        _targetScreenRightBaseDistance,
        _landscapeCameraDistanceModifier
      );
      ApplyDistanceModifier(
        _targetScreenTopCamera,
        _targetScreenTopBaseDirection,
        _targetScreenTopBaseDistance,
        _portraitCameraDistanceModifier
      );
      var activeCamera = viewport.IsLandscape
        ? _landscapeCameraTargetSide == LandscapeCameraTargetSide.Left
          ? _targetScreenLeftCamera
          : _targetScreenRightCamera
        : _targetScreenTopCamera;
      SetCameraTarget(activeCamera);
      if (_activeCamera != activeCamera)
      {
        SetActiveCamera(activeCamera);
      }
    }

    static void CacheCameraDefaults(
      CinemachineCamera camera,
      out Vector3 baseDirection,
      out float baseDistance
    )
    {
      var localPosition = camera.transform.localPosition;
      baseDirection =
        localPosition.sqrMagnitude < Mathf.Epsilon ? Vector3.back : localPosition.normalized;
      baseDistance = localPosition.magnitude;
    }

    void ApplyDistanceModifier(
      CinemachineCamera camera,
      Vector3 baseDirection,
      float baseDistance,
      float modifier
    )
    {
      var distance = Mathf.Max(0.01f, baseDistance + modifier);
      camera.transform.localPosition = baseDirection * distance;
    }

    void SetActiveCamera(CinemachineCamera activeCamera)
    {
      _targetScreenLeftCamera.Priority = _targetScreenLeftCamera == activeCamera ? 10 : 0;
      _targetScreenRightCamera.Priority = _targetScreenRightCamera == activeCamera ? 10 : 0;
      _targetScreenTopCamera.Priority = _targetScreenTopCamera == activeCamera ? 10 : 0;
      _activeCamera = activeCamera;
    }

    void ResetCameraPriorities()
    {
      _targetScreenLeftCamera.Priority = 0;
      _targetScreenRightCamera.Priority = 0;
      _targetScreenTopCamera.Priority = 0;
      _activeCamera = null;
      _activationRoutine = null;
    }

    void SetCameraTarget(CinemachineCamera activeCamera)
    {
      activeCamera.Follow = transform;
      activeCamera.LookAt = transform;
    }

    bool TryStartFocusTransition()
    {
      if (!Application.isPlaying)
      {
        return false;
      }

      var mapCamera = _mapCamera;
      if (mapCamera == null || mapCamera.FocusSiteCamera == null)
      {
        return false;
      }

      _activationRoutine = StartCoroutine(FocusThenActivate(mapCamera));
      return true;
    }

    IEnumerator FocusThenActivate(DreamscapeMapCamera mapCamera)
    {
      yield return mapCamera.FocusOnSite(this);
      ApplyCameraState();
      _activationRoutine = null;
    }
  }
}
