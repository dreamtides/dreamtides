#nullable enable

using System.Collections;
using System.Collections.Generic;
using System.Linq;
using DG.Tweening;
using Dreamcaller.Components;
using Dreamcaller.Layout;
using Dreamcaller.Schema;
using Dreamcaller.Utils;
using UnityEngine;

namespace Dreamcaller.Services
{
  public class LayoutUpdateService : Service
  {
    [SerializeField] Card? _cardPrefab;
    Card CardPrefab { get => Errors.CheckNotNull(_cardPrefab); }

    Dictionary<ClientCardId, Card> Cards { get; } = new();

    public IEnumerator UpdateLayout(BattleView view)
    {
      var sequence = TweenUtils.Sequence("UpdateLayout").SetEase(Ease.InOutSine);
      var toDelete = Cards.Keys.ToHashSet();

      foreach (var cardView in view.Cards)
      {
        toDelete.Remove(cardView.Id);
        var layout = LayoutForPosition(cardView.Position.Position);
        Card card;
        if (Cards.ContainsKey(cardView.Id))
        {
          card = Cards[cardView.Id];
        }
        else
        {
          var position = cardView.CreatePosition != null ?
              LayoutForPosition(cardView.CreatePosition.Position).GetTargetPosition() :
              layout.GetTargetPosition();
          card = ComponentUtils.Instantiate(CardPrefab, position);
          Cards[cardView.Id] = card;
        }

        layout.Add(card);
        card.Render(Registry, cardView, sequence);
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
        var card = Errors.CheckNotNull(Cards[cardId]);
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

        Cards.Remove(cardId);

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
        Destroy(Cards[cardId].gameObject);
      }
    }

    ObjectLayout LayoutForPosition(Position position)
    {
      if (position.PositionClass.InHand is { } inHand)
      {
        return inHand switch
        {
          DisplayPlayer.User => Registry.UserHand,
          _ => Registry.Offscreen,
        };
      }

      return Registry.Offscreen;
    }
  }
}

