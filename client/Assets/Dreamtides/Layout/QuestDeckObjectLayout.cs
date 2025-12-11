#nullable enable

using System.Runtime.CompilerServices;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Layout
{
  /// <summary>
  /// A RenderAsChildObjectLayout that positions children in a pile-like
  /// arrangement along the Z-axis (depth). Used for the quest deck which
  /// needs children as transform children.
  /// </summary>
  public sealed class QuestDeckObjectLayout : RenderAsChildObjectLayout
  {
    [SerializeField]
    internal float _singleElementY = 0.5f;

    [SerializeField]
    internal float _yMultiplier = 1.0f;

    public override Vector3 CalculateObjectLocalPosition(int index, int count) =>
      new(0f, 0f, _yMultiplier * Mathf.Lerp(0f, 1f, YPosition(index, count)));

    public override Vector3? CalculateObjectLocalRotation(int index, int count) => Vector3.zero;

    public override float? CalculateObjectLocalScale(int index, int count) => 1.0f;

    float YPosition(int index, int count) =>
      count switch
      {
        _ when index >= count => 0.65f,
        0 => _singleElementY,
        1 => _singleElementY,
        2 => new[] { 0.4f, 0.6f }[index],
        3 => new[] { 0.4f, 0.5f, 0.6f }[index],
        4 => new[] { 0.40f, 0.45f, 0.50f, 0.55f }[index],
        5 => new[] { 0.40f, 0.45f, 0.50f, 0.55f, 0.6f }[index],
        6 => new[] { 0.40f, 0.45f, 0.50f, 0.55f, 0.6f, 0.65f }[index],
        _ => index / ((float)count - 1),
      };
  }
}
