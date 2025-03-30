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
using UnityEngine;

namespace Dreamcaller.Services
{
  public class LayoutService : Service
  {
    [SerializeField] Card _cardPrefab = null!;
    [SerializeField] Card _tokenPrefab = null!;
    [SerializeField] Card _dreamwellPrefab = null!;
    [SerializeField] Card _enemyPrefab = null!;
    [SerializeField] Card _dreamsignPrefab = null!;

    Dictionary<string, Card> Cards { get; } = new();

    public IEnumerator UpdateLayout(UpdateBattleCommand command, Sequence? sequence = null)
    {
      var view = command.Battle;
      var toDelete = Cards.Keys.ToHashSet();

      if (command.UpdateSound != null)
      {
        Registry.SoundService.Play(command.UpdateSound);
      }

      foreach (var cardView in view.Cards)
      {
        var cardId = cardView.ClientId();
        toDelete.Remove(cardId);
        var layout = LayoutForPosition(cardView.Position);
        Card card;
        if (Cards.ContainsKey(cardId))
        {
          card = Cards[cardId];
        }
        else
        {
          card = cardView.Prefab switch
          {
            CardPrefab.Token => ComponentUtils.Instantiate(_tokenPrefab),
            CardPrefab.Dreamwell => ComponentUtils.Instantiate(_dreamwellPrefab),
            CardPrefab.Enemy => ComponentUtils.Instantiate(_enemyPrefab),
            CardPrefab.Dreamsign => ComponentUtils.Instantiate(_dreamsignPrefab),
            _ => ComponentUtils.Instantiate(_cardPrefab)
          };
          if (cardView.CreatePosition != null)
          {
            LayoutForPosition(cardView.CreatePosition).ApplyTargetTransform(card);
          }
          else
          {
            layout.ApplyTargetTransform(card);
          }
          Cards[cardId] = card;
        }

        card.SortingKey = (int)cardView.Position.SortingKey;
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

    public Card GetCard(CardId id) =>
        Errors.CheckNotNull(Cards[id.ClientId()]);

    /// <summary>
    /// Returns the game object for the given game object id.
    /// </summary>
    public Displayable GetGameObject(GameObjectId id)
    {
      if (id.Deck is { } deck)
      {
        return deck switch
        {
          PlayerName.User => Registry.Layout.UserDeck,
          PlayerName.Enemy => Registry.Layout.EnemyDeck,
          _ => throw Errors.UnknownEnumValue(deck)
        };
      }

      if (id.Void is { } voidPile)
      {
        return voidPile switch
        {
          PlayerName.User => Registry.Layout.UserVoid,
          PlayerName.Enemy => Registry.Layout.EnemyVoid,
          _ => throw Errors.UnknownEnumValue(voidPile)
        };
      }

      if (id.Avatar is { } avatar)
      {
        return avatar switch
        {
          PlayerName.User => Registry.Layout.UserStatusDisplay,
          PlayerName.Enemy => Registry.Layout.EnemyStatusDisplay,
          _ => throw Errors.UnknownEnumValue(avatar)
        };
      }

      return GetCard(id.CardId);
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
      var layout = LayoutForPosition(card.CardView.Position);
      layout.Add(card);
    }

    /// <summary>
    /// Moves an object to a new target ObjectLayout, optionally animating the
    /// transition if a sequence is provided.
    /// </summary>
    public void MoveObject(Displayable displayable, ObjectPosition position, Sequence? sequence = null)
    {
      var layout = LayoutForPosition(position);
      layout.Add(displayable);
      ApplyAllLayouts(sequence);
    }

    /// <summary>
    /// Jumps an object towards the camera by 'distance' units
    /// </summary>
    public void MoveTowardsCamera(MonoBehaviour component, float distance)
    {
      var towardsCameraDirection = -Registry.Layout.MainCamera.transform.forward;
      component.transform.position += towardsCameraDirection * distance;
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
      if (Registry.Layout.Browser.Objects.Count > 0)
      {
        Registry.Layout.Browser.Show(Registry, sequence);
      }
      else
      {
        Registry.Layout.Browser.Hide(Registry, sequence);
      }

      if (Registry.Layout.CardOrderSelector.Objects.Count > 0 ||
          Registry.Layout.CardOrderSelectorVoid.Objects.Count > 0)
      {
        Registry.Layout.CardOrderSelector.Show(Registry, sequence);
      }
      else
      {
        Registry.Layout.CardOrderSelector.Hide(Registry, sequence);
      }

      Registry.Layout.UserHand.ApplyLayout(sequence);
      Registry.Layout.EnemyHand.ApplyLayout(sequence);
      Registry.Layout.UserDeck.ApplyLayout(sequence);
      Registry.Layout.EnemyDeck.ApplyLayout(sequence);
      Registry.Layout.UserVoid.ApplyLayout(sequence);
      Registry.Layout.EnemyVoid.ApplyLayout(sequence);
      Registry.Layout.UserStatusDisplay.ApplyLayout(sequence);
      Registry.Layout.EnemyStatusDisplay.ApplyLayout(sequence);
      Registry.Layout.UserBattlefield.ApplyLayout(sequence);
      Registry.Layout.EnemyBattlefield.ApplyLayout(sequence);
      Registry.Layout.DrawnCardsPosition.ApplyLayout(sequence);
      Registry.Layout.Stack.ApplyLayout(sequence);
      Registry.Layout.SelectingTargetsEnemy.ApplyLayout(sequence);
      Registry.Layout.SelectingTargetsUser.ApplyLayout(sequence);
      Registry.Layout.Browser.ApplyLayout(sequence);
      Registry.Layout.UserDreamwell.ApplyLayout(sequence);
      Registry.Layout.EnemyDreamwell.ApplyLayout(sequence);
      Registry.Layout.DreamwellActivation.ApplyLayout(sequence);
      Registry.Layout.CardOrderSelector.ApplyLayout(sequence);
      Registry.Layout.CardOrderSelectorVoid.ApplyLayout(sequence);
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
            var layout = LayoutForPosition(card.CardView.DestroyPosition);
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
        Destroy(card.gameObject);
      }
    }

    ObjectLayout LayoutForPosition(ObjectPosition objectPosition)
    {
      var position = objectPosition.Position;
      if (position.Enum == PositionEnum.Drawn)
      {
        return Registry.Layout.DrawnCardsPosition;
      }

      if (position.Enum == PositionEnum.OnStack)
      {
        return Registry.Layout.Stack;
      }

      if (position.Enum == PositionEnum.Browser)
      {
        return Registry.Layout.Browser;
      }

      if (position.Enum == PositionEnum.DreamwellActivation)
      {
        return Registry.Layout.DreamwellActivation;
      }

      if (position.PositionClass == null)
      {
        throw new InvalidOperationException($"Unknown layout position: ${position.Enum}");
      }

      if (position.PositionClass.InHand is { } inHand)
      {
        return inHand switch
        {
          PlayerName.User => Registry.Layout.UserHand,
          PlayerName.Enemy => Registry.Layout.EnemyHand,
          _ => throw Errors.UnknownEnumValue(inHand),
        };
      }

      if (position.PositionClass.InDeck is { } inDeck)
      {
        return inDeck switch
        {
          PlayerName.User => Registry.Layout.UserDeck,
          PlayerName.Enemy => Registry.Layout.EnemyDeck,
          _ => throw Errors.UnknownEnumValue(inDeck),
        };
      }

      if (position.PositionClass.OnBattlefield is { } onBattlefield)
      {
        return onBattlefield switch
        {
          PlayerName.User => Registry.Layout.UserBattlefield,
          PlayerName.Enemy => Registry.Layout.EnemyBattlefield,
          _ => throw Errors.UnknownEnumValue(onBattlefield),
        };
      }

      if (position.PositionClass.InVoid is { } inVoid)
      {
        return inVoid switch
        {
          PlayerName.User => Registry.Layout.UserVoid,
          PlayerName.Enemy => Registry.Layout.EnemyVoid,
          _ => throw Errors.UnknownEnumValue(inVoid),
        };
      }

      if (position.PositionClass.InPlayerStatus is { } inPlayerStatus)
      {
        return inPlayerStatus switch
        {
          PlayerName.User => Registry.Layout.UserStatusDisplay,
          PlayerName.Enemy => Registry.Layout.EnemyStatusDisplay,
          _ => throw Errors.UnknownEnumValue(inPlayerStatus),
        };
      }

      if (position.PositionClass.SelectingTargets is { } selectingTargets)
      {
        return selectingTargets switch
        {
          PlayerName.User => Registry.Layout.SelectingTargetsUser,
          PlayerName.Enemy => Registry.Layout.SelectingTargetsEnemy,
          _ => throw Errors.UnknownEnumValue(selectingTargets),
        };
      }

      if (position.PositionClass.InDreamwell is { } inDreamwell)
      {
        return inDreamwell switch
        {
          PlayerName.User => Registry.Layout.UserDreamwell,
          PlayerName.Enemy => Registry.Layout.EnemyDreamwell,
          _ => throw Errors.UnknownEnumValue(inDreamwell),
        };
      }

      if (position.PositionClass.HiddenWithinCard is { } cardId)
      {
        return GetCard(cardId).ContainedObjects;
      }

      if (position.PositionClass.CardOrderSelector is { } cardOrderSelectorTarget)
      {
        return cardOrderSelectorTarget switch
        {
          CardOrderSelectionTarget.Deck => Registry.Layout.CardOrderSelector,
          CardOrderSelectionTarget.Void => Registry.Layout.CardOrderSelectorVoid,
          _ => throw Errors.UnknownEnumValue(cardOrderSelectorTarget),
        };
      }

      throw new InvalidOperationException($"Unknown layout position: ${position.PositionClass}, ${position.Enum}");
    }
  }
}

