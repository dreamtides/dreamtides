#nullable enable

using System.Collections;
using DG.Tweening;
using Dreamtides.Layout;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.Utils;

namespace Dreamtides.Animations
{
  public class DrawUserCardsAnimation
  {
    public IEnumerator Handle(
      MoveCardsWithCustomAnimationCommand command,
      CardAnimationService service
    )
    {
      for (var i = 0; i < command.Cards.Count; ++i)
      {
        if (i < command.Cards.Count - 1)
        {
          service.StartCoroutine(DrawUserCard(command, i, isLastCard: false, service));
          yield return new UnityEngine.WaitForSeconds(command.StaggerInterval.ToSeconds());
        }
        else
        {
          yield return DrawUserCard(command, i, isLastCard: true, service);
        }
      }
    }

    IEnumerator DrawUserCard(
      MoveCardsWithCustomAnimationCommand command,
      int index,
      bool isLastCard,
      CardAnimationService service
    )
    {
      var cardView = command.Cards[index];
      var card = service.Registry.CardService.GetCard(cardView.Id);
      if (card.Parent)
      {
        card.Parent.RemoveIfPresent(card);
      }

      service.Registry.SoundService.PlayDrawCardSound();
      var sequence = TweenUtils.Sequence("DrawUserCard");
      var moveDuration = 0.3f;
      card.SortingKey = (int)cardView.Position.SortingKey;
      card.Render(service.Registry, cardView, sequence);
      card.GameContext = GameContext.DrawnCards;
      sequence.Insert(
        0,
        card.transform.DOMove(
            service.Registry.Layout.DrawnCardsPosition.transform.position,
            moveDuration
          )
          .SetEase(Ease.OutCubic)
      );
      sequence.Insert(
        0,
        card.transform.DORotateQuaternion(
          service.Registry.Layout.DrawnCardsPosition.transform.rotation,
          moveDuration
        )
      );
      yield return new UnityEngine.WaitForSeconds(moveDuration + command.PauseDuration.ToSeconds());

      var layout = service.Registry.CardService.LayoutForPosition(command.Destination);
      layout.Add(card);
      if (!isLastCard)
      {
        // Running this on the last card will conflict with LayoutService.ApplyLayout.
        layout.ApplyLayout(TweenUtils.Sequence("DrawUserCardMoveToHand"));
      }
    }
  }
}
