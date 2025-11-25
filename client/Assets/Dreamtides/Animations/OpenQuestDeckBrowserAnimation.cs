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
  public class OpenQuestDeckBrowserAnimation
  {
    const float PhaseDurationSeconds = 0.3f;

    public IEnumerator Handle(
      MoveCardsWithCustomAnimationCommand command,
      CardAnimationService service
    )
    {
      var questDeckLayout = service.Registry.DreamscapeLayout.QuestDeck;
      var browserLayout = service.Registry.DreamscapeLayout.QuestDeckBrowser;
      var cardViews = command.Cards;
      var cards = cardViews.Select(cv => service.Registry.CardService.GetCard(cv.Id)).ToList();
      var stagger = command.StaggerInterval.ToSeconds();

      var finalCount = cards.Count;
      for (var i = 0; i < cards.Count; ++i)
      {
        var card = cards[i];
        var cardView = cardViews[i];
        if (i < cards.Count - 1)
        {
          service.StartCoroutine(
            MoveCardToBrowser(browserLayout, card, cardView, i, finalCount, stagger * i, service)
          );
        }
        else
        {
          yield return MoveCardToBrowser(
            browserLayout,
            card,
            cardView,
            i,
            finalCount,
            stagger * i,
            service
          );
        }
      }
    }

    IEnumerator MoveCardToBrowser(
      QuestDeckBrowserObjectLayout browserLayout,
      Card card,
      CardView cardView,
      int targetIndex,
      int finalCount,
      float startDelay,
      CardAnimationService service
    )
    {
      if (startDelay > 0)
      {
        yield return new WaitForSeconds(startDelay);
      }

      if (card.Parent)
      {
        card.Parent.RemoveIfPresent(card);
      }

      card.transform.SetParent(null, worldPositionStays: true);

      card.SortingKey = (int)cardView.Position.SortingKey;
      var renderSeq = TweenUtils.Sequence("OpenBrowserRender");
      card.Render(cardView, renderSeq);

      var targetLocalPosition = browserLayout.CalculateObjectPosition(targetIndex, finalCount);
      var targetLocalRotation =
        browserLayout.CalculateObjectRotation(targetIndex, finalCount) ?? Vector3.zero;
      var targetLocalScale =
        browserLayout.CalculateObjectScale(targetIndex, finalCount) ?? card.DefaultScale;

      var worldSpaceParent = browserLayout.WorldSpaceParent;
      var targetWorldPosition = worldSpaceParent.TransformPoint(targetLocalPosition);
      var targetWorldRotation = worldSpaceParent.rotation * Quaternion.Euler(targetLocalRotation);

      var viewport = service.Registry.GameViewport;
      var currentScreenPos = viewport.WorldToScreenPoint(card.transform.position);
      var targetScreenPos = viewport.WorldToScreenPoint(targetWorldPosition);
      var intermediateScreenPos = new Vector3(
        targetScreenPos.x,
        targetScreenPos.y,
        currentScreenPos.z
      );
      var intermediateWorldPos = viewport.ScreenToWorldPoint(intermediateScreenPos);

      var initialScale = card.transform.localScale;
      var intermediateScale = initialScale * 3f;

      var phase1Seq = TweenUtils.Sequence("OpenBrowserPhase1");
      phase1Seq.Insert(0, card.transform.DOMove(intermediateWorldPos, PhaseDurationSeconds));
      phase1Seq.Insert(
        0,
        card.transform.DORotateQuaternion(targetWorldRotation, PhaseDurationSeconds)
      );
      phase1Seq.Insert(0, card.transform.DOScale(intermediateScale, PhaseDurationSeconds));

      yield return phase1Seq.WaitForCompletion();

      var phase2Seq = TweenUtils.Sequence("OpenBrowserPhase2");
      phase2Seq.Insert(0, card.transform.DOMove(targetWorldPosition, PhaseDurationSeconds));
      phase2Seq.Insert(
        0,
        card.transform.DOScale(Vector3.one * targetLocalScale, PhaseDurationSeconds)
      );

      yield return phase2Seq.WaitForCompletion();

      browserLayout.Add(card);
    }
  }
}
