#nullable enable

using System;
using Dreamtides.Services;
using UnityEngine;

namespace Dreamtides.Components
{
  /// <summary>
  /// Interface for interacting with a camera's viewport which allows for a different implementation
  /// to be used in unit tests.
  /// </summary>
  public interface IGameViewport
  {
    /// <summary>
    /// Returns true if the viewport is in landscape mode.
    /// </summary>
    bool IsLandscape { get; }

    /// <summary>
    /// Returns the width of the viewport in screen pixels.
    /// </summary>
    float ScreenWidth { get; }

    /// <summary>
    /// Returns the height of the viewport in screen pixels.
    /// </summary>
    float ScreenHeight { get; }

    /// <summary>
    /// Get the render rect for the Canvas.
    /// </summary>
    Rect CanvasPixelRect { get; }

    /// <summary>
    /// Get the minimum anchor for the screen safe area.
    /// </summary>
    Vector2 SafeAreaMinimumAnchor { get; }

    /// <summary>
    /// Get the maximum anchor for the screen safe area.
    /// </summary>
    Vector2 SafeAreaMaximumAnchor { get; }

    Vector3 WorldToViewportPoint(Vector3 worldPosition);

    Vector3 WorldToScreenPoint(Vector3 worldPosition);

    Vector3 ScreenToWorldPoint(Vector3 position);

    RectTransform CanvasRootRect { get; }
  }

  public sealed class RealViewport : IGameViewport
  {
    readonly bool _isLandscape;
    readonly Camera _camera;
    readonly Canvas _canvas;
    readonly RectTransform _canvasSafeArea;
    readonly RectTransform _canvasRootRect;

    RealViewport(
      bool isLandscape,
      Camera camera,
      Canvas canvas,
      RectTransform canvasSafeArea,
      RectTransform canvasRootRect
    )
    {
      _isLandscape = isLandscape;
      _camera = camera;
      _canvas = canvas;
      _canvasSafeArea = canvasSafeArea;
      _canvasRootRect = canvasRootRect;
    }

    public RealViewport(Registry registry)
      : this(
        registry.IsLandscape,
        registry.MainCamera,
        registry.Canvas,
        registry.CanvasSafeArea,
        registry.Canvas.GetComponent<RectTransform>()
      ) { }

    public static RealViewport? CreateForEditor()
    {
      var camera =
        Camera.main
        ?? UnityEngine.Object.FindFirstObjectByType<Camera>(FindObjectsInactive.Include);
      if (camera == null)
      {
        Debug.LogWarning("Unable to find a Camera in the scene.");
        return null;
      }

      var canvas = UnityEngine.Object.FindFirstObjectByType<Canvas>(FindObjectsInactive.Include);
      if (canvas == null)
      {
        Debug.LogWarning("Unable to find a Canvas in the scene.");
        return null;
      }

      var safeArea = canvas.GetComponent<RectTransform>();
      if (safeArea == null)
      {
        Debug.LogWarning("Canvas is missing a RectTransform component.");
        return null;
      }

      safeArea.anchorMin = Vector2.zero;
      safeArea.anchorMax = Vector2.one;

      var rootRect = canvas.GetComponent<RectTransform>();
      return new RealViewport(Screen.width > Screen.height, camera, canvas, safeArea, rootRect);
    }

    public bool IsLandscape => _isLandscape;

    public float ScreenWidth => Screen.width;

    public float ScreenHeight => Screen.height;

    public Vector2 SafeAreaMinimumAnchor => _canvasSafeArea.anchorMin;

    public Vector2 SafeAreaMaximumAnchor => _canvasSafeArea.anchorMax;

    public Rect CanvasPixelRect => _canvas.pixelRect;

    public Vector3 WorldToViewportPoint(Vector3 worldPosition)
    {
      return _camera.WorldToViewportPoint(worldPosition);
    }

    public Vector3 WorldToScreenPoint(Vector3 worldPosition)
    {
      return _camera.WorldToScreenPoint(worldPosition);
    }

    public Vector3 ScreenToWorldPoint(Vector3 position)
    {
      return _camera.ScreenToWorldPoint(position);
    }

    public RectTransform CanvasRootRect => _canvasRootRect;
  }

  /// <summary>
  /// A fake camera that can be used in unit tests to simulate different screen
  /// resolutions.
  /// </summary>
  public sealed class FakeViewport : IGameViewport
  {
    readonly Transform _cameraTransform;
    readonly Vector2 _screenResolution;
    readonly float _aspectRatio;
    readonly float _tanHalfVerticalFov;
    readonly Rect _canvasPixelRect;
    readonly Vector2 _safeAreaMinimumAnchor;
    readonly Vector2 _safeAreaMaximumAnchor;
    readonly RectTransform _canvasRootRect;

    public FakeViewport(
      Vector2 screenResolution,
      Transform cameraTransform,
      float fieldOfView,
      RectTransform canvasRootRect,
      Rect? canvasPixelRect = null,
      Vector2? safeAreaMinimumAnchor = null,
      Vector2? safeAreaMaximumAnchor = null
    )
    {
      if (screenResolution.x <= 0f || screenResolution.y <= 0f)
      {
        throw new ArgumentOutOfRangeException(nameof(screenResolution));
      }

      if (cameraTransform == null)
      {
        throw new ArgumentNullException(nameof(cameraTransform));
      }

      if (canvasRootRect == null)
      {
        throw new ArgumentNullException(nameof(canvasRootRect));
      }

      _cameraTransform = cameraTransform;
      _screenResolution = screenResolution;

      if (fieldOfView <= 0f || fieldOfView >= 180f)
      {
        throw new ArgumentOutOfRangeException(nameof(fieldOfView));
      }

      _aspectRatio = screenResolution.x / screenResolution.y;
      _tanHalfVerticalFov = Mathf.Tan(fieldOfView * Mathf.Deg2Rad * 0.5f);
      _canvasPixelRect =
        canvasPixelRect ?? new Rect(0f, 0f, screenResolution.x, screenResolution.y);
      _safeAreaMinimumAnchor = safeAreaMinimumAnchor ?? Vector2.zero;
      _safeAreaMaximumAnchor = safeAreaMaximumAnchor ?? Vector2.one;
      _canvasRootRect = canvasRootRect;
    }

    public bool IsLandscape => _screenResolution.x > _screenResolution.y;

    public float ScreenWidth => _screenResolution.x;

    public float ScreenHeight => _screenResolution.y;

    public Vector2 SafeAreaMinimumAnchor => _safeAreaMinimumAnchor;

    public Vector2 SafeAreaMaximumAnchor => _safeAreaMaximumAnchor;

    public Rect CanvasPixelRect => _canvasPixelRect;

    public Vector3 WorldToViewportPoint(Vector3 worldPosition)
    {
      var viewPosition = _cameraTransform.InverseTransformPoint(worldPosition);
      var denominator = viewPosition.z * _tanHalfVerticalFov;
      var xNormalized = viewPosition.x / (denominator * _aspectRatio);
      var yNormalized = viewPosition.y / denominator;
      var x = xNormalized * 0.5f + 0.5f;
      var y = yNormalized * 0.5f + 0.5f;
      return new Vector3(x, y, viewPosition.z);
    }

    public Vector3 WorldToScreenPoint(Vector3 worldPosition)
    {
      var viewPosition = _cameraTransform.InverseTransformPoint(worldPosition);
      var denominator = viewPosition.z * _tanHalfVerticalFov;
      var xNormalized = viewPosition.x / (denominator * _aspectRatio);
      var yNormalized = viewPosition.y / denominator;
      var screenX = (xNormalized * 0.5f + 0.5f) * _screenResolution.x;
      var screenY = (yNormalized * 0.5f + 0.5f) * _screenResolution.y;
      return new Vector3(screenX, screenY, viewPosition.z);
    }

    public Vector3 ScreenToWorldPoint(Vector3 position)
    {
      var viewportX = position.x / _screenResolution.x;
      var viewportY = position.y / _screenResolution.y;
      var xNormalized = (viewportX - 0.5f) * 2f;
      var yNormalized = (viewportY - 0.5f) * 2f;
      var z = position.z;
      var denominator = z * _tanHalfVerticalFov;
      var localX = xNormalized * denominator * _aspectRatio;
      var localY = yNormalized * denominator;
      var localPoint = new Vector3(localX, localY, z);
      return _cameraTransform.TransformPoint(localPoint);
    }

    public RectTransform CanvasRootRect => _canvasRootRect;
  }
}
