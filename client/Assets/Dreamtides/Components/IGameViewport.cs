#nullable enable

using System;
using UnityEngine;

namespace Dreamtides.Components
{
  /// <summary>
  /// Interface for interacting with a camera's viewport which allows for a different implementation
  /// to be used in unit tests.
  /// </summary>
  public interface IGameViewport
  {
    Vector3 WorldToViewportPoint(Vector3 worldPosition);

    Vector3 WorldToScreenPoint(Vector3 worldPosition);

    Vector3 ScreenToWorldPoint(Vector3 position);
  }

  public sealed class RealCamera : IGameViewport
  {
    readonly Camera _camera;

    public RealCamera(Camera camera)
    {
      _camera = camera;
    }

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
  }

  /// <summary>
  /// A fake camera that can be used in unit tests to simulate different screen
  /// resolutions.
  /// </summary>
  public sealed class FakeCamera : IGameViewport
  {
    readonly Transform _cameraTransform;
    readonly Vector2 _screenResolution;
    readonly float _aspectRatio;
    readonly float _tanHalfVerticalFov;

    public FakeCamera(Vector2 screenResolution, Transform cameraTransform, float fieldOfView)
    {
      if (screenResolution.x <= 0f || screenResolution.y <= 0f)
      {
        throw new ArgumentOutOfRangeException(nameof(screenResolution));
      }

      if (cameraTransform == null)
      {
        throw new ArgumentNullException(nameof(cameraTransform));
      }

      _cameraTransform = cameraTransform;
      _screenResolution = screenResolution;

      if (fieldOfView <= 0f || fieldOfView >= 180f)
      {
        throw new ArgumentOutOfRangeException(nameof(fieldOfView));
      }

      _aspectRatio = screenResolution.x / screenResolution.y;
      _tanHalfVerticalFov = Mathf.Tan(fieldOfView * Mathf.Deg2Rad * 0.5f);
    }

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
      var x = (xNormalized * 0.5f + 0.5f) * _screenResolution.x;
      var y = (yNormalized * 0.5f + 0.5f) * _screenResolution.y;
      return new Vector3(x, y, viewPosition.z);
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
  }
}
