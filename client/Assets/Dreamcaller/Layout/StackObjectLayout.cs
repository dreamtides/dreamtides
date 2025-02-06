#nullable enable


using Dreamcaller.Utils;
using UnityEngine;

namespace Dreamcaller.Layout
{
  public class StackObjectLayout : ObjectLayout
  {
    [SerializeField] float _singleElementY = 0.5f;

    public override Vector3 CalculateObjectPosition(int index, int count) =>
      new(
        transform.position.x,
        transform.position.y + Mathf.Lerp(0f, 1f, YPosition(index, count)),
        transform.position.z);

    public override Vector3? CalculateObjectRotation(int index, int count) => transform.rotation.eulerAngles;

    float YPosition(int index, int count) => count switch
    {
      0 => _singleElementY,
      1 => _singleElementY,
      2 => new[] { 0.4f, 0.6f }[index],
      3 => new[] { 0.4f, 0.5f, 0.6f }[index],
      4 => new[] { 0.40f, 0.45f, 0.50f, 0.55f }[index],
      5 => new[] { 0.40f, 0.45f, 0.50f, 0.55f, 0.6f }[index],
      6 => new[] { 0.40f, 0.45f, 0.50f, 0.55f, 0.6f, 0.65f }[index],
      _ => index / ((float)count - 1)
    };
  }
}
