#nullable enable

using UnityEngine;

namespace Dreamcaller.Layout
{
  public sealed class RectangularObjectLayout : ObjectLayout
  {
    [SerializeField] float _width;
    [SerializeField] float _height;
    [SerializeField] float _itemHorizontalSpacing;
    [SerializeField] float _itemVerticalSpacing;
    [SerializeField] float _itemWidth;
    [SerializeField] float _itemHeight;
    [SerializeField] int _rowCount;

    public override Vector3 CalculateObjectPosition(int index, int count) =>
      transform.position + new Vector3(
        CalculateXOffset(index, count),
        0.5f,
        CalculateZOffset(index, count));

    public override Vector3? CalculateObjectRotation(int index, int count) => transform.rotation.eulerAngles;

    float CalculateXOffset(int index, int count)
    {
      return LinearObjectLayout.CalculateOffset(
        _width,
        _itemHorizontalSpacing,
        _itemWidth,
        index % _rowCount,
        count > _rowCount ? _rowCount : count);
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
      Gizmos.DrawSphere(transform.position + new Vector3(_width / 2f, 0, _height / 2f), radius: 0.5f);
      Gizmos.DrawSphere(transform.position + new Vector3(_width / 2f, 0, _height / -2f), radius: 0.5f);
      Gizmos.DrawSphere(transform.position, radius: 0.5f);
      Gizmos.DrawSphere(transform.position + new Vector3(_width / -2f, 0, _height / 2f), radius: 0.5f);
      Gizmos.DrawSphere(transform.position + new Vector3(_width / -2f, 0, _height / -2f), radius: 0.5f);
    }
  }
}