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
    CinemachineCamera _leaveSiteCamera = null!;

    [SerializeField]
    List<DreamscapeSite> _sites = new();

    Coroutine? _transitionRoutine;

    public CinemachineCamera Camera => _camera;

    public void ActivateWithTransition()
    {
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
        FindObjectsInactive.Include,
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

    IEnumerator TransitionToMap()
    {
      // All of this logic basically exists to make the site -> map camera
      // transition look slightly better. We use a two-camera setup so that the
      // camera "pulls back" to the map view while keeping the site in focus.
      // It's not really necessary, but I find it satisfying to watch.

      var brain = ResolveBrain();
      var activeSite = GetActiveSite();
      var targetTransform = activeSite != null ? activeSite.transform : null;
      if (_leaveSiteCamera == null || targetTransform == null)
      {
        _camera.Priority = 10;
        if (_leaveSiteCamera != null)
        {
          _leaveSiteCamera.Priority = 0;
        }
        DeactivateAllSites();
        _transitionRoutine = null;
        yield break;
      }

      ConfigureLeaveSiteCamera(targetTransform, _camera.transform.position);
      _camera.Priority = 0;
      _leaveSiteCamera.Priority = 20;
      yield return WaitForBlend(brain);
      DeactivateAllSites();
      _camera.Priority = 21;
      _leaveSiteCamera.Priority = 0;
      yield return WaitForBlend(brain);
      _transitionRoutine = null;
    }

    IEnumerator WaitForBlend(CinemachineBrain? brain)
    {
      yield return null;
      if (brain == null)
      {
        yield break;
      }

      while (brain.IsBlending)
      {
        yield return null;
      }
    }

    void ConfigureLeaveSiteCamera(Transform target, Vector3 mapPosition)
    {
      SetLeaveSiteLens();
      _leaveSiteCamera.Follow = target;
      _leaveSiteCamera.LookAt = target;
      _leaveSiteCamera.transform.position = mapPosition;
      var direction = target.position - mapPosition;
      _leaveSiteCamera.transform.rotation =
        direction.sqrMagnitude < Mathf.Epsilon
          ? _camera.transform.rotation
          : Quaternion.LookRotation(direction, Vector3.up);
    }

    void SetLeaveSiteLens()
    {
      var lens = _leaveSiteCamera.Lens;
      lens.FieldOfView = 60f;
      _leaveSiteCamera.Lens = lens;
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

    CinemachineBrain? ResolveBrain()
    {
      if (!Application.isPlaying)
      {
        return null;
      }

      var mainCamera = Registry.MainCamera;
      return mainCamera != null ? mainCamera.GetComponent<CinemachineBrain>() : null;
    }

    IGameViewport? ResolveViewport() =>
      Application.isPlaying ? Registry.GameViewport : RealViewport.CreateForEditor();
  }
}
