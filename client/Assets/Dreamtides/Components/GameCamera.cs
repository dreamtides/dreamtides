#nullable enable

using System.Runtime.CompilerServices;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Components
{
  public class GameCamera : MonoBehaviour
  {
    [SerializeField]
    internal float _zoomInThreshold = 0.18f;

    [SerializeField]
    internal Camera _camera = null!;

    public void AdjustFieldOfView(BattleCameraBounds bounds)
    {
      for (int i = 0; i < 25; i++)
      {
        var bottomLeft = _camera.WorldToViewportPoint(bounds.BottomLeftAnchor.position);
        if (bottomLeft.x > _zoomInThreshold)
        {
          _camera.fieldOfView -= 1;
        }
      }

      for (int i = 0; i < 25; i++)
      {
        var topLeft = _camera.WorldToViewportPoint(bounds.TopLeftAnchor.position);
        if (topLeft.x > _zoomInThreshold)
        {
          _camera.fieldOfView -= 1;
        }
      }

      for (int i = 0; i < 25; i++)
      {
        var bottomLeft = _camera.WorldToViewportPoint(bounds.BottomLeftAnchor.position);
        if (bottomLeft.x < 0 || bottomLeft.y < 0)
        {
          _camera.fieldOfView += 1;
        }
      }

      for (int i = 0; i < 25; i++)
      {
        var topLeft = _camera.WorldToViewportPoint(bounds.TopLeftAnchor.position);
        if (topLeft.x < 0 || topLeft.y < 0)
        {
          _camera.fieldOfView += 1;
        }
      }
    }
  }
}
