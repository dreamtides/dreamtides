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
      var stagger = command.StaggerInterval.ToSeconds();
      service.Registry.DreamscapeLayout.DreamscapeBackgroundOverlay.Show(
        0.7f,
        TweenUtils.Sequence("ShowShopBackground")
      );

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

      foreach (var card in cards)
      {
        card.ActionButton.SetActive(true);
      }
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
      var targetPosition = shopLayout.CalculateObjectPosition(targetIndex, finalCount);
      var rotationEuler =
        shopLayout.CalculateObjectRotation(targetIndex, finalCount)
        ?? shopLayout.transform.rotation.eulerAngles;
      var targetRotation = Quaternion.Euler(rotationEuler);
      var targetScale =
        shopLayout.CalculateObjectScale(targetIndex, finalCount)
        ?? shopLayout.transform.localScale.x;
      service.Registry.SoundService.PlayDrawCardSound();
      var seq = TweenUtils.Sequence("ShopMove");
      card.SortingKey = (int)cardView.Position.SortingKey;
      card.Render(service.Registry, cardView, seq);
      seq.Insert(0, card.transform.DOMove(targetPosition, TweenUtils.MoveAnimationDurationSeconds));
      seq.Insert(
        0,
        card.transform.DORotateQuaternion(targetRotation, TweenUtils.MoveAnimationDurationSeconds)
      );
      seq.Insert(
        0,
        card.transform.DOScale(Vector3.one * targetScale, TweenUtils.MoveAnimationDurationSeconds)
      );
      yield return seq.WaitForCompletion();
    }
  }
}
