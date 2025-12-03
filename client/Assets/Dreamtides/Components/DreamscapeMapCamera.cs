#nullable enable

using System.Collections;
using System.Collections.Generic;
using Dreamtides.Buttons;
using Dreamtides.Layout;
using Dreamtides.Schema;
using Dreamtides.Services;
using Unity.Cinemachine;
using UnityEngine;

namespace Dreamtides.Components
{
  public class DreamscapeMapCamera : Displayable
  {
    static DreamscapeMapCamera? _blendOwner;
    static CinemachineCore.GetBlendOverrideDelegate? _previousBlendOverride;

    [SerializeField]
    CinemachineCamera _camera = null!;

    [SerializeField]
    float _yRotation = 0f;

    [SerializeField]
    CinemachineCamera _focusSiteCamera = null!;

    [SerializeField]
    float _toSiteTransitionWaitDuration = 1f;

    [SerializeField]
    float _fromSiteTransitionWaitDuration = 1f;

    [SerializeField]
    float _rotateBlendDuration = 0.3f;

    [SerializeField]
    List<DreamscapeSite> _sites = new();

    readonly Dictionary<DreamscapeSite, CanvasButton> _siteButtonsBySite = new();

    Coroutine? _transitionRoutine;
    Coroutine? _positionRoutine;

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
      PositionSiteButtons();
    }

    protected override void OnInitialize()
    {
      ApplyBlendOverride();
    }

    protected override void OnStart()
    {
      FrameSites();
      EnsureSiteButtons();
      PositionSiteButtons();
      SchedulePositionSiteButtons();
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
      yield return new WaitForSeconds(_toSiteTransitionWaitDuration);
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
      yield return new WaitForSeconds(_fromSiteTransitionWaitDuration);
      DeactivateAllSites();
      _camera.Priority = 21;
      _focusSiteCamera.Priority = 0;
      yield return new WaitForSeconds(_fromSiteTransitionWaitDuration);
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

    void OnDestroy()
    {
      if (_blendOwner != this)
      {
        return;
      }

      CinemachineCore.GetBlendOverride = _previousBlendOverride;
      _blendOwner = null;
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

    void ApplyBlendOverride()
    {
      if (_focusSiteCamera == null || _camera == null)
      {
        return;
      }

      _blendOwner = this;
      _previousBlendOverride ??= CinemachineCore.GetBlendOverride;
      CinemachineCore.GetBlendOverride = BlendOverride;
    }

    CinemachineBlendDefinition BlendOverride(
      ICinemachineCamera from,
      ICinemachineCamera to,
      CinemachineBlendDefinition defaultBlend,
      Object _owner
    )
    {
      if (_blendOwner == this && IsFocusBlend(from, to))
      {
        return new CinemachineBlendDefinition(
          CinemachineBlendDefinition.Styles.EaseInOut,
          _rotateBlendDuration
        );
      }

      return _previousBlendOverride != null
        ? _previousBlendOverride(from, to, defaultBlend, _owner)
        : defaultBlend;
    }

    bool IsFocusBlend(ICinemachineCamera from, ICinemachineCamera to)
    {
      return (ReferenceEquals(from, _focusSiteCamera) && ReferenceEquals(to, _camera))
        || (ReferenceEquals(from, _camera) && ReferenceEquals(to, _focusSiteCamera));
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

    void EnsureSiteButtons()
    {
      if (_sites.Count == 0)
      {
        return;
      }

      var viewport = ResolveViewport();
      if (viewport == null)
      {
        return;
      }

      var service = Registry.DreamscapeService;
      for (var i = 0; i < _sites.Count; i++)
      {
        var site = _sites[i];
        if (site == null)
        {
          continue;
        }
        if (!_siteButtonsBySite.TryGetValue(site, out var button))
        {
          button = service.CreateSiteButton();
          _siteButtonsBySite[site] = button;
        }
        button.gameObject.SetActive(true);
        button.SetView(new ButtonView { Action = GameActionEnum.NoOp, Label = site.ButtonLabel });
      }
    }

    void PositionSiteButtons()
    {
      if (_sites.Count == 0)
      {
        return;
      }

      var viewport = ResolveViewport();
      if (viewport == null)
      {
        return;
      }

      EnsureSiteButtons();
      var worldPositions = new List<Vector3>(_sites.Count);
      var buttonRects = new List<RectTransform>(_sites.Count);
      for (var i = 0; i < _sites.Count; i++)
      {
        var site = _sites[i];
        if (site == null)
        {
          continue;
        }
        if (!_siteButtonsBySite.TryGetValue(site, out var button))
        {
          continue;
        }
        worldPositions.Add(site.transform.position);
        buttonRects.Add(button.GetComponent<RectTransform>());
      }

      if (worldPositions.Count == 0 || buttonRects.Count != worldPositions.Count)
      {
        return;
      }

      var positioner = new DreamscapeSiteButtonPositioner(viewport, Registry.CanvasSafeArea);
      positioner.PositionButtons(worldPositions, buttonRects);
    }

    void SchedulePositionSiteButtons()
    {
      if (!Application.isPlaying)
      {
        return;
      }
      if (_positionRoutine != null)
      {
        StopCoroutine(_positionRoutine);
      }
      _positionRoutine = StartCoroutine(PositionSiteButtonsNextFrame());
    }

    IEnumerator PositionSiteButtonsNextFrame()
    {
      yield return new WaitForEndOfFrame();
      PositionSiteButtons();
      _positionRoutine = null;
    }

    IGameViewport? ResolveViewport() =>
      Application.isPlaying ? Registry.GameViewport : RealViewport.CreateForEditor();
  }
}
