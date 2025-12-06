#nullable enable

using System;
using Unity.Cinemachine;
using UnityEngine;

namespace Dreamtides.Components
{
  public class DreamscapeBattleSite : DreamscapeSite
  {
    [SerializeField]
    Transform _portraitBattleLayoutAnchor = null!;

    [SerializeField]
    Transform _landscapeBattleLayoutAnchor = null!;

    bool _lastIsLandscape;
    bool _hasValidatedBattleCamera;

    [SerializeField]
    bool _debugUpdateContinuously;

    protected override bool HasSiteCharacter => false;

    protected override void OnActiveCameraChanged(IGameViewport viewport, CinemachineCamera camera)
    {
      AlignBattleLayout(viewport);
    }

    protected override void OnUpdate()
    {
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

    protected override void ResetCameraPriorities()
    {
      var camera = ResolveBattleCamera();
      camera.Priority = 0;
      base.ResetCameraPriorities();
    }

    void AlignBattleLayout(IGameViewport viewport)
    {
      var anchor = ResolveBattleAnchor(viewport);
      var layout = anchor.parent;
      if (layout == null)
      {
        throw new InvalidOperationException("Battle layout anchor is missing a parent.");
      }
      var layoutRotation = transform.rotation * Quaternion.Inverse(anchor.localRotation);
      var layoutPosition = transform.position - layoutRotation * anchor.localPosition;
      layout.SetPositionAndRotation(layoutPosition, layoutRotation);
      _lastIsLandscape = viewport.IsLandscape;
    }

    CinemachineCamera ResolveBattleCamera()
    {
      return RequireCamera(Registry.Layout.BattleCamera, "BattleCamera");
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
