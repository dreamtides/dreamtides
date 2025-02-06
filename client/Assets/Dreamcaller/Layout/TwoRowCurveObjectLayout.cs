#nullable enable

using UnityEngine;

namespace Dreamcaller.Layout
{
  public class TwoRowCurveObjectLayout : ObjectLayout
  {
    [SerializeField] CurveObjectLayout _topRow = null!;
    [SerializeField] CurveObjectLayout _bottomRow = null!;

    public override Vector3 CalculateObjectPosition(int index, int count)
    {
      if (count == 0) return transform.position;
      var (row, adjustedIndex, adjustedCount) = GetRowInfo(index, count);
      return row.CalculateObjectPosition(adjustedIndex, adjustedCount);
    }

    public override Vector3? CalculateObjectRotation(int index, int count)
    {
      if (count == 0) return null;
      var (row, adjustedIndex, adjustedCount) = GetRowInfo(index, count);
      return row.CalculateObjectRotation(adjustedIndex, adjustedCount);
    }

    public override float? CalculateObjectScale(int index, int count)
    {
      if (count == 0) return null;
      var (row, adjustedIndex, adjustedCount) = GetRowInfo(index, count);
      return row.CalculateObjectScale(adjustedIndex, adjustedCount);
    }

    /// <summary>
    /// Helper to select the appropriate row and adjust the index/count for that row.
    /// Returns a tuple of (selected row, adjusted index, adjusted count).
    /// </summary>
    (CurveObjectLayout row, int index, int count) GetRowInfo(int index, int count)
    {
      if (count <= 7)
      {
        return (_topRow, index, count);
      }
      var halfCount = (count + 1) / 2;
      if (index < halfCount)
      {
        return (_topRow, index, halfCount);
      }
      return (_bottomRow, index - halfCount, count - halfCount);
    }
  }
}
