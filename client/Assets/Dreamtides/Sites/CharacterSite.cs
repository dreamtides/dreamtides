#nullable enable

using System;
using System.Runtime.CompilerServices;
using Dreamtides.Components;
using Dreamtides.Layout;
using Unity.Cinemachine;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Sites
{
  public enum LandscapeCameraTargetSide
  {
    Left,
    Right,
  }

  public class CharacterSite : AbstractDreamscapeSite
  {
    [SerializeField]
    internal CinemachineCamera _targetScreenLeftCamera = null!;

    [SerializeField]
    internal CinemachineCamera _targetScreenRightCamera = null!;

    [SerializeField]
    internal CinemachineCamera _targetScreenTopCamera = null!;

    [SerializeField]
    internal GameObject _siteCharacter = null!;

    [SerializeField]
    internal LandscapeCameraTargetSide _landscapeCameraTargetSide = LandscapeCameraTargetSide.Left;

    [SerializeField]
    internal float _landscapeCameraDistanceModifier = 0f;

    [SerializeField]
    internal float _portraitCameraDistanceModifier = 0f;

    [SerializeField]
    internal ObjectLayout _characterOwnedObjects = null!;

    [SerializeField]
    internal Transform _characterSpeechPosition = null!;

    [SerializeField]
    internal MecanimAnimator _characterAnimator = null!;

    Vector3 _targetScreenLeftBaseDirection;
    Vector3 _targetScreenRightBaseDirection;
    Vector3 _targetScreenTopBaseDirection;
    float _targetScreenLeftBaseDistance;
    float _targetScreenRightBaseDistance;
    float _targetScreenTopBaseDistance;

    public ObjectLayout CharacterOwnedObjects => _characterOwnedObjects;

    public Transform CharacterSpeechPosition => _characterSpeechPosition;

    public MecanimAnimator CharacterAnimator => _characterAnimator;

    protected override void OnStart()
    {
      _siteCharacter.SetActive(true);
      base.OnStart();
    }

    protected override void EnsureCameraDefaults()
    {
      if (_hasCameraDefaults)
      {
        return;
      }
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
      _hasCameraDefaults = true;
    }

    protected override void ApplyDistanceModifiers(IGameViewport viewport)
    {
      if (viewport == null)
      {
        throw new InvalidOperationException("Viewport is not available.");
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

    protected override CinemachineCamera ResolveCameraForViewport(IGameViewport viewport)
    {
      if (viewport.IsLandscape)
      {
        return _landscapeCameraTargetSide == LandscapeCameraTargetSide.Left
          ? RequireCamera(_targetScreenLeftCamera, nameof(_targetScreenLeftCamera))
          : RequireCamera(_targetScreenRightCamera, nameof(_targetScreenRightCamera));
      }
      return RequireCamera(_targetScreenTopCamera, nameof(_targetScreenTopCamera));
    }

    protected override void SetCameraTargets()
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
    }

    protected override void SetCameraTarget(CinemachineCamera activeCamera)
    {
      if (activeCamera == null)
      {
        throw new InvalidOperationException($"Site {name} is missing a camera target.");
      }
      activeCamera.Follow = transform;
      activeCamera.LookAt = transform;
    }

    protected override void ResetCameraPriorities()
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
      _activeCamera = null;
    }
  }
}
