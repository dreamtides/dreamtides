#nullable enable

using System.Collections;
using System.Collections.Generic;
using Dreamtides.Layout;
using Dreamtides.Services;
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
    CinemachineCamera _focusSiteCamera = null!;

    [SerializeField]
    float _transitionWaitDuration = 1f;

    [SerializeField]
    List<DreamscapeSite> _sites = new();

    Coroutine? _transitionRoutine;

    public CinemachineCamera Camera => _camera;

    public CinemachineCamera FocusSiteCamera => _focusSiteCamera;

    public void ActivateWithTransition()
    {
      // All of this logic basically exists to make the site <-> map camera
      // transition look slightly better. We use a two-camera setup so that the
      // camera "pulls back" to the map view while keeping the site in focus.
      // It's not really necessary, but I find it satisfying to watch.

      if (_camera == null)
      {
        return;
      }

      FrameSites();
      if (_transitionRoutine != null)
      {
        StopCoroutine(_transitionRoutine);
      }

      _transitionRoutine = StartCoroutine(TransitionToMap());
    }

    public void FrameSites()
    {
      if (_camera == null)
      {
        return;
      }

      var viewport = ResolveViewport();
      if (viewport == null)
      {
        return;
      }

      var sites = FindObjectsByType<DreamscapeSite>(
        FindObjectsInactive.Exclude,
        FindObjectsSortMode.None
      );

      _sites.Clear();
      var positions = new List<Vector3>(sites.Length);
      for (var i = 0; i < sites.Length; i++)
      {
        var site = sites[i];
        if (site == null)
        {
          continue;
        }

        _sites.Add(site);
        site.SetMapCamera(this);
        if (!site.gameObject.scene.isLoaded || !site.gameObject.activeInHierarchy)
        {
          continue;
        }

        positions.Add(site.transform.position);
      }

      if (positions.Count == 0)
      {
        return;
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
      SetLensFieldOfView();
      _camera.transform.SetPositionAndRotation(position, rotation);
      SyncFocusSitePosition();
    }

    protected override void OnInitialize()
    {
      FrameSites();
    }

    protected override void OnUpdate()
    {
      FrameSites();
    }

    void SetLensFieldOfView()
    {
      var lens = _camera.Lens;
      lens.FieldOfView = 60f;
      _camera.Lens = lens;
    }

    public IEnumerator FocusOnSite(DreamscapeSite site)
    {
      if (_focusSiteCamera == null)
      {
        yield break;
      }

      SyncFocusSitePosition();
      SetFocusSiteLens();
      _focusSiteCamera.Follow = site.transform;
      _focusSiteCamera.LookAt = site.transform;
      AimFocusSiteCamera(site.transform);
      var priority = Mathf.Max(_camera.Priority, _focusSiteCamera.Priority) + 1;
      _focusSiteCamera.Priority = priority;
      yield return new WaitForSeconds(_transitionWaitDuration);
      _camera.Priority = 0;
      _focusSiteCamera.Priority = 0;
      SyncFocusSitePosition();
    }

    IEnumerator TransitionToMap()
    {
      var activeSite = GetActiveSite();
      var targetTransform = activeSite != null ? activeSite.transform : null;
      if (_focusSiteCamera == null || targetTransform == null)
      {
        _camera.Priority = 10;
        if (_focusSiteCamera != null)
        {
          _focusSiteCamera.Priority = 0;
        }
        DeactivateAllSites();
        _transitionRoutine = null;
        yield break;
      }

      ConfigureFocusSiteCamera(targetTransform, _camera.transform.position);
      _camera.Priority = 0;
      _focusSiteCamera.Priority = 20;
      yield return new WaitForSeconds(_transitionWaitDuration);
      DeactivateAllSites();
      _camera.Priority = 21;
      _focusSiteCamera.Priority = 0;
      yield return new WaitForSeconds(_transitionWaitDuration);
      SyncFocusSitePosition();
      _transitionRoutine = null;
    }

    void ConfigureFocusSiteCamera(Transform target, Vector3 mapPosition)
    {
      SetFocusSiteLens();
      _focusSiteCamera.Follow = target;
      _focusSiteCamera.LookAt = target;
      _focusSiteCamera.transform.position = mapPosition;
      AimFocusSiteCamera(target);
    }

    void SetFocusSiteLens()
    {
      var lens = _focusSiteCamera.Lens;
      lens.FieldOfView = 60f;
      _focusSiteCamera.Lens = lens;
    }

    void SyncFocusSitePosition()
    {
      if (_focusSiteCamera == null)
      {
        return;
      }

      _focusSiteCamera.transform.position = _camera.transform.position;
    }

    void AimFocusSiteCamera(Transform target)
    {
      if (_focusSiteCamera == null)
      {
        return;
      }

      var direction = target.position - _focusSiteCamera.transform.position;
      _focusSiteCamera.transform.rotation =
        direction.sqrMagnitude < Mathf.Epsilon
          ? _camera.transform.rotation
          : Quaternion.LookRotation(direction, Vector3.up);
    }

    DreamscapeSite? GetActiveSite()
    {
      for (var i = 0; i < _sites.Count; i++)
      {
        var site = _sites[i];
        if (site != null && site.IsActive)
        {
          return site;
        }
      }

      return null;
    }

    void DeactivateAllSites()
    {
      for (var i = 0; i < _sites.Count; i++)
      {
        var site = _sites[i];
        if (site != null)
        {
          site.SetActive(false);
        }
      }
    }

    IGameViewport? ResolveViewport() =>
      Application.isPlaying ? Registry.GameViewport : RealViewport.CreateForEditor();
  }
}
