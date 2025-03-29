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
  }
}