#nullable enable

using System.Runtime.CompilerServices;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Layout
{
  public class ForwardPileObjectLayout : StandardObjectLayout
  {
    [SerializeField]
    internal float _singleElementOffset = 0.5f;

    [SerializeField]
    internal float _offsetMultiplier = 1.0f;

    [SerializeField]
    internal bool _flip;

    public override Vector3 CalculateObjectPosition(int index, int count) =>
      transform.position
      + transform.forward * (_offsetMultiplier * Mathf.Lerp(0f, 1f, PileOffset(index, count)));

    public override Vector3? CalculateObjectRotation(int index, int count)
    {
      var eulerAngles = transform.rotation.eulerAngles;
      if (_flip)
      {
        eulerAngles.y += 180f;
      }
      return eulerAngles;
    }

    public override float? CalculateObjectScale(int index, int count) => transform.localScale.x;

    float PileOffset(int index, int count) =>
      count switch
      {
        _ when index >= count => 0.65f,
        0 => _singleElementOffset,
        1 => _singleElementOffset,
        2 => new[] { 0.4f, 0.6f }[index],
        3 => new[] { 0.4f, 0.5f, 0.6f }[index],
        4 => new[] { 0.40f, 0.45f, 0.50f, 0.55f }[index],
        5 => new[] { 0.40f, 0.45f, 0.50f, 0.55f, 0.6f }[index],
        6 => new[] { 0.40f, 0.45f, 0.50f, 0.55f, 0.6f, 0.65f }[index],
        _ => index / ((float)count - 1),
      };
  }
}
