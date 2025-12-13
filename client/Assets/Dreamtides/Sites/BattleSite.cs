#nullable enable

using System;
using System.Runtime.CompilerServices;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Services;
using Unity.Cinemachine;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Sites
{
  public class BattleSite : AbstractDreamscapeSite
  {
    [SerializeField]
    internal Transform _portraitBattleLayoutAnchor = null!;

    [SerializeField]
    internal Transform _landscapeBattleLayoutAnchor = null!;

    [SerializeField]
    internal bool _debugUpdateContinuously;

    [SerializeField]
    internal ObjectLayout _battleCardOrigin = null!;

    bool _lastIsLandscape;
    bool _hasValidatedBattleCamera;

    public ObjectLayout BattleCardOrigin => _battleCardOrigin;

    protected override void OnActiveCameraChanged(IGameViewport viewport, CinemachineCamera camera)
    {
      if (Mode == GameMode.Battle)
      {
        return;
      }
      AlignBattleLayout(viewport);
    }

    protected override void OnUpdate()
    {
      if (Mode == GameMode.Battle)
      {
        return;
      }
      base.OnUpdate();
      var viewport = Registry.GameViewport;
      if (!_debugUpdateContinuously && !IsActive)
      {
        return;
      }
      var isLandscape = viewport.IsLandscape;
      if (_debugUpdateContinuously || isLandscape != _lastIsLandscape)
      {
        AlignBattleLayout(viewport);
      }
    }

    protected override void EnsureCameraDefaults()
    {
      if (_hasValidatedBattleCamera)
      {
        return;
      }
      RequireCamera(ResolveBattleCamera(), "BattleCamera");
      _hasValidatedBattleCamera = true;
    }

    protected override void ApplyDistanceModifiers(IGameViewport viewport)
    {
      if (viewport == null)
      {
        throw new InvalidOperationException("Viewport is not available.");
      }
    }

    protected override CinemachineCamera ResolveCameraForViewport(IGameViewport viewport)
    {
      if (viewport == null)
      {
        throw new InvalidOperationException("Viewport is not available.");
      }
      return ResolveBattleCamera();
    }

    protected override void SetCameraTargets()
    {
      var camera = ResolveBattleCamera();
      SetCameraTarget(camera);
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
      var camera = ResolveBattleCamera();
      camera.Priority = 0;
      _activeCamera = null;
    }

    void AlignBattleLayout(IGameViewport viewport)
    {
      var anchor = ResolveBattleAnchor(viewport);
      var layout = anchor.parent;
      if (layout == null)
      {
        return;
      }
      var layoutRotation = transform.rotation * Quaternion.Inverse(anchor.localRotation);
      var layoutPosition = transform.position - layoutRotation * anchor.localPosition;
      layout.SetPositionAndRotation(layoutPosition, layoutRotation);
      _lastIsLandscape = viewport.IsLandscape;
    }

    CinemachineCamera ResolveBattleCamera()
    {
      return RequireCamera(Registry.BattleLayout.BattleCamera, "BattleCamera");
    }

    Transform ResolveBattleAnchor(IGameViewport viewport)
    {
      var anchor = viewport.IsLandscape
        ? _landscapeBattleLayoutAnchor
        : _portraitBattleLayoutAnchor;
      if (anchor == null)
      {
        throw new InvalidOperationException("Battle site is missing a battle layout anchor.");
      }
      return anchor;
    }
  }
}
