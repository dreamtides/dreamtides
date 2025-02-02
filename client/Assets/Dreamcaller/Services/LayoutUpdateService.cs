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
using Newtonsoft.Json;
using UnityEngine;

namespace Dreamcaller.Services
{
  public class LayoutUpdateService : Service
  {
    [SerializeField] Card? _cardPrefab;
    Card CardPrefab { get => Errors.CheckNotNull(_cardPrefab); }

    Dictionary<string, Card> Cards { get; } = new();

    public IEnumerator UpdateLayout(BattleView view, Sequence? sequence = null)
    {
      sequence?.SetEase(Ease.InOutSine);
      var toDelete = Cards.Keys.ToHashSet();

      foreach (var cardView in view.Cards)
      {
        var cardId = SerializeCardId(cardView.Id);
        toDelete.Remove(cardId);
        var layout = LayoutForPosition(cardView.Position.Position);
        Card card;
        if (Cards.ContainsKey(cardId))
        {
          card = Cards[cardId];
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
          Cards[cardId] = card;
        }

        card.Render(Registry, cardView, sequence);
        layout.Add(card);
      }

      var delete = PrepareToDelete(sequence, toDelete);
      ApplyAllLayouts(sequence);
      if (delete.Count > 0)
      {
        if (sequence != null)
        {
          sequence.AppendCallback(() => DestroyCards(delete));
        }
        else
        {
          DestroyCards(delete);
        }
      }

      if (sequence != null)
      {
        yield return sequence.WaitForCompletion();
      }
    }

    /// <summary>
    /// Runs all layout animations immediately
    /// </summary>
    public void RunAnimations(Action? onComplete = null)
    {
      StartCoroutine(RunAnimationsAsync(onComplete));
    }

    /// <summary>
    /// Adds a card to its correct parent layout
    /// </summary>
    public void AddToParent(Card card)
    {
      var layout = LayoutForPosition(card.CardView.Position.Position);
      layout.Add(card);
    }

    IEnumerator RunAnimationsAsync(Action? onComplete = null)
    {
      var sequence = TweenUtils.Sequence("RunAnimations");
      ApplyAllLayouts(sequence);
      yield return sequence.WaitForCompletion();
      onComplete?.Invoke();
    }

    void ApplyAllLayouts(Sequence? sequence)
    {
      Registry.UserHand.ApplyLayout(sequence);
      Registry.EnemyHand.ApplyLayout(sequence);
      Registry.UserDeck.ApplyLayout(sequence);
      Registry.EnemyDeck.ApplyLayout(sequence);
      Registry.UserVoid.ApplyLayout(sequence);
      Registry.EnemyVoid.ApplyLayout(sequence);
      Registry.UserBattlefield.ApplyLayout(sequence);
      Registry.EnemyBattlefield.ApplyLayout(sequence);
      Registry.DrawnCardsPosition.ApplyLayout(sequence);
    }

    List<Card> PrepareToDelete(Sequence? sequence, HashSet<string> toDelete)
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

      if (position.PositionClass.InVoid is { } inVoid)
      {
        return inVoid switch
        {
          DisplayPlayer.User => Registry.UserVoid,
          DisplayPlayer.Enemy => Registry.EnemyVoid,
          _ => throw Errors.UnknownEnumValue(inVoid),
        };
      }

      return Registry.Offscreen;
    }

    string SerializeCardId(CardId id)
    {
      return JsonConvert.SerializeObject(id, Converter.Settings);
    }
  }
}

