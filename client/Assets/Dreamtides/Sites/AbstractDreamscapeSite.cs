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
  public abstract class AbstractDreamscapeSite : Displayable
  {
    [SerializeField]
    internal string _siteId = Guid.NewGuid().ToString();

    [SerializeField]
    internal bool _isActive = true;

    [SerializeField]
    internal string _buttonLabel = string.Empty;

    [SerializeField]
    internal string _debugClickAction = string.Empty;

    protected const int DefaultActivePriority = 60;

    protected bool _hasCameraDefaults;
    protected CinemachineCamera? _activeCamera;
    protected int _activePriority = DefaultActivePriority;

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
      return UpdateActiveCamera(viewport);
    }

    internal void SetActiveWithoutFocus(bool isActive)
    {
      _activePriority = DefaultActivePriority;
      SetActiveInternal(isActive);
    }

    protected internal virtual CinemachineCamera GetActiveCamera(IGameViewport viewport)
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
      OnDeactivated();
    }

    protected override void OnInitialize()
    {
      EnsureCameraDefaults();
    }

    protected override void OnStart()
    {
      EnsureCameraDefaults();
      SetCameraTargets();
      if (_isActive)
      {
        var viewport = Registry.GameViewport;
        UpdateActiveCamera(viewport);
      }
    }

    protected override void OnUpdate()
    {
      if (!_isActive)
      {
        return;
      }
      var viewport = Registry.GameViewport;
      UpdateActiveCamera(viewport);
    }

    protected virtual void OnActiveCameraChanged(
      IGameViewport viewport,
      CinemachineCamera camera
    ) { }

    protected virtual void OnDeactivated() { }

    public virtual void OnWillOpenSite() { }

    public virtual void OnOpenedSite() { }

    protected abstract void EnsureCameraDefaults();

    protected abstract void ApplyDistanceModifiers(IGameViewport viewport);

    protected abstract CinemachineCamera ResolveCameraForViewport(IGameViewport viewport);

    protected abstract void SetCameraTargets();

    protected abstract void SetCameraTarget(CinemachineCamera activeCamera);

    protected abstract void ResetCameraPriorities();

    protected static void CacheCameraDefaults(
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

    protected static void ApplyDistanceModifier(
      CinemachineCamera camera,
      Vector3 baseDirection,
      float baseDistance,
      float modifier
    )
    {
      var distance = Mathf.Max(0.01f, baseDistance + modifier);
      camera.transform.localPosition = baseDirection * distance;
    }

    protected CinemachineCamera RequireCamera(CinemachineCamera camera, string fieldName)
    {
      if (camera == null)
      {
        throw new InvalidOperationException($"{name} is missing required camera {fieldName}.");
      }
      return camera;
    }

    void SetActiveInternal(bool isActive)
    {
      EnsureCameraDefaults();
      _isActive = isActive;
      if (_isActive)
      {
        var viewport = Registry.GameViewport;
        UpdateActiveCamera(viewport);
      }
      else
      {
        ResetCameraPriorities();
        OnDeactivated();
      }
    }

    void SetCameraPriority(CinemachineCamera camera, int priority)
    {
      camera.Priority = priority;
    }

    CinemachineCamera UpdateActiveCamera(IGameViewport viewport)
    {
      var camera = GetActiveCamera(viewport);
      var cameraChanged = _activeCamera != camera;
      if (camera.Priority != _activePriority)
      {
        SetCameraPriority(camera, _activePriority);
      }
      if (cameraChanged)
      {
        _activeCamera = camera;
        OnActiveCameraChanged(viewport, camera);
      }
      return camera;
    }
  }
}
