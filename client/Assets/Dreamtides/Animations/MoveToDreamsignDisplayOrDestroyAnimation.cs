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
  public class MoveToDreamsignDisplayOrDestroyAnimation
  {
    public IEnumerator Handle(
      MoveCardsWithCustomAnimationCommand command,
      CardAnimationService service
    )
    {
      var dreamsignLayout = service.Registry.DreamscapeLayout.DreamsignDisplay;
      var destroyedQuestCardsLayout = service.Registry.DreamscapeLayout.DestroyedQuestCards;
      var cardViews = command.Cards;

      var toDisplay = cardViews
        .Where(cv => cv.Position.Position.Enum == PositionEnum.DreamsignDisplay)
        .ToList();
      var toDestroy = cardViews
        .Where(cv => cv.Position.Position.Enum != PositionEnum.DreamsignDisplay)
        .ToList();

      var stagger = command.StaggerInterval.ToSeconds();
      var pause = command.PauseDuration.ToSeconds();

      bool destroyDone = false;
      bool displayDone = false;

      service.StartCoroutine(
        DestroyFlow(
          toDestroy,
          destroyedQuestCardsLayout,
          stagger,
          service,
          command.CardTail,
          () => destroyDone = true
        )
      );

      service.StartCoroutine(
        DisplayFlow(
          toDisplay,
          dreamsignLayout,
          pause,
          stagger,
          service,
          command.CardTail,
          () => displayDone = true
        )
      );

      yield return new WaitUntil(() => destroyDone && displayDone);
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

      for (var i = 0; i < toDestroy.Count; ++i)
      {
        var cardView = toDestroy[i];
        var card = service.Registry.CardService.GetCard(cardView.Id);
        card.SortingKey = (int)cardView.Position.SortingKey;
        card.TurnFaceDown(TweenUtils.Sequence("DestroyQuestCardFlip"));
      }

      yield return new WaitForSeconds(0.3f);

      for (var i = 0; i < toDestroy.Count; ++i)
      {
        var cardView = toDestroy[i];
        var card = service.Registry.CardService.GetCard(cardView.Id);
        destroyedLayout.Add(card);
        destroyedLayout.ApplyLayout(TweenUtils.Sequence("DestroyQuestCardMove"));
      }

      onDone();
    }

    IEnumerator DisplayFlow(
      List<CardView> toDisplay,
      ObjectLayout dreamsignLayout,
      float pause,
      float stagger,
      CardAnimationService service,
      ProjectileAddress? cardTail,
      System.Action onDone
    )
    {
      yield return new WaitForSeconds(0.3f);

      for (var i = 0; i < toDisplay.Count; ++i)
      {
        var cardView = toDisplay[i];
        var card = service.Registry.CardService.GetCard(cardView.Id);
        card.SortingKey = (int)cardView.Position.SortingKey;

        if (cardTail != null)
        {
          card.SetCardTrail(cardTail, TweenUtils.MoveAnimationDurationSeconds);
        }

        card.transform.SetParent(dreamsignLayout.transform, worldPositionStays: true);

        var addSeq = TweenUtils.Sequence("DreamsignDisplayAdd");
        dreamsignLayout.Add(card);
        dreamsignLayout.ApplyLayout(addSeq);
        yield return addSeq.WaitForCompletion();

        if (cardTail != null)
        {
          card.ClearCardTrail();
        }
      }

      onDone();
    }
  }
}
