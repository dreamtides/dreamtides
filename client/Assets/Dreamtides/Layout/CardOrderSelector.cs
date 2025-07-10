#nullable enable

using System.Runtime.CompilerServices;
using Dreamtides.Schema;
using Dreamtides.Utils;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]
namespace Dreamtides.Layout
{
  public class CardOrderSelector : AbstractCardBrowser
  {
    [SerializeField] internal float _initialSpacing = 0.5f;
    [SerializeField] internal float _initialOffset;
    [SerializeField] internal GameObject _deckImage = null!;
    [SerializeField] internal GameObject _voidImage = null!;
    [SerializeField] internal ObjectLayout _cardOrderSelectorVoid = null!;

    public CardOrderSelectorView? View { get; set; }

    public override Vector3 CalculateObjectPosition(int index, int count)
    {
      var offset = CenteredObjectLayout.CalculateOffset(TotalWidth(), _initialSpacing, _cardWidth, index, count);
      return _leftEdge.position + new Vector3(offset + _initialOffset, 0, 0);
    }

    public override Vector3? CalculateObjectRotation(int index, int count) => transform.rotation.eulerAngles;

    /// <summary>
    /// Returns the index position within the selector which most closely maps
    /// to the position of the given Transform.
    /// </summary>
    public DeckCardSelectedOrder SelectCardOrderWithinDisplay(Transform t, long cardId)
    {
      var targetPosition = t.position.x;

      if (targetPosition > _cardOrderSelectorVoid.transform.position.x - _cardWidth)
      {
        return new DeckCardSelectedOrder
        {
          CardId = cardId,
          Target = new CardOrderSelectionTarget { Enum = CardOrderSelectionTargetEnum.Void },
        };
      }

      if (Objects.Count == 0)
      {
        return new DeckCardSelectedOrder
        {
          CardId = cardId,
          Target = new CardOrderSelectionTarget
          {
            CardOrderSelectionTargetClass = new CardOrderSelectionTargetClass
            {
              Deck = 0,
            }
          },
        };
      }

      if (targetPosition < Objects[0].transform.position.x)
      {
        return new DeckCardSelectedOrder
        {
          CardId = cardId,
          Target = new CardOrderSelectionTarget
          {
            CardOrderSelectionTargetClass = new CardOrderSelectionTargetClass
            {
              Deck = 0,
            }
          },
        };
      }

      if (targetPosition > Objects[Objects.Count - 1].transform.position.x)
      {
        return new DeckCardSelectedOrder
        {
          CardId = cardId,
          Target = new CardOrderSelectionTarget
          {
            CardOrderSelectionTargetClass = new CardOrderSelectionTargetClass
            {
              Deck = Objects.Count,
            }
          },
        };
      }

      for (int i = 0; i < Objects.Count - 1; i++)
      {
        var currentPosition = Objects[i].transform.position.x;
        var nextPosition = Objects[i + 1].transform.position.x;

        if (targetPosition >= currentPosition && targetPosition <= nextPosition)
        {
          return new DeckCardSelectedOrder
          {
            CardId = cardId,
            Target = new CardOrderSelectionTarget
            {
              CardOrderSelectionTargetClass = new CardOrderSelectionTargetClass
              {
                Deck = (targetPosition - currentPosition < nextPosition - targetPosition) ? i + 1 : i + 2,
              }
            },
          };
        }
      }

      return new DeckCardSelectedOrder
      {
        CardId = cardId,
        Target = new CardOrderSelectionTarget
        {
          CardOrderSelectionTargetClass = new CardOrderSelectionTargetClass
          {
            Deck = 0,
          }
        },
      };
    }

    protected override void OnShowStart()
    {
      if (View?.IncludeDeck == true)
      {
        _deckImage.SetActive(true);
        TweenUtils.FadeInSprite(_deckImage.GetComponent<SpriteRenderer>());
      }
      if (View?.IncludeVoid == true)
      {
        _voidImage.SetActive(true);
        TweenUtils.FadeInSprite(_voidImage.GetComponent<SpriteRenderer>());
      }
    }

    protected override void OnHideStart()
    {
      if (View?.IncludeDeck == true)
      {
        TweenUtils.FadeOutSprite(_deckImage.GetComponent<SpriteRenderer>());
      }

      if (View?.IncludeVoid == true)
      {
        TweenUtils.FadeOutSprite(_voidImage.GetComponent<SpriteRenderer>());
      }
    }

    protected override void OnHideComplete()
    {
      _deckImage.SetActive(false);
      _voidImage.SetActive(false);
    }
  }
}