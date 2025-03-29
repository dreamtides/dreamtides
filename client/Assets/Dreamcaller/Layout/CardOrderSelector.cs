#nullable enable

using UnityEngine;

namespace Dreamcaller.Layout
{
  public class CardOrderSelector : AbstractCardBrowser
  {
    [SerializeField] float _initialSpacing = 0.5f;

    protected override Vector3 CalculateObjectPosition(int index, int count)
    {
      var offset = LinearObjectLayout.CalculateOffset(TotalWidth(), _initialSpacing, _cardWidth, index, count);
      return transform.position + new Vector3(offset, 0, 0);
    }

    protected override Vector3? CalculateObjectRotation(int index, int count) => transform.rotation.eulerAngles;

    /// <summary>
    /// Returns the index position within the selector which most closely maps
    /// to the position of the given Transform.
    /// </summary>
    public int HorizontalIndexPositionWithinDisplay(Transform t)
    {
      if (Objects.Count == 0)
      {
        return 0;
      }

      var targetPosition = GetAxisPosition(t);
      if (targetPosition < GetAxisPosition(Objects[0].transform))
      {
        return 0;
      }

      if (targetPosition > GetAxisPosition(Objects[Objects.Count - 1].transform))
      {
        return Objects.Count;
      }

      for (int i = 0; i < Objects.Count - 1; i++)
      {
        var currentPosition = GetAxisPosition(Objects[i].transform);
        var nextPosition = GetAxisPosition(Objects[i + 1].transform);

        if (targetPosition >= currentPosition && targetPosition <= nextPosition)
        {
          return (targetPosition - currentPosition < nextPosition - targetPosition) ? i + 1 : i + 2;
        }
      }

      return 0;
    }
  }
}