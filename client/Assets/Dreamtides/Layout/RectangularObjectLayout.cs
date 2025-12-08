#nullable enable

using System.Runtime.CompilerServices;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Layout
{
  public sealed class RectangularObjectLayout : StandardObjectLayout
  {
    [SerializeField]
    internal float _width;

    [SerializeField]
    internal float _height;

    [SerializeField]
    internal float _itemHorizontalSpacing;

    [SerializeField]
    internal float _itemVerticalSpacing;

    [SerializeField]
    internal float _itemWidth;

    [SerializeField]
    internal float _itemHeight;

    [SerializeField]
    internal int _rowCount;

    public override Vector3 CalculateObjectPosition(int index, int count) =>
      transform.position
      + new Vector3(CalculateXOffset(index, count), 0.5f, CalculateZOffset(index, count));

    public override Vector3? CalculateObjectRotation(int index, int count) =>
      transform.rotation.eulerAngles;

    float CalculateXOffset(int index, int count)
    {
      return CenteredObjectLayout.CalculateOffset(
        _width,
        _itemHorizontalSpacing,
        _itemWidth,
        index % _rowCount,
        count > _rowCount ? _rowCount : count
      );
    }

    float CalculateZOffset(int index, int count)
    {
      if (count <= _rowCount)
      {
        return 0f;
      }

      var rowNumber = index / _rowCount;
      var rowHeight = _itemHeight + _itemVerticalSpacing;
      return rowNumber * rowHeight - (_height / 2f);
    }

    void OnDrawGizmosSelected()
    {
      Gizmos.color = Color.blue;
      Gizmos.DrawSphere(
        transform.position + new Vector3(_width / 2f, 0, _height / 2f),
        radius: 0.5f
      );
      Gizmos.DrawSphere(
        transform.position + new Vector3(_width / 2f, 0, _height / -2f),
        radius: 0.5f
      );
      Gizmos.DrawSphere(transform.position, radius: 0.5f);
      Gizmos.DrawSphere(
        transform.position + new Vector3(_width / -2f, 0, _height / 2f),
        radius: 0.5f
      );
      Gizmos.DrawSphere(
        transform.position + new Vector3(_width / -2f, 0, _height / -2f),
        radius: 0.5f
      );
    }
  }
}
