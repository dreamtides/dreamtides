#nullable enable

using System.Runtime.CompilerServices;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Layout
{
  public class CenteredObjectLayout : StandardObjectLayout
  {
    [SerializeField]
    internal float _width;

    [SerializeField]
    internal float _initialSpacing;

    [SerializeField]
    internal float _cardSize;

    [SerializeField]
    internal bool _vertical;

    public override Vector3 CalculateObjectPosition(int index, int count)
    {
      var offset = CalculateOffset(_width, _initialSpacing, _cardSize, index, count);
      return transform.position
        + (_vertical ? new Vector3(0, 0, offset) : new Vector3(offset, 0, 0));
    }

    public override Vector3? CalculateObjectRotation(int index, int count) =>
      transform.rotation.eulerAngles;

    public override float? CalculateObjectScale(int index, int count) => transform.localScale.x;

    public static float CalculateOffset(
      float width,
      float initialSpacing,
      float itemWidth,
      int index,
      int count,
      float minOffsetMultiplier = 1f,
      float maxOffsetMultiplier = 1f
    )
    {
      var availableWidth = Mathf.Min(width, (itemWidth + initialSpacing) * count);
      var offset = availableWidth / 2f - itemWidth / 2f;

      return count switch
      {
        0 or 1 => 0,
        _ => Mathf.Lerp(
          -offset * minOffsetMultiplier,
          offset * maxOffsetMultiplier,
          index / (count - 1f)
        ),
      };
    }

    void OnDrawGizmosSelected()
    {
      Gizmos.color = Color.blue;
      Gizmos.DrawSphere(
        transform.position
          + (_vertical ? new Vector3(0, 0, _width / 2f) : new Vector3(_width / 2f, 0, 0)),
        radius: 1
      );
      Gizmos.DrawSphere(transform.position, radius: 1);
      Gizmos.DrawSphere(
        transform.position
          + (_vertical ? new Vector3(0, 0, _width / -2f) : new Vector3(_width / -2f, 0, 0)),
        radius: 1
      );
    }
  }
}
