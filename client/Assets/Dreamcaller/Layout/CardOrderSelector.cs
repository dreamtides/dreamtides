#nullable enable

using Dreamcaller.Schema;
using Dreamcaller.Utils;
using UnityEngine;

namespace Dreamcaller.Layout
{
  public class CardOrderSelector : AbstractCardBrowser
  {
    [SerializeField] float _initialSpacing = 0.5f;
    [SerializeField] float _initialOffset;
    [SerializeField] GameObject _deckImage;
    [SerializeField] GameObject _voidImage;

    public CardOrderSelectorView? View { get; set; }

    protected override Vector3 CalculateObjectPosition(int index, int count)
    {
      var offset = LinearObjectLayout.CalculateOffset(TotalWidth(), _initialSpacing, _cardWidth, index, count);
      return _leftEdge.position + ToAxisPosition(offset + _initialOffset);
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

    protected override void OnShowStart()
    {
      if (View?.IncludeDeck == true)
      {
        _deckImage.SetActive(true);
        TweenUtils.FadeIn(_deckImage.GetComponent<SpriteRenderer>());
      }
      if (View?.IncludeVoid == true)
      {
        _voidImage.SetActive(true);
        TweenUtils.FadeIn(_voidImage.GetComponent<SpriteRenderer>());
      }
    }

    protected override void OnHideStart()
    {
      if (View?.IncludeDeck == true)
      {
        TweenUtils.FadeOut(_deckImage.GetComponent<SpriteRenderer>());
      }
      if (View?.IncludeVoid == true)
      {
        TweenUtils.FadeOut(_voidImage.GetComponent<SpriteRenderer>());
      }
    }

    protected override void OnHideComplete()
    {
      _deckImage.SetActive(false);
      _voidImage.SetActive(false);
    }
  }
}