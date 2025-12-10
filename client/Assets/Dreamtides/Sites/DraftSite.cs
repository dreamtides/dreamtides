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
  public class DraftSite : AbstractDreamscapeSite
  {
    [SerializeField]
    internal CinemachineCamera _targetDraftSiteCamera = null!;

    [SerializeField]
    internal ObjectLayout _siteDeckLayout = null!;

    Vector3 _targetDraftSiteBaseDirection;
    float _targetDraftSiteBaseDistance;

    public ObjectLayout SiteDeckLayout => _siteDeckLayout;

    protected override void EnsureCameraDefaults()
    {
      if (_hasCameraDefaults)
      {
        return;
      }
      CacheCameraDefaults(
        RequireCamera(_targetDraftSiteCamera, nameof(_targetDraftSiteCamera)),
        out _targetDraftSiteBaseDirection,
        out _targetDraftSiteBaseDistance
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
        RequireCamera(_targetDraftSiteCamera, nameof(_targetDraftSiteCamera)),
        _targetDraftSiteBaseDirection,
        _targetDraftSiteBaseDistance,
        0f
      );
    }

    protected override CinemachineCamera ResolveCameraForViewport(IGameViewport viewport)
    {
      return RequireCamera(_targetDraftSiteCamera, nameof(_targetDraftSiteCamera));
    }

    protected override void SetCameraTargets()
    {
      if (_targetDraftSiteCamera != null)
      {
        SetCameraTarget(_targetDraftSiteCamera);
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
      if (_targetDraftSiteCamera != null)
      {
        _targetDraftSiteCamera.Priority = 0;
      }
      _activeCamera = null;
    }
  }
}
