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

  public readonly struct DreamscapeSiteCameraPose
  {
    public DreamscapeSiteCameraPose(
      Vector3 position,
      Quaternion rotation,
      float fieldOfView,
      Vector3 focusTarget
    )
    {
      Position = position;
      Rotation = rotation;
      FieldOfView = fieldOfView;
      FocusTarget = focusTarget;
    }

    public Vector3 Position { get; }

    public Quaternion Rotation { get; }

    public float FieldOfView { get; }

    public Vector3 FocusTarget { get; }
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

    bool _hasCameraDefaults;
    Vector3 _targetScreenLeftBaseDirection;
    Vector3 _targetScreenRightBaseDirection;
    Vector3 _targetScreenTopBaseDirection;
    float _targetScreenLeftBaseDistance;
    float _targetScreenRightBaseDistance;
    float _targetScreenTopBaseDistance;
    Vector3 _targetDraftSiteBaseDirection;
    float _targetDraftSiteBaseDistance;

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

    public DreamscapeSiteCameraPose ResolveCameraPose(
      IGameViewport viewport,
      float fallbackFieldOfView
    )
    {
      if (viewport == null)
      {
        throw new InvalidOperationException("Viewport is not available.");
      }
      if (_hasCameraDefaults)
      {
        return BuildCameraPose(viewport, fallbackFieldOfView);
      }
      EnsureCameraDefaults();
      return BuildCameraPose(viewport, fallbackFieldOfView);
    }

    public void SetActive(bool isActive)
    {
      EnsureCameraDefaults();
      _isActive = isActive;
    }

    public void Activate()
    {
      SetActive(isActive: true);
    }

    internal void SetActiveWithoutFocus(bool isActive)
    {
      _isActive = isActive;
    }

    protected override void OnInitialize()
    {
      EnsureCameraDefaults();
      DisableAnchorCameras();
    }

    protected override void OnStart()
    {
      _siteCharacter.SetActive(!_draftSite);
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
      if (_draftSite)
      {
        CacheCameraDefaults(
          _targetDraftSiteCamera,
          out _targetDraftSiteBaseDirection,
          out _targetDraftSiteBaseDistance
        );
      }
      _hasCameraDefaults = true;
    }

    DreamscapeSiteCameraPose BuildCameraPose(IGameViewport viewport, float fallbackFieldOfView)
    {
      var anchor = ResolveAnchor(viewport);
      var distance = Mathf.Max(0.01f, anchor.Distance + anchor.DistanceModifier);
      var localOffset = anchor.Direction * distance;
      var worldPosition = transform.TransformPoint(localOffset);
      var focusTarget = transform.position;
      var rotation = BuildFocusRotation(worldPosition, focusTarget);
      var fieldOfView =
        anchor.Camera != null ? anchor.Camera.Lens.FieldOfView : Mathf.Max(1f, fallbackFieldOfView);
      return new DreamscapeSiteCameraPose(worldPosition, rotation, fieldOfView, focusTarget);
    }

    CameraAnchor ResolveAnchor(IGameViewport viewport)
    {
      if (_draftSite)
      {
        if (_targetDraftSiteCamera == null)
        {
          throw new InvalidOperationException("Draft site camera is missing.");
        }
        return new CameraAnchor(
          _targetDraftSiteCamera,
          _targetDraftSiteBaseDirection,
          _targetDraftSiteBaseDistance,
          0f
        );
      }
      if (_targetScreenLeftCamera == null)
      {
        throw new InvalidOperationException("Left site camera is missing.");
      }
      if (_targetScreenRightCamera == null)
      {
        throw new InvalidOperationException("Right site camera is missing.");
      }
      if (_targetScreenTopCamera == null)
      {
        throw new InvalidOperationException("Top site camera is missing.");
      }
      if (viewport.IsLandscape)
      {
        return _landscapeCameraTargetSide == LandscapeCameraTargetSide.Left
          ? new CameraAnchor(
            _targetScreenLeftCamera,
            _targetScreenLeftBaseDirection,
            _targetScreenLeftBaseDistance,
            _landscapeCameraDistanceModifier
          )
          : new CameraAnchor(
            _targetScreenRightCamera,
            _targetScreenRightBaseDirection,
            _targetScreenRightBaseDistance,
            _landscapeCameraDistanceModifier
          );
      }
      return new CameraAnchor(
        _targetScreenTopCamera,
        _targetScreenTopBaseDirection,
        _targetScreenTopBaseDistance,
        _portraitCameraDistanceModifier
      );
    }

    static Quaternion BuildFocusRotation(Vector3 cameraPosition, Vector3 focusTarget)
    {
      var direction = focusTarget - cameraPosition;
      return direction.sqrMagnitude < Mathf.Epsilon
        ? Quaternion.identity
        : Quaternion.LookRotation(direction, Vector3.up);
    }

    static void CacheCameraDefaults(
      CinemachineCamera camera,
      out Vector3 baseDirection,
      out float baseDistance
    )
    {
      if (camera == null)
      {
        throw new InvalidOperationException("Site camera anchor is not assigned.");
      }
      var localPosition = camera.transform.localPosition;
      baseDirection =
        localPosition.sqrMagnitude < Mathf.Epsilon ? Vector3.back : localPosition.normalized;
      baseDistance = localPosition.magnitude;
    }

    void DisableAnchorCameras()
    {
      DisableCamera(_targetScreenLeftCamera);
      DisableCamera(_targetScreenRightCamera);
      DisableCamera(_targetScreenTopCamera);
      if (_targetDraftSiteCamera != null)
      {
        DisableCamera(_targetDraftSiteCamera);
      }
    }

    static void DisableCamera(CinemachineCamera camera)
    {
      if (camera == null)
      {
        throw new InvalidOperationException("Site camera anchor is not assigned.");
      }
      camera.Priority = -100;
      camera.enabled = false;
    }

    readonly struct CameraAnchor
    {
      public CameraAnchor(
        CinemachineCamera camera,
        Vector3 direction,
        float distance,
        float distanceModifier
      )
      {
        Camera = camera;
        Direction = direction;
        Distance = distance;
        DistanceModifier = distanceModifier;
      }

      public CinemachineCamera Camera { get; }

      public Vector3 Direction { get; }

      public float Distance { get; }

      public float DistanceModifier { get; }
    }
  }
}
