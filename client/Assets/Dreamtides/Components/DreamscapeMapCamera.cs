#nullable enable

using System;
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
    const int MapPriority = 70;
    const int SitePriority = 80;
    const int ReturnMapPriority = 90;
    const float BlendTimeoutSeconds = 10f;

    [SerializeField]
    CinemachineCamera _camera = null!;

    [SerializeField]
    float _yRotation = 0f;

    [SerializeField]
    AnimationCurve _mapToSiteBlendCurve = new(
      new Keyframe(0f, 0f, 0f, 4f),
      new Keyframe(0.2f, 0.85f, 0f, 0f),
      new Keyframe(1f, 1f, 0f, 0f)
    );

    [SerializeField]
    AnimationCurve _siteToMapBlendCurve = new(
      new Keyframe(0f, 0f, 0f, 0f),
      new Keyframe(0.75f, 0.2f, 0f, 0f),
      new Keyframe(1f, 1f, 0f, 0f)
    );

    [SerializeField]
    float _mapToSiteBlendDuration = 2f;

    [SerializeField]
    float _siteToMapBlendDuration = 2f;

    [SerializeField]
    List<DreamscapeSite> _sites = new();

    readonly Dictionary<DreamscapeSite, CanvasButton> _siteButtonsBySite = new();
    readonly Dictionary<DreamscapeSite, Vector2> _siteButtonPositions = new();

    Coroutine? _transitionRoutine;
    Coroutine? _positionRoutine;
    CinemachineBlenderSettings? _runtimeBlendSettings;
    CinemachineBlenderSettings.CustomBlend[]? _baseCustomBlends;
    bool _siteButtonsVisible;
    bool _initialFramingComplete;
    bool _hasCachedSiteButtonPositions;
    DreamscapeSite? _activeSite;
    Transform? _defaultFollowTarget;
    Transform? _defaultLookAtTarget;

    public CinemachineCamera Camera => _camera;

    public bool IsTransitioning => _transitionRoutine != null;

    public void ActivateWithTransition()
    {
      if (_camera == null)
      {
        throw new InvalidOperationException("Map camera is not assigned.");
      }

      // All of this logic exists to make the site <-> map camera
      // transition keep the site centered through the blend.
      // It's not really necessary, but I find it satisfying to watch.
      if (_transitionRoutine != null)
      {
        StopCoroutine(_transitionRoutine);
      }

      FrameSites();
      HideSiteButtons();
      _transitionRoutine = StartCoroutine(TransitionToMap());
    }

    public void FrameSites()
    {
      if (_camera == null)
      {
        throw new InvalidOperationException("Map camera is not assigned.");
      }

      var viewport = ResolveViewport();
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
        if (!site.gameObject.scene.isLoaded || !site.gameObject.activeInHierarchy)
        {
          continue;
        }

        positions.Add(site.transform.position);
      }

      if (positions.Count == 0)
      {
        throw new InvalidOperationException("No dreamscape sites found to frame.");
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
      PositionSiteButtons();
    }

    public IEnumerator FocusSite(DreamscapeSite site)
    {
      if (site == null)
      {
        throw new InvalidOperationException("Site is required.");
      }
      HideSiteButtons();
      if (_transitionRoutine != null)
      {
        StopCoroutine(_transitionRoutine);
      }
      _transitionRoutine = StartCoroutine(TransitionToSite(site));
      yield return _transitionRoutine;
    }

    public void HideSiteButtons()
    {
      if (!_siteButtonsVisible)
      {
        return;
      }
      _siteButtonsVisible = false;
      foreach (var button in _siteButtonsBySite.Values)
      {
        if (button != null)
        {
          button.gameObject.SetActive(false);
        }
      }
    }

    public void ShowSiteButtons()
    {
      if (_siteButtonsVisible)
      {
        return;
      }
      _siteButtonsVisible = true;
      EnsureSiteButtons();
      PositionSiteButtons();
      foreach (var button in _siteButtonsBySite.Values)
      {
        if (button != null)
        {
          button.gameObject.SetActive(true);
        }
      }
    }

    protected override void OnInitialize()
    {
      CacheDefaultTargets();
      ValidateBlendCurves();
    }

    protected override void OnStart()
    {
      FrameSites();
      EnsureSiteButtons();
      PositionSiteButtons();
      SchedulePositionSiteButtons();
      ShowSiteButtons();
      _camera.Priority = MapPriority;
    }

    void SetLensFieldOfView()
    {
      var lens = _camera.Lens;
      lens.FieldOfView = 60f;
      _camera.Lens = lens;
    }

    IEnumerator TransitionToSite(DreamscapeSite site)
    {
      var brain = ResolveBrain();
      var viewport = ResolveViewport();
      DeactivateOtherSites(site);
      _activeSite = site;
      var siteCamera = site.ActivateForViewport(viewport, SitePriority);
      ApplyBlendSettings(siteCamera);
      SetMapCameraFocus(site.transform);
      _camera.Priority = MapPriority;
      yield return WaitForBlend(brain, siteCamera);
      _transitionRoutine = null;
    }

    IEnumerator TransitionToMap()
    {
      var brain = ResolveBrain();
      var activeSite = GetActiveSite();
      if (activeSite == null)
      {
        RestoreMapCameraFocus();
        _camera.Priority = MapPriority;
        ShowSiteButtons();
        _transitionRoutine = null;
        yield break;
      }

      var viewport = ResolveViewport();
      var siteCamera = activeSite.GetActiveCamera(viewport);
      ApplyBlendSettings(siteCamera);
      SetMapCameraFocus(activeSite.transform);
      _camera.Priority = ReturnMapPriority;
      yield return WaitForBlend(brain, _camera);
      DeactivateAllSites();
      RestoreMapCameraFocus();
      _camera.Priority = MapPriority;
      ShowSiteButtons();
      _transitionRoutine = null;
    }

    CinemachineBrain ResolveBrain()
    {
      var brain = Registry.CinemachineBrain;
      if (brain == null)
      {
        throw new InvalidOperationException("CinemachineBrain is not available.");
      }
      return brain;
    }

    void CacheDefaultTargets()
    {
      if (_camera == null)
      {
        throw new InvalidOperationException("Map camera is not assigned.");
      }
      _defaultFollowTarget = _camera.Follow;
      _defaultLookAtTarget = _camera.LookAt;
    }

    void SetMapCameraFocus(Transform target)
    {
      if (target == null)
      {
        throw new InvalidOperationException("Focus target is required.");
      }
      _camera.Follow = target;
      _camera.LookAt = target;
    }

    void RestoreMapCameraFocus()
    {
      _camera.Follow = _defaultFollowTarget;
      _camera.LookAt = _defaultLookAtTarget;
    }

    void ValidateBlendCurves()
    {
      if (_mapToSiteBlendCurve == null)
      {
        throw new InvalidOperationException("Map to site blend curve is not set.");
      }
      if (_siteToMapBlendCurve == null)
      {
        throw new InvalidOperationException("Site to map blend curve is not set.");
      }
      if (_mapToSiteBlendDuration < Mathf.Epsilon)
      {
        throw new InvalidOperationException("Map to site blend duration must be positive.");
      }
      if (_siteToMapBlendDuration < Mathf.Epsilon)
      {
        throw new InvalidOperationException("Site to map blend duration must be positive.");
      }
    }

    void ApplyBlendSettings(CinemachineCamera siteCamera)
    {
      if (siteCamera == null)
      {
        throw new InvalidOperationException("Site camera is not available.");
      }
      var brain = ResolveBrain();
      _baseCustomBlends ??=
        brain.CustomBlends != null && brain.CustomBlends.CustomBlends != null
          ? (CinemachineBlenderSettings.CustomBlend[])brain.CustomBlends.CustomBlends.Clone()
          : Array.Empty<CinemachineBlenderSettings.CustomBlend>();

      var blends = new List<CinemachineBlenderSettings.CustomBlend>(_baseCustomBlends);
      UpsertBlend(
        blends,
        _camera.Name,
        siteCamera.Name,
        CreateBlendDefinition(_mapToSiteBlendCurve, _mapToSiteBlendDuration)
      );
      UpsertBlend(
        blends,
        siteCamera.Name,
        _camera.Name,
        CreateBlendDefinition(_siteToMapBlendCurve, _siteToMapBlendDuration)
      );

      _runtimeBlendSettings ??= ScriptableObject.CreateInstance<CinemachineBlenderSettings>();
      _runtimeBlendSettings.CustomBlends = blends.ToArray();
      brain.CustomBlends = _runtimeBlendSettings;
    }

    static CinemachineBlendDefinition CreateBlendDefinition(AnimationCurve curve, float duration)
    {
      var definition = new CinemachineBlendDefinition(
        CinemachineBlendDefinition.Styles.Custom,
        duration
      );
      definition.CustomCurve = curve;
      return definition;
    }

    static void UpsertBlend(
      List<CinemachineBlenderSettings.CustomBlend> blends,
      string from,
      string to,
      CinemachineBlendDefinition blend
    )
    {
      var replacement = new CinemachineBlenderSettings.CustomBlend
      {
        From = from,
        To = to,
        Blend = blend,
      };
      var index = -1;
      for (var i = 0; i < blends.Count; i++)
      {
        var existing = blends[i];
        if (existing.From == from && existing.To == to)
        {
          index = i;
          break;
        }
      }

      if (index >= 0)
      {
        blends[index] = replacement;
        return;
      }
      blends.Add(replacement);
    }

    IEnumerator WaitForBlend(CinemachineBrain brain, ICinemachineCamera target)
    {
      var elapsed = 0f;
      while (brain.IsBlending || !CinemachineCore.IsLive(target))
      {
        elapsed += Time.deltaTime;
        if (elapsed > BlendTimeoutSeconds)
        {
          throw new InvalidOperationException($"Camera blend to {target.Name} timed out.");
        }
        yield return null;
      }
    }

    DreamscapeSite? GetActiveSite()
    {
      if (_activeSite != null && _activeSite.IsActive)
      {
        return _activeSite;
      }
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
          site.Deactivate();
        }
      }
      _activeSite = null;
    }

    void DeactivateOtherSites(DreamscapeSite site)
    {
      for (var i = 0; i < _sites.Count; i++)
      {
        var other = _sites[i];
        if (other != null && other != site)
        {
          other.Deactivate();
        }
      }
    }

    void EnsureSiteButtons()
    {
      if (_sites.Count == 0)
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
          button = service.CreateOpenSiteButton();
          _siteButtonsBySite[site] = button;
          button.Initialize(Registry, Mode, TestConfiguration);
          button.StartFromRegistry();
        }
        button.gameObject.SetActive(true);
        var action = string.IsNullOrEmpty(site.DebugClickAction)
          ? new OnClickUnion { Enum = GameActionEnum.NoOp }
          : new OnClickUnion
          {
            OnClickClass = new OnClickClass
            {
              DebugAction = new DebugAction
              {
                DebugActionClass = new DebugActionClass
                {
                  ApplyTestScenarioAction = site.DebugClickAction,
                },
              },
            },
          };
        button.SetView(new ButtonView { Action = action, Label = site.ButtonLabel });
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
        throw new InvalidOperationException("Viewport is not available.");
      }

      if (
        _hasCachedSiteButtonPositions
        && _siteButtonPositions.Count == _siteButtonsBySite.Count
        && _siteButtonPositions.Count > 0
      )
      {
        foreach (var kvp in _siteButtonPositions)
        {
          if (_siteButtonsBySite.TryGetValue(kvp.Key, out var cachedButton) && cachedButton != null)
          {
            var rect = cachedButton.GetComponent<RectTransform>();
            rect.anchoredPosition = kvp.Value;
          }
        }
        return;
      }

      EnsureSiteButtons();
      var worldPositions = new List<Vector3>(_sites.Count);
      var buttonRects = new List<RectTransform>(_sites.Count);
      var orderedSites = new List<DreamscapeSite>(_sites.Count);
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
        orderedSites.Add(site);
      }

      if (
        worldPositions.Count == 0
        || buttonRects.Count != worldPositions.Count
        || orderedSites.Count != worldPositions.Count
      )
      {
        return;
      }

      var positioner = new DreamscapeSiteButtonPositioner(viewport, Registry.CanvasSafeArea);
      var resolved = positioner.PositionButtons(worldPositions, buttonRects);
      if (!_initialFramingComplete)
      {
        return;
      }
      for (var i = 0; i < resolved.Count; i++)
      {
        _siteButtonPositions[orderedSites[i]] = resolved[i];
      }
      _hasCachedSiteButtonPositions = _siteButtonPositions.Count == orderedSites.Count;
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
      _initialFramingComplete = true;
      PositionSiteButtons();
      _positionRoutine = null;
    }

    IGameViewport ResolveViewport()
    {
      var viewport = Application.isPlaying ? Registry.GameViewport : RealViewport.CreateForEditor();
      return viewport ?? throw new InvalidOperationException("Viewport is not available.");
    }
  }
}
