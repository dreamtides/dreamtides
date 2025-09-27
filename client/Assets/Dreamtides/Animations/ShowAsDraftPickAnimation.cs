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
  public class ShowAsDraftPickAnimation
  {
    public IEnumerator Handle(
      MoveCardsWithCustomAnimationCommand command,
      CardAnimationService service
    )
    {
      var draftLayout = service.Registry.DreamscapeLayout.DraftPickLayout;
      var cardViews = command.Cards;
      var cards = cardViews.Select(cv => service.Registry.CardService.GetCard(cv.Id)).ToList();
      var stagger = command.StaggerInterval.ToSeconds();

      var existingCount = draftLayout.Objects.Count;
      var finalCount = existingCount + cards.Count;
      for (int i = 0; i < cards.Count; ++i)
      {
        var card = cards[i];
        var cardView = cardViews[i];
        int targetIndex = existingCount + i;
        if (i < cards.Count - 1)
        {
          service.StartCoroutine(
            MoveCardToDraftPick(draftLayout, card, cardView, targetIndex, finalCount, service)
          );
          if (stagger > 0)
          {
            yield return new UnityEngine.WaitForSeconds(stagger);
          }
        }
        else
        {
          yield return MoveCardToDraftPick(
            draftLayout,
            card,
            cardView,
            targetIndex,
            finalCount,
            service
          );
        }
      }
    }

    IEnumerator MoveCardToDraftPick(
      SitePickObjectLayout draftLayout,
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
      var targetPosition = draftLayout.CalculateObjectPosition(targetIndex, finalCount);
      var rotationEuler =
        draftLayout.CalculateObjectRotation(targetIndex, finalCount)
        ?? draftLayout.transform.rotation.eulerAngles;
      var targetRotation = Quaternion.Euler(rotationEuler);
      var targetScale =
        draftLayout.CalculateObjectScale(targetIndex, finalCount)
        ?? draftLayout.transform.localScale.x;
      service.Registry.SoundService.PlayDrawCardSound();
      var seq = TweenUtils.Sequence("DraftPickMove");
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
