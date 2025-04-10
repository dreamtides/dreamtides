#nullable enable

using UnityEngine;

namespace Dreamtides.Components
{
  public class GameCamera : MonoBehaviour
  {
    [SerializeField] Transform _topLeft = null!;
    [SerializeField] Transform _bottomLeft = null!;
    [SerializeField] float _zoomInThreshold = 0.18f;
    [SerializeField] Camera _camera = null!;

    void Awake()
    {
      for (int i = 0; i < 25; i++)
      {
        var bottomLeft = _camera.WorldToViewportPoint(_bottomLeft.position);
        if (bottomLeft.x > _zoomInThreshold)
        {
          _camera.fieldOfView -= 1;
        }
      }

      for (int i = 0; i < 25; i++)
      {
        var topLeft = _camera.WorldToViewportPoint(_topLeft.position);
        if (topLeft.x > _zoomInThreshold)
        {
          _camera.fieldOfView -= 1;
        }
      }

      for (int i = 0; i < 25; i++)
      {
        var bottomLeft = _camera.WorldToViewportPoint(_bottomLeft.position);
        if (bottomLeft.x < 0 || bottomLeft.y < 0)
        {
          _camera.fieldOfView += 1;
        }
      }

      for (int i = 0; i < 25; i++)
      {
        var topLeft = _camera.WorldToViewportPoint(_topLeft.position);
        if (topLeft.x < 0 || topLeft.y < 0)
        {
          _camera.fieldOfView += 1;
        }
      }
    }

  }
}