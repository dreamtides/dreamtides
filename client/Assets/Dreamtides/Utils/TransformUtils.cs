#nullable enable

using UnityEngine;

namespace Dreamtides.Utils
{
  public static class TransformUtils
  {
    public static Rect RectTransformToScreenSpace(RectTransform transform)
    {
      var size = Vector2.Scale(transform.rect.size, transform.lossyScale);
      var x = transform.position.x - (transform.pivot.x * size.x);
      var y = transform.position.y - ((1.0f - transform.pivot.y) * size.y);
      return new Rect(x, y, size.x, size.y);
    }
  }
}
