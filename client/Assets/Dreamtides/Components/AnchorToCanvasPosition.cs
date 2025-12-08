#nullable enable

using System.Runtime.CompilerServices;
using Dreamtides.Layout;
using Dreamtides.Services;
using Dreamtides.Utils;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Components
{
  public class AnchorToCanvasPosition : MonoBehaviour
  {
    [SerializeField]
    internal Registry _registry = null!;

    [SerializeField]
    internal RectTransform _rectTransform = null!;

    [SerializeField]
    internal float _distanceFromCamera = 10.0f;

    [SerializeField]
    internal bool _xCoordinateOnly;

    [SerializeField]
    internal ObjectLayout? _toUpdate;

    [SerializeField]
    internal float _xOffset;

    int _frameCount;

    void Update()
    {
      _frameCount++;

      if (Registry.TestConfiguration != null && _frameCount < 3)
      {
        return;
      }

      var screenPoint = TransformUtils.RectTransformToScreenSpace(_rectTransform).center;
      var anchor = _registry.MainCamera.ScreenToWorldPoint(
        new Vector3(screenPoint.x, screenPoint.y, _distanceFromCamera)
      );

      var cameraRight = _registry.MainCamera.transform.right;
      var offsetVector = cameraRight * _xOffset;

      if (_xCoordinateOnly)
      {
        var newPosition = anchor + offsetVector;
        transform.position = new Vector3(newPosition.x, transform.position.y, newPosition.z);
      }
      else
      {
        transform.position = anchor + offsetVector;
      }

      if (GetComponent<ObjectLayout>() is { } layout)
      {
        layout.ApplyLayout();
      }
      if (_toUpdate)
      {
        _toUpdate.ApplyLayout();
      }
    }
  }
}
