#nullable enable

using System.Collections;
using System.Linq;
using DG.Tweening;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.Utils;
using UnityEngine;

namespace Dreamtides.Animations
{
  public class ShowInShopLayoutAnimation
  {
    public IEnumerator Handle(
      MoveCardsWithCustomAnimationCommand command,
      CardAnimationService service
    )
    {
      var shopLayout = service.Registry.DreamscapeLayout.ShopLayout;
      var cardViews = command.Cards;
      var cards = cardViews.Select(cv => service.Registry.CardService.GetCard(cv.Id)).ToList();
      for (int i = 0; i < cards.Count; i++)
      {
        cards[i].HideButtonAttachmentUntilNextRender();
      }
      var stagger = command.StaggerInterval.ToSeconds();

      service.Registry.DreamscapeService.ShowShopWithCards(cards);
      var existingCount = shopLayout.Objects.Count;
      var finalCount = existingCount + cards.Count;
      for (int i = 0; i < cards.Count; ++i)
      {
        var card = cards[i];
        var cardView = cardViews[i];
        int targetIndex = existingCount + i;
        if (i < cards.Count - 1)
        {
          service.StartCoroutine(
            MoveCardToShop(shopLayout, card, cardView, targetIndex, finalCount, service)
          );
          if (stagger > 0)
          {
            yield return new WaitForSeconds(stagger);
          }
        }
        else
        {
          yield return MoveCardToShop(shopLayout, card, cardView, targetIndex, finalCount, service);
        }
      }
    }

    public IEnumerator HandleHide(
      MoveCardsWithCustomAnimationCommand command,
      CardAnimationService service
    )
    {
      var shopLayout = service.Registry.DreamscapeLayout.ShopLayout;
      var cardViews = command.Cards;
      var cards = cardViews.Select(cv => service.Registry.CardService.GetCard(cv.Id)).ToList();
      for (int i = 0; i < cards.Count; i++)
      {
        cards[i].HideButtonAttachmentUntilNextRender();
      }
      var stagger = command.StaggerInterval.ToSeconds();

      for (int i = 0; i < cards.Count; ++i)
      {
        var card = cards[i];
        if (i < cards.Count - 1)
        {
          service.StartCoroutine(ScaleCardDownInPlace(card));
          if (stagger > 0)
          {
            yield return new WaitForSeconds(stagger);
          }
        }
        else
        {
          yield return ScaleCardDownInPlace(card);
        }
      }
    }

    IEnumerator ScaleCardDownInPlace(Card card)
    {
      var seq = TweenUtils.Sequence("ShopScaleDown");
      seq.Insert(0, card.transform.DOScale(0, TweenUtils.MoveAnimationDurationSeconds));
      yield return seq.WaitForCompletion();
    }

    IEnumerator MoveCardToShop(
      StandardObjectLayout shopLayout,
      Card card,
      CardView cardView,
      int targetIndex,
      int finalCount,
      CardAnimationService service
    )
    {
      if (card.Parent)
      {
        card.Parent.RemoveIfPresent(card);
      }
      // Compute target transform from the layout
      var targetPosition = shopLayout.CalculateObjectPosition(targetIndex, finalCount);
      var rotationEuler =
        shopLayout.CalculateObjectRotation(targetIndex, finalCount)
        ?? shopLayout.transform.rotation.eulerAngles;
      var targetRotation = Quaternion.Euler(rotationEuler);
      var targetScale =
        shopLayout.CalculateObjectScale(targetIndex, finalCount)
        ?? shopLayout.transform.localScale.x;

      // Immediately jump to final position/rotation at tiny scale
      card.transform.position = targetPosition;
      card.transform.rotation = targetRotation;
      card.transform.localScale = Vector3.one * 0.01f;

      service.Registry.SoundService.PlayDrawCardSound();

      var seq = TweenUtils.Sequence("ShopScale");
      card.SortingKey = (int)cardView.Position.SortingKey;
      card.Render(cardView, seq);

      // Animate only the scale from 0.01 to the layout's target scale
      seq.Insert(
        0,
        card.transform.DOScale(Vector3.one * targetScale, TweenUtils.MoveAnimationDurationSeconds)
      );

      yield return seq.WaitForCompletion();
    }
  }
}
