#nullable enable

using UnityEngine;

namespace Dreamtides.Layout
{
  public class StackingObjectLayout : StandardObjectLayout
  {
    [SerializeField] float _offset;
    [SerializeField] bool _stackRight;

    protected override Vector3 CalculateObjectPosition(int index, int count)
    {
      var xOffset = _stackRight
          ? index * _offset
          : (index - (count - 1)) * _offset;
      return new Vector3(transform.position.x + xOffset, transform.position.y, transform.position.z);
    }

    protected override Vector3? CalculateObjectRotation(int index, int count) => transform.rotation.eulerAngles;

    protected override float? CalculateObjectScale(int index, int count) => transform.localScale.x;
  }
}