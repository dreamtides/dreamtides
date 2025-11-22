#nullable enable

using Dreamtides.Components;
using Dreamtides.Services;
using UnityEngine;

namespace Dreamtides.Layout
{
  public enum SceneElementScreenAnchor
  {
    TopLeft,

    TopCenter,

    TopRight,

    MiddleLeft,

    // Centered within the left half of the screen
    MiddleLeftHalf,

    MiddleCenter,

    MiddleRight,

    // Centered within the right half of the screen
    MiddleRightHalf,

    // Centered within the bottom half of the screen
    MiddleBottomHalf,

    // Centered within the top half of the screen
    MiddleTopHalf,

    BottomLeft,

    BottomCenter,

    BottomRight,
  }

  public class SceneElementScreenPosition : SceneElement
  {
    [SerializeField]
    GameMode _gameMode = GameMode.Quest;

    [SerializeField]
    SceneElementScreenAnchor _anchor = SceneElementScreenAnchor.MiddleCenter;

    [SerializeField]
    float _xOffset = 0;

    [SerializeField]
    float _yOffset = 0;

    [SerializeField]
    float _distanceFromCamera = 5;

    [SerializeField]
    SceneElementScreenAnchor _landscapeAnchor = SceneElementScreenAnchor.MiddleCenter;

    [SerializeField]
    float _landscapeXOffset = 0;

    [SerializeField]
    float _landscapeYOffset = 0;

    [SerializeField]
    float _landscapeDistanceFromCamera = 0;

    [SerializeField]
    bool _useLandscapeAnchor = false;

    [SerializeField]
    bool _useLandscapeXOffset = false;

    [SerializeField]
    bool _useLandscapeYOffset = false;

    [SerializeField]
    bool _useLandscapeDistanceFromCamera = false;

    [SerializeField]
    bool _ignoreSafeArea = false;

    bool _validationInitialized = false;

    SceneElementScreenAnchor _prevLandscapeAnchor;
    float _prevLandscapeXOffset;
    float _prevLandscapeYOffset;
    float _prevLandscapeDistance;

    static Vector2 AnchorToScreenPoint(Rect rect, SceneElementScreenAnchor anchor)
    {
      var centerX = rect.xMin + rect.width * 0.5f;
      var centerY = rect.yMin + rect.height * 0.5f;
      switch (anchor)
      {
        case SceneElementScreenAnchor.TopLeft:
          return new Vector2(rect.xMin, rect.yMax);
        case SceneElementScreenAnchor.TopCenter:
          return new Vector2(centerX, rect.yMax);
        case SceneElementScreenAnchor.TopRight:
          return new Vector2(rect.xMax, rect.yMax);
        case SceneElementScreenAnchor.MiddleLeft:
          return new Vector2(rect.xMin, centerY);
        case SceneElementScreenAnchor.MiddleLeftHalf:
          return new Vector2(rect.xMin + rect.width * 0.25f, centerY);
        case SceneElementScreenAnchor.MiddleCenter:
          return new Vector2(centerX, centerY);
        case SceneElementScreenAnchor.MiddleRight:
          return new Vector2(rect.xMax, centerY);
        case SceneElementScreenAnchor.MiddleRightHalf:
          return new Vector2(rect.xMin + rect.width * 0.75f, centerY);
        case SceneElementScreenAnchor.MiddleBottomHalf:
          return new Vector2(centerX, rect.yMin + rect.height * 0.25f);
        case SceneElementScreenAnchor.MiddleTopHalf:
          return new Vector2(centerX, rect.yMin + rect.height * 0.75f);
        case SceneElementScreenAnchor.BottomLeft:
          return new Vector2(rect.xMin, rect.yMin);
        case SceneElementScreenAnchor.BottomCenter:
          return new Vector2(centerX, rect.yMin);
        case SceneElementScreenAnchor.BottomRight:
          return new Vector2(rect.xMax, rect.yMin);
      }
      return new Vector2(centerX, centerY);
    }

    static Vector3 ComputeWorldPosition(
      IGameViewport gameViewport,
      Rect rect,
      SceneElementScreenAnchor anchor,
      float xOffset,
      float yOffset,
      float distance
    )
    {
      var anchorPoint = AnchorToScreenPoint(rect, anchor);
      var screenPoint = new Vector3(anchorPoint.x + xOffset, anchorPoint.y + yOffset, distance);
      return gameViewport.ScreenToWorldPoint(screenPoint);
    }

    static Rect ComputeFullScreenRect(IGameViewport gameViewport)
    {
      return new Rect(0f, 0f, gameViewport.ScreenWidth, gameViewport.ScreenHeight);
    }

    Rect ComputeSafeAreaScreenRect(IGameViewport gameViewport)
    {
      var pixelRect = gameViewport.CanvasPixelRect;
      var min = gameViewport.SafeAreaMinimumAnchor;
      var max = gameViewport.SafeAreaMaximumAnchor;
      var xMin = pixelRect.x + pixelRect.width * min.x;
      var xMax = pixelRect.x + pixelRect.width * max.x;
      var yMin = pixelRect.y + pixelRect.height * min.y;
      var yMax = pixelRect.y + pixelRect.height * max.y;
      return Rect.MinMaxRect(xMin, yMin, xMax, yMax);
    }

    void OnValidate()
    {
      if (!_validationInitialized)
      {
        _prevLandscapeAnchor = _landscapeAnchor;
        _prevLandscapeXOffset = _landscapeXOffset;
        _prevLandscapeYOffset = _landscapeYOffset;
        _prevLandscapeDistance = _landscapeDistanceFromCamera;
        _validationInitialized = true;
        return;
      }

      if (_landscapeAnchor != _prevLandscapeAnchor)
      {
        _useLandscapeAnchor = true;
      }
      if (!Mathf.Approximately(_landscapeXOffset, _prevLandscapeXOffset))
      {
        _useLandscapeXOffset = true;
      }
      if (!Mathf.Approximately(_landscapeYOffset, _prevLandscapeYOffset))
      {
        _useLandscapeYOffset = true;
      }
      if (!Mathf.Approximately(_landscapeDistanceFromCamera, _prevLandscapeDistance))
      {
        _useLandscapeDistanceFromCamera = true;
      }

      if (!_useLandscapeAnchor)
      {
        _landscapeAnchor = SceneElementScreenAnchor.MiddleCenter;
      }
      if (!_useLandscapeXOffset)
      {
        _landscapeXOffset = 0f;
      }
      if (!_useLandscapeYOffset)
      {
        _landscapeYOffset = 0f;
      }
      if (!_useLandscapeDistanceFromCamera)
      {
        _landscapeDistanceFromCamera = 0f;
      }

      _prevLandscapeAnchor = _landscapeAnchor;
      _prevLandscapeXOffset = _landscapeXOffset;
      _prevLandscapeYOffset = _landscapeYOffset;
      _prevLandscapeDistance = _landscapeDistanceFromCamera;
    }

    protected override void OnUpdate(GameMode mode, TestConfiguration? testConfiguration)
    {
      if (mode != _gameMode)
      {
        if (gameObject.activeSelf)
        {
          gameObject.SetActive(false);
        }
        return;
      }

      if (!gameObject.activeSelf)
      {
        gameObject.SetActive(true);
      }

      var viewport = Registry.GameViewport;
      var isLandscape = viewport.IsLandscape;
      var anchor = isLandscape && _useLandscapeAnchor ? _landscapeAnchor : _anchor;
      var xOffset = isLandscape && _useLandscapeXOffset ? _landscapeXOffset : _xOffset;
      var yOffset = isLandscape && _useLandscapeYOffset ? _landscapeYOffset : _yOffset;
      var distance =
        isLandscape && _useLandscapeDistanceFromCamera
          ? _landscapeDistanceFromCamera
          : _distanceFromCamera;

      var rect = _ignoreSafeArea
        ? ComputeFullScreenRect(viewport)
        : ComputeSafeAreaScreenRect(viewport);
      var world = ComputeWorldPosition(viewport, rect, anchor, xOffset, yOffset, distance);
      transform.position = world;
    }

#if UNITY_EDITOR
    void OnDrawGizmos()
    {
      var viewport = Application.isPlaying
        ? new RealViewport(Registry)
        : RealViewport.CreateForEditor();
      if (viewport == null)
      {
        return;
      }

      var isLandscape = viewport.IsLandscape;
      var anchor = isLandscape && _useLandscapeAnchor ? _landscapeAnchor : _anchor;
      var xOffset = isLandscape && _useLandscapeXOffset ? _landscapeXOffset : _xOffset;
      var yOffset = isLandscape && _useLandscapeYOffset ? _landscapeYOffset : _yOffset;
      var distance =
        isLandscape && _useLandscapeDistanceFromCamera
          ? _landscapeDistanceFromCamera
          : _distanceFromCamera;
      var rect = Application.isPlaying
        ? (_ignoreSafeArea ? ComputeFullScreenRect(viewport) : ComputeSafeAreaScreenRect(viewport))
        : ComputeFullScreenRect(viewport);
      var baseWorld = ComputeWorldPosition(viewport, rect, anchor, 0f, 0f, distance);
      var world = ComputeWorldPosition(viewport, rect, anchor, xOffset, yOffset, distance);
      Gizmos.color = Color.yellow;
      Gizmos.DrawSphere(baseWorld, 0.05f);
      var bl = viewport.ScreenToWorldPoint(new Vector3(rect.xMin, rect.yMin, distance));
      var br = viewport.ScreenToWorldPoint(new Vector3(rect.xMax, rect.yMin, distance));
      var tl = viewport.ScreenToWorldPoint(new Vector3(rect.xMin, rect.yMax, distance));
      var tr = viewport.ScreenToWorldPoint(new Vector3(rect.xMax, rect.yMax, distance));
      Gizmos.DrawSphere(bl, 0.05f);
      Gizmos.DrawSphere(br, 0.05f);
      Gizmos.DrawSphere(tl, 0.05f);
      Gizmos.DrawSphere(tr, 0.05f);
      Gizmos.color = Color.red;
      Gizmos.DrawSphere(world, 0.05f);
    }

    [ContextMenu("Jump To Expected Position (Editor)")]
    void JumpToExpectedPositionEditor()
    {
      var width = Screen.width;
      var height = Screen.height;
      var isLandscape = width > height;
      var anchor = isLandscape && _useLandscapeAnchor ? _landscapeAnchor : _anchor;
      var xOffset = isLandscape && _useLandscapeXOffset ? _landscapeXOffset : _xOffset;
      var yOffset = isLandscape && _useLandscapeYOffset ? _landscapeYOffset : _yOffset;
      var distance =
        isLandscape && _useLandscapeDistanceFromCamera
          ? _landscapeDistanceFromCamera
          : _distanceFromCamera;
      var viewport = RealViewport.CreateForEditor();
      if (viewport != null)
      {
        var rect = ComputeFullScreenRect(viewport);
        var world = ComputeWorldPosition(viewport, rect, anchor, xOffset, yOffset, distance);
        transform.position = world;
      }
    }

#if UNITY_EDITOR
    [UnityEditor.CustomEditor(typeof(SceneElementScreenPosition))]
    class SceneElementScreenPositionEditor : UnityEditor.Editor
    {
      public override void OnInspectorGUI()
      {
        base.OnInspectorGUI();
        var targetComp = (SceneElementScreenPosition)target;
        if (UnityEngine.GUILayout.Button("Jump To Expected Position"))
        {
          targetComp.JumpToExpectedPositionEditor();
        }
      }
    }
#endif
#endif
  }
}
