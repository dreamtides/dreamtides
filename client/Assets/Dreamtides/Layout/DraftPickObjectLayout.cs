#nullable enable

using Dreamtides.Services;
using UnityEngine;

namespace Dreamtides.Layout
{
  public sealed class DraftPickObjectLayout : StandardObjectLayout
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] float _horizontalSpacing;
    [SerializeField] float _verticalSpacing;
    [SerializeField] float _cardWidth;
    [SerializeField] float _cardHeight;

    public override Vector3 CalculateObjectPosition(int index, int count)
    {
      var isLandscape = _registry.IsLandscape;
      if (count <= 0)
      {
        return transform.position;
      }
      else if (isLandscape)
      {
        var localX = ComputeHorizontalOffset(index, count, _horizontalSpacing);
        return transform.position + transform.right * localX;
      }
      else
      {
        var topRowCount = (count + 1) / 2;
        var bottomRowCount = count - topRowCount;

        var isTopRow = index < topRowCount;
        var indexInRow = isTopRow ? index : index - topRowCount;
        var rowCount = isTopRow ? topRowCount : bottomRowCount;

        var localX = ComputeHorizontalOffset(indexInRow, rowCount, _horizontalSpacing);

        float localY = count <= 1 ? 0f : (isTopRow ? _verticalSpacing / 2f : -_verticalSpacing / 2f);

        return transform.position + transform.right * localX + transform.up * localY;
      }
    }

    public override Vector3? CalculateObjectRotation(int index, int count) =>
        transform.rotation.eulerAngles;

    public override float? CalculateObjectScale(int index, int count) => transform.localScale.x;

    static float ComputeHorizontalOffset(int indexInRow, int rowCount, float spacing)
    {
      if (rowCount <= 1)
      {
        return 0f;
      }
      var totalWidth = spacing * (rowCount - 1);
      return -totalWidth / 2f + indexInRow * spacing;
    }

    void OnDrawGizmosSelected()
    {
      Gizmos.color = Color.blue;
      var center = transform.position;
      var halfLayoutX = _cardWidth / 2f + _horizontalSpacing / 2f;
      var halfLayoutY = _cardHeight / 2f + _verticalSpacing / 2f;

      var right = transform.right;
      var upAxis = transform.up;

      Gizmos.DrawSphere(center, 0.15f);
      Gizmos.DrawSphere(center + (-right * halfLayoutX + upAxis * halfLayoutY), 0.15f);
      Gizmos.DrawSphere(center + (right * halfLayoutX + upAxis * halfLayoutY), 0.15f);
      Gizmos.DrawSphere(center + (-right * halfLayoutX - upAxis * halfLayoutY), 0.15f);
      Gizmos.DrawSphere(center + (right * halfLayoutX - upAxis * halfLayoutY), 0.15f);
    }
  }
}