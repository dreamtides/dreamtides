#nullable enable

using System.Collections;
using System.Collections.Generic;
using System.Linq;
using DG.Tweening;
using Dreamtides.Layout;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.Utils;
using UnityEngine;

namespace Dreamtides.Animations
{
  public class MoveToQuestDeckOrDestroyAnimation
  {
    public IEnumerator Handle(
      MoveCardsWithCustomAnimationCommand command,
      CardAnimationService service
    )
    {
      var questDeckLayout = service.Registry.DreamscapeLayout.QuestDeck;
      var destroyedQuestCardsLayout = service.Registry.DreamscapeLayout.DestroyedQuestCards;
      var cardViews = command.Cards;
      var toQuestDeck = cardViews
        .Where(cv => cv.Position.Position.Enum == PositionEnum.QuestDeck)
        .ToList();
      var toDestroy = cardViews
        .Where(cv => cv.Position.Position.Enum != PositionEnum.QuestDeck)
        .ToList();

      var stagger = command.StaggerInterval.ToSeconds();
      var pause = command.PauseDuration.ToSeconds();

      bool destroyDone = false;
      bool questDone = false;

      service.StartCoroutine(
        DestroyFlow(
          toDestroy,
          destroyedQuestCardsLayout,
          stagger,
          service,
          command.CardTrail,
          () => destroyDone = true
        )
      );

      service.StartCoroutine(
        QuestDeckFlow(
          toQuestDeck,
          questDeckLayout,
          pause,
          stagger,
          service,
          command.CardTrail,
          () => questDone = true
        )
      );

      yield return new WaitUntil(() => destroyDone && questDone);
    }

    IEnumerator DestroyFlow(
      List<CardView> toDestroy,
      ObjectLayout destroyedLayout,
      float stagger,
      CardAnimationService service,
      ProjectileAddress? cardTail,
      System.Action onDone
    )
    {
      service.Registry.SoundService.Play(service.FlipCardSound);

      for (int i = 0; i < toDestroy.Count; ++i)
      {
        var cardView = toDestroy[i];
        var card = service.Registry.CardService.GetCard(cardView.Id);
        card.SortingKey = (int)cardView.Position.SortingKey;
        card.TurnFaceDown(TweenUtils.Sequence("DestroyQuestCardFlip"));
      }

      yield return new WaitForSeconds(0.3f);

      for (int i = 0; i < toDestroy.Count; ++i)
      {
        var cardView = toDestroy[i];
        var card = service.Registry.CardService.GetCard(cardView.Id);
        destroyedLayout.Add(card);
        destroyedLayout.ApplyLayout(TweenUtils.Sequence("DestroyQuestCardMove"));
      }

      onDone();
    }

    IEnumerator QuestDeckFlow(
      List<CardView> toQuestDeck,
      ObjectLayout questDeckLayout,
      float pause,
      float stagger,
      CardAnimationService service,
      ProjectileAddress? cardTail,
      System.Action onDone
    )
    {
      yield return new WaitForSeconds(0.3f);

      for (int i = 0; i < toQuestDeck.Count; ++i)
      {
        var cardView = toQuestDeck[i];
        var card = service.Registry.CardService.GetCard(cardView.Id);
        var moveSeq = TweenUtils.Sequence("QuestDeckMoveAbove");
        card.SortingKey = (int)cardView.Position.SortingKey;
        var anchor = service.Registry.DreamscapeLayout.AboveQuestDeck;

        if (cardTail != null)
        {
          card.SetCardTrail(cardTail);
        }

        moveSeq.Insert(
          0,
          card.transform.DOMove(anchor.position, TweenUtils.MoveAnimationDurationSeconds)
        );
        moveSeq.Insert(
          0,
          card.transform.DORotateQuaternion(
            anchor.rotation,
            TweenUtils.MoveAnimationDurationSeconds
          )
        );
        moveSeq.Insert(
          0,
          card.transform.DOScale(anchor.localScale, TweenUtils.MoveAnimationDurationSeconds)
        );

        yield return moveSeq.WaitForCompletion();

        service.Registry.SoundService.Play(service.MoveToQuestDeckSound);

        if (cardTail != null)
        {
          card.ClearCardTrail();
        }

        var flipSeq = TweenUtils.Sequence("QuestDeckFlip");
        card.TurnFaceDown(flipSeq);
        yield return flipSeq.WaitForCompletion();

        var addSeq = TweenUtils.Sequence("QuestDeckAdd");
        questDeckLayout.Add(card);
        questDeckLayout.ApplyLayout(addSeq);
        yield return addSeq.WaitForCompletion();
      }

      onDone();
    }
  }
}
