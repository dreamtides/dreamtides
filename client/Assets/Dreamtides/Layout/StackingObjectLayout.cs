#nullable enable

using System.Runtime.CompilerServices;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Layout
{
  public class StackingObjectLayout : StandardObjectLayout
  {
    [SerializeField]
    internal float _offset;

    [SerializeField]
    internal float _shrinkOffset;

    [SerializeField]
    internal int _shrinkOffsetThreshold;

    [SerializeField]
    internal bool _stackRight;

    public override Vector3 CalculateObjectPosition(int index, int count)
    {
      var offsetAmount = _stackRight
        ? index * GetOffset(count)
        : (index - (count - 1)) * GetOffset(count);
      return transform.position + transform.right * offsetAmount;
    }

    public override Vector3? CalculateObjectRotation(int index, int count) =>
      transform.rotation.eulerAngles;

    public override float? CalculateObjectScale(int index, int count) => transform.localScale.x;

    float GetOffset(int count)
    {
      if (count < _shrinkOffsetThreshold)
      {
        return _offset;
      }
      else
      {
        return _shrinkOffset - (count - _shrinkOffsetThreshold) * 0.05f;
      }
    }
  }
}
