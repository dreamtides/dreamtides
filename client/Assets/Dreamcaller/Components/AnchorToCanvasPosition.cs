#nullable enable

using System.Collections;
using Dreamcaller.Services;
using Dreamcaller.Utils;
using UnityEngine;

namespace Dreamcaller.Components
{
  public class AnchorToCanvasPosition : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] RectTransform _rectTransform = null!;
    [SerializeField] float _distanceFromCamera = 10.0f;

    public IEnumerator Start()
    {
      // Wait for GameCamera field of view changes to stabilize.
      yield return new WaitForSeconds(0.1f);
      var screenPoint = TransformUtils.RectTransformToScreenSpace(_rectTransform).center;
      var anchor = _registry.Layout.MainCamera.ScreenToWorldPoint(
          new Vector3(screenPoint.x, screenPoint.y, _distanceFromCamera));
      transform.position = anchor;
    }
  }
}