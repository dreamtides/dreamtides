#nullable enable

using System.Runtime.CompilerServices;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Layout
{
  public sealed class DreamsignDisplayLayout : RenderAsChildObjectLayout
  {
    [SerializeField]
    internal float _horizontalSpacing;

    [SerializeField]
    internal float _verticalSpacing;

    [SerializeField]
    internal float _cardWidth;

    [SerializeField]
    internal float _cardHeight;

    public override Vector3 CalculateObjectLocalPosition(int index, int count)
    {
      if (count <= 0)
      {
        return Vector3.zero;
      }

      var column = index / 2;
      var row = index % 2;

      var localX = -_horizontalSpacing * column;
      var localY = count <= 1 ? 0f : (row == 0 ? _verticalSpacing / 2f : -_verticalSpacing / 2f);

      return new Vector3(localX, localY, 0f);
    }

    public override Vector3? CalculateObjectLocalRotation(int index, int count) => Vector3.zero;

    public override float? CalculateObjectLocalScale(int index, int count) => 1.0f;

    void OnDrawGizmosSelected()
    {
      Gizmos.color = Color.blue;
      var center = transform.position;
      var s = transform.lossyScale.x;
      var halfLayoutX = _cardWidth * s / 2f + _horizontalSpacing * s / 2f;
      var halfLayoutY = _cardHeight * s / 2f + _verticalSpacing * s / 2f;

      var right = transform.right;
      var upAxis = transform.up;

      Gizmos.DrawSphere(center, 0.15f * s);
      Gizmos.DrawSphere(center + (-right * halfLayoutX + upAxis * halfLayoutY), 0.15f * s);
      Gizmos.DrawSphere(center + (right * halfLayoutX + upAxis * halfLayoutY), 0.15f * s);
      Gizmos.DrawSphere(center + (-right * halfLayoutX - upAxis * halfLayoutY), 0.15f * s);
      Gizmos.DrawSphere(center + (right * halfLayoutX - upAxis * halfLayoutY), 0.15f * s);
    }
  }
}
