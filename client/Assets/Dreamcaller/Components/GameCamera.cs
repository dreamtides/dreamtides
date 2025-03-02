#nullable enable

using System.Collections;
using UnityEngine;

namespace Dreamcaller.Components
{
  public class GameCamera : MonoBehaviour
  {
    [SerializeField] Transform _topLeft = null!;
    [SerializeField] Transform _bottomLeft = null!;
    [SerializeField] float _zoomInThreshold = 0.18f;
    [SerializeField] Camera _camera = null!;

    void Start()
    {
      for (int i = 0; i < 10; i++)
      {
        var topLeft = _camera.WorldToViewportPoint(_bottomLeft.position);
        if (topLeft.x > _zoomInThreshold)
        {
          _camera.fieldOfView -= 1;
        }

        if (topLeft.x < 0)
        {
          _camera.fieldOfView += 1;
        }
      }

      for (int i = 0; i < 10; i++)
      {
        var topLeft = _camera.WorldToViewportPoint(_topLeft.position);
        if (topLeft.x > _zoomInThreshold)
        {
          _camera.fieldOfView -= 1;
        }

        if (topLeft.x < 0)
        {
          _camera.fieldOfView += 1;
        }
      }
    }

  }
}