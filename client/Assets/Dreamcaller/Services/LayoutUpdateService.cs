#nullable enable

using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using DG.Tweening;
using Dreamcaller.Components;
using Dreamcaller.Layout;
using Dreamcaller.Schema;
using Dreamcaller.Utils;

namespace Dreamcaller.Services
{
  public class LayoutUpdateService : Service
  {
    readonly Dictionary<ClientCardId, Card> _cards = new();

    public IEnumerator UpdateLayout(BattleView view)
    {
      var sequence = TweenUtils.Sequence("UpdateLayout").SetEase(Ease.InOutSine);
      var toDelete = _cards.Keys.ToHashSet();

      foreach (var cardView in view.Cards)
      {
        toDelete.Remove(cardView.Id);
        var layout = LayoutForPosition(cardView.Position.Position);
        Card card;
        if (_cards.ContainsKey(cardView.Id))
        {
          card = _cards[cardView.Id];
        }
        else
        {
          card = InstantiateCardPrefab();
          _cards[cardView.Id] = card;
          card.transform.position = cardView.CreatePosition != null ?
              LayoutForPosition(cardView.CreatePosition.Position).GetTargetPosition() :
              layout.GetTargetPosition();
        }

        layout.Add(card);
        card.Render(cardView, sequence);
      }

      InsertDeleteAnimations(sequence, toDelete);
      InsertAllLayoutAnimations(sequence);
      sequence.AppendCallback(() => DestroyCards(toDelete));
      yield return sequence.WaitForCompletion();
    }

    void InsertAllLayoutAnimations(Sequence sequence)
    {
      Registry.UserHand.InsertAnimationSequence(sequence);
    }

    void InsertDeleteAnimations(Sequence sequence, HashSet<ClientCardId> toDelete)
    {
      foreach (var cardId in toDelete)
      {
        var card = Errors.CheckNotNull(_cards[cardId]);
        if (card.CardView.DestroyPosition != null)
        {
          if (card.CardView.DestroyPosition.Position.PositionClass.InDeck != null)
          {
            card.TurnFaceDown(sequence);
          }

          var layout = LayoutForPosition(card.CardView.DestroyPosition.Position);
          sequence.Insert(0, card.transform.DOMove(layout.GetTargetPosition(),
              TweenUtils.MoveAnimationDurationSeconds));
        }

        _cards.Remove(cardId);

        if (card.Parent)
        {
          card.Parent.RemoveIfPresent(card);
        }
      }
    }

    void DestroyCards(HashSet<ClientCardId> toDelete)
    {
      foreach (var cardId in toDelete)
      {
        Destroy(_cards[cardId].gameObject);
      }
    }

    Card InstantiateCardPrefab() => throw new NotImplementedException();

    ObjectLayout LayoutForPosition(Position position)
    {
      if (position.PositionClass.InHand is { } inHand)
      {
        return inHand switch
        {
          DisplayPlayer.User => Registry.UserHand,
          _ => throw new NotImplementedException(),
        };
      }

      throw new NotImplementedException();
    }
  }
}

