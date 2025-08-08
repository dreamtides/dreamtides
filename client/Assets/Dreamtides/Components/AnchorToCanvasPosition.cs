#nullable enable

using System.Collections;
using Dreamtides.Layout;
using Dreamtides.Services;
using Dreamtides.Utils;
using UnityEngine;

namespace Dreamtides.Components
{
  public class AnchorToCanvasPosition : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] RectTransform _rectTransform = null!;
    [SerializeField] float _distanceFromCamera = 10.0f;
    [SerializeField] bool _xCoordinateOnly;
    [SerializeField] ObjectLayout? _toUpdate;
    [SerializeField] float _xOffset;

    public IEnumerator Start()
    {
      // Wait for GameCamera field of view changes to stabilize.
      yield return new WaitForEndOfFrame();
      var screenPoint = TransformUtils.RectTransformToScreenSpace(_rectTransform).center;
      var anchor = _registry.Layout.MainCamera.ScreenToWorldPoint(
          new Vector3(screenPoint.x, screenPoint.y, _distanceFromCamera));

      if (_xCoordinateOnly)
      {
        transform.position = new Vector3(anchor.x + _xOffset, transform.position.y, transform.position.z);
      }
      else
      {
        transform.position = new Vector3(anchor.x + _xOffset, anchor.y, anchor.z);
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