#nullable enable

using System;
using Dreamtides.Layout;
using Dreamtides.Services;
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
    CinemachineCamera _targetDraftSiteCamera = null!;

    [SerializeField]
    GameObject _siteCharacter = null!;

    [SerializeField]
    bool _draftSite;

    [SerializeField]
    LandscapeCameraTargetSide _landscapeCameraTargetSide = LandscapeCameraTargetSide.Left;

    [SerializeField]
    float _landscapeCameraDistanceModifier = 0f;

    [SerializeField]
    float _portraitCameraDistanceModifier = 0f;

    [SerializeField]
    string _siteId = Guid.NewGuid().ToString();

    [SerializeField]
    bool _isActive = true;

    [SerializeField]
    string _buttonLabel = string.Empty;

    [SerializeField]
    string _debugClickAction = string.Empty;

    [SerializeField]
    ObjectLayout _characterOwnedObjects = null!;

    [SerializeField]
    ObjectLayout _siteDeckLayout = null!;

    [SerializeField]
    Transform _characterSpeechPosition = null!;

    [SerializeField]
    MecanimAnimator _characterAnimator = null!;

    const int DefaultActivePriority = 60;

    bool _hasCameraDefaults;
    Vector3 _targetScreenLeftBaseDirection;
    Vector3 _targetScreenRightBaseDirection;
    Vector3 _targetScreenTopBaseDirection;
    float _targetScreenLeftBaseDistance;
    float _targetScreenRightBaseDistance;
    float _targetScreenTopBaseDistance;
    Vector3 _targetDraftSiteBaseDirection;
    float _targetDraftSiteBaseDistance;
    CinemachineCamera? _activeCamera;
    int _activePriority = DefaultActivePriority;

    public bool IsActive => _isActive;

    public string ButtonLabel => _buttonLabel;

    public string DebugClickAction => _debugClickAction;

    public Guid SiteId
    {
      get
      {
        if (Guid.TryParse(_siteId, out var parsed))
        {
          return parsed;
        }
        throw new InvalidOperationException($"Invalid site id string: {_siteId}");
      }
    }

    public ObjectLayout CharacterOwnedObjects => _characterOwnedObjects;

    public ObjectLayout SiteDeckLayout => _siteDeckLayout;

    public Transform CharacterSpeechPosition => _characterSpeechPosition;

    public MecanimAnimator CharacterAnimator => _characterAnimator;

    public void SetActive(bool isActive)
    {
      _activePriority = DefaultActivePriority;
      SetActiveInternal(isActive);
    }

    public void Activate()
    {
      SetActive(isActive: true);
    }

    internal CinemachineCamera ActivateForViewport(IGameViewport viewport, int priority)
    {
      if (viewport == null)
      {
        throw new InvalidOperationException("Viewport is not available.");
      }
      _activePriority = priority;
      _isActive = true;
      var camera = GetActiveCamera(viewport);
      SetCameraPriority(camera, priority);
      _activeCamera = camera;
      return camera;
    }

    internal void SetActiveWithoutFocus(bool isActive)
    {
      _activePriority = DefaultActivePriority;
      SetActiveInternal(isActive);
    }

    internal CinemachineCamera GetActiveCamera(IGameViewport viewport)
    {
      EnsureCameraDefaults();
      ApplyDistanceModifiers(viewport);
      var camera = ResolveCameraForViewport(viewport);
      SetCameraTarget(camera);
      return camera;
    }

    internal void Deactivate()
    {
      _isActive = false;
      ResetCameraPriorities();
    }

    protected override void OnInitialize()
    {
      EnsureCameraDefaults();
    }

    protected override void OnStart()
    {
      _siteCharacter.SetActive(!_draftSite);
      EnsureCameraDefaults();
      SetCameraTargets();
      if (_isActive)
      {
        var viewport = Registry.GameViewport;
        var camera = GetActiveCamera(viewport);
        SetCameraPriority(camera, _activePriority);
        _activeCamera = camera;
      }
    }

    protected override void OnUpdate()
    {
      if (!_isActive)
      {
        return;
      }
      var viewport = Registry.GameViewport;
      var camera = GetActiveCamera(viewport);
      if (_activeCamera != camera || camera.Priority != _activePriority)
      {
        SetCameraPriority(camera, _activePriority);
        _activeCamera = camera;
      }
    }

    void SetActiveInternal(bool isActive)
    {
      EnsureCameraDefaults();
      _isActive = isActive;
      if (_isActive)
      {
        var viewport = Registry.GameViewport;
        var camera = GetActiveCamera(viewport);
        SetCameraPriority(camera, _activePriority);
        _activeCamera = camera;
      }
      else
      {
        ResetCameraPriorities();
      }
    }

    void EnsureCameraDefaults()
    {
      if (_hasCameraDefaults)
      {
        return;
      }
      if (_draftSite)
      {
        CacheCameraDefaults(
          RequireCamera(_targetDraftSiteCamera, nameof(_targetDraftSiteCamera)),
          out _targetDraftSiteBaseDirection,
          out _targetDraftSiteBaseDistance
        );
      }
      else
      {
        CacheCameraDefaults(
          RequireCamera(_targetScreenLeftCamera, nameof(_targetScreenLeftCamera)),
          out _targetScreenLeftBaseDirection,
          out _targetScreenLeftBaseDistance
        );
        CacheCameraDefaults(
          RequireCamera(_targetScreenRightCamera, nameof(_targetScreenRightCamera)),
          out _targetScreenRightBaseDirection,
          out _targetScreenRightBaseDistance
        );
        CacheCameraDefaults(
          RequireCamera(_targetScreenTopCamera, nameof(_targetScreenTopCamera)),
          out _targetScreenTopBaseDirection,
          out _targetScreenTopBaseDistance
        );
      }
      _hasCameraDefaults = true;
    }

    void ApplyDistanceModifiers(IGameViewport viewport)
    {
      if (viewport == null)
      {
        throw new InvalidOperationException("Viewport is not available.");
      }
      if (_draftSite)
      {
        ApplyDistanceModifier(
          RequireCamera(_targetDraftSiteCamera, nameof(_targetDraftSiteCamera)),
          _targetDraftSiteBaseDirection,
          _targetDraftSiteBaseDistance,
          0f
        );
        return;
      }
      ApplyDistanceModifier(
        RequireCamera(_targetScreenLeftCamera, nameof(_targetScreenLeftCamera)),
        _targetScreenLeftBaseDirection,
        _targetScreenLeftBaseDistance,
        _landscapeCameraDistanceModifier
      );
      ApplyDistanceModifier(
        RequireCamera(_targetScreenRightCamera, nameof(_targetScreenRightCamera)),
        _targetScreenRightBaseDirection,
        _targetScreenRightBaseDistance,
        _landscapeCameraDistanceModifier
      );
      ApplyDistanceModifier(
        RequireCamera(_targetScreenTopCamera, nameof(_targetScreenTopCamera)),
        _targetScreenTopBaseDirection,
        _targetScreenTopBaseDistance,
        _portraitCameraDistanceModifier
      );
    }

    CinemachineCamera ResolveCameraForViewport(IGameViewport viewport)
    {
      if (_draftSite)
      {
        return RequireCamera(_targetDraftSiteCamera, nameof(_targetDraftSiteCamera));
      }
      if (viewport.IsLandscape)
      {
        return _landscapeCameraTargetSide == LandscapeCameraTargetSide.Left
          ? RequireCamera(_targetScreenLeftCamera, nameof(_targetScreenLeftCamera))
          : RequireCamera(_targetScreenRightCamera, nameof(_targetScreenRightCamera));
      }
      return RequireCamera(_targetScreenTopCamera, nameof(_targetScreenTopCamera));
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
      baseDistance = Mathf.Max(0.01f, localPosition.magnitude);
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

    void SetCameraPriority(CinemachineCamera camera, int priority)
    {
      camera.Priority = priority;
    }

    void ResetCameraPriorities()
    {
      if (_targetScreenLeftCamera != null)
      {
        _targetScreenLeftCamera.Priority = 0;
      }
      if (_targetScreenRightCamera != null)
      {
        _targetScreenRightCamera.Priority = 0;
      }
      if (_targetScreenTopCamera != null)
      {
        _targetScreenTopCamera.Priority = 0;
      }
      if (_targetDraftSiteCamera != null)
      {
        _targetDraftSiteCamera.Priority = 0;
      }
      _activeCamera = null;
    }

    void SetCameraTargets()
    {
      if (_targetScreenLeftCamera != null)
      {
        SetCameraTarget(_targetScreenLeftCamera);
      }
      if (_targetScreenRightCamera != null)
      {
        SetCameraTarget(_targetScreenRightCamera);
      }
      if (_targetScreenTopCamera != null)
      {
        SetCameraTarget(_targetScreenTopCamera);
      }
      if (_targetDraftSiteCamera != null)
      {
        SetCameraTarget(_targetDraftSiteCamera);
      }
    }

    void SetCameraTarget(CinemachineCamera activeCamera)
    {
      if (activeCamera == null)
      {
        throw new InvalidOperationException($"Site {name} is missing a camera target.");
      }
      activeCamera.Follow = transform;
      activeCamera.LookAt = transform;
    }

    CinemachineCamera RequireCamera(CinemachineCamera camera, string fieldName)
    {
      if (camera == null)
      {
        throw new InvalidOperationException($"{name} is missing required camera {fieldName}.");
      }
      return camera;
    }
  }
}
