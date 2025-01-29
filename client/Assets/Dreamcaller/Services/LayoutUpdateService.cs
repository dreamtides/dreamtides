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

    Dictionary<string, Card> Cards { get; } = new();

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
          card = ComponentUtils.Instantiate(CardPrefab);
          if (cardView.CreatePosition != null)
          {
            LayoutForPosition(cardView.CreatePosition.Position).ApplyTargetTransform(card);
          }
          else
          {
            layout.ApplyTargetTransform(card);
          }
          Cards[cardView.Id] = card;
        }

        layout.Add(card);
        card.Render(Registry, cardView, sequence);
      }

      var delete = InsertDeleteAnimations(sequence, toDelete);
      InsertAllLayoutAnimations(sequence);
      if (delete.Count > 0)
      {
        sequence.AppendCallback(() => DestroyCards(delete));
      }

      yield return sequence.WaitForCompletion();
    }

    void InsertAllLayoutAnimations(Sequence sequence)
    {
      Registry.UserHand.InsertAnimationSequence(sequence);
      Registry.EnemyHand.InsertAnimationSequence(sequence);
      Registry.UserDeck.InsertAnimationSequence(sequence);
      Registry.EnemyDeck.InsertAnimationSequence(sequence);
      Registry.UserBattlefield.InsertAnimationSequence(sequence);
      Registry.EnemyBattlefield.InsertAnimationSequence(sequence);
      Registry.DrawnCardsPosition.InsertAnimationSequence(sequence);
    }

    List<Card> InsertDeleteAnimations(Sequence sequence, HashSet<string> toDelete)
    {
      var cards = new List<Card>();
      foreach (var cardId in toDelete)
      {
        var card = Errors.CheckNotNull(Cards[cardId]);
        cards.Add(card);
        if (card.CardView.DestroyPosition != null)
        {
          if (card.CardView.DestroyPosition.Position.PositionClass.InDeck != null)
          {
            card.TurnFaceDown(sequence);
          }

          if (card.CardView.DestroyPosition != null)
          {
            var layout = LayoutForPosition(card.CardView.DestroyPosition.Position);
            layout.ApplyTargetTransform(card, sequence);
          }
        }

        Cards.Remove(cardId);

        if (card.Parent)
        {
          card.Parent.RemoveIfPresent(card);
        }
      }

      return cards;
    }

    void DestroyCards(List<Card> delete)
    {
      foreach (var card in delete)
      {
        Debug.Log($"Destroying card {card.CardView.Revealed?.Name} with id {card.CardView.Id}");
        Destroy(card.gameObject);
      }
    }

    ObjectLayout LayoutForPosition(Position position)
    {
      if (position.Enum == PositionEnum.Drawn)
      {
        return Registry.DrawnCardsPosition;
      }

      if (position.PositionClass == null)
      {
        return Registry.Offscreen;
      }

      if (position.PositionClass.InHand is { } inHand)
      {
        return inHand switch
        {
          DisplayPlayer.User => Registry.UserHand,
          DisplayPlayer.Enemy => Registry.EnemyHand,
          _ => throw Errors.UnknownEnumValue(inHand),
        };
      }

      if (position.PositionClass.InDeck is { } inDeck)
      {
        return inDeck switch
        {
          DisplayPlayer.User => Registry.UserDeck,
          DisplayPlayer.Enemy => Registry.EnemyDeck,
          _ => throw Errors.UnknownEnumValue(inDeck),
        };
      }

      if (position.PositionClass.OnBattlefield is { } onBattlefield)
      {
        return onBattlefield switch
        {
          DisplayPlayer.User => Registry.UserBattlefield,
          DisplayPlayer.Enemy => Registry.EnemyBattlefield,
          _ => throw Errors.UnknownEnumValue(onBattlefield),
        };
      }

      return Registry.Offscreen;
    }
  }
}

