#nullable enable

using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using DG.Tweening;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Schema;
using Dreamtides.Utils;
using Newtonsoft.Json;
using UnityEngine;

namespace Dreamtides.Services
{
  public class CardService : Service
  {
    [SerializeField]
    Card _cardPrefab = null!;

    [SerializeField]
    Card _eventCardPrefab = null!;

    [SerializeField]
    Card _tokenPrefab = null!;

    [SerializeField]
    Card _dreamwellPrefab = null!;

    [SerializeField]
    Card _identityCardPrefab = null!;

    [SerializeField]
    Card _enemyPrefab = null!;

    [SerializeField]
    Card _dreamsignPrefab = null!;

    [SerializeField]
    Card _iconCardPrefab = null!;

    [SerializeField]
    Card _journeyCardPrefab = null!;

    [SerializeField]
    Card _offerCostCardPrefab = null!;

    Dictionary<string, Card> Cards { get; } = new();

    public Card GetCard(string id) => Errors.CheckNotNull(Cards[id]);

    public Card? GetCardIfExists(string id) => Cards.TryGetValue(id, out var card) ? card : null;

    public IEnumerable<string> GetCardIds() => Cards.Keys;

    public IEnumerator HandleUpdateBattleCommand(
      UpdateBattleCommand command,
      Sequence? sequence = null
    )
    {
      return ApplyUpdate(command.Battle.Cards, command.UpdateSound, sequence);
    }

    public IEnumerator HandleUpdateQuestCards(UpdateQuestCommand command, Sequence? sequence = null)
    {
      return ApplyUpdate(command.Quest.Cards, command.UpdateSound, sequence);
    }

    IEnumerator ApplyUpdate(
      List<CardView> cardViews,
      AudioClipAddress updateSound,
      Sequence? sequence = null
    )
    {
      var toDelete = Cards.Keys.ToHashSet();

      if (updateSound != null)
      {
        Registry.SoundService.Play(updateSound);
      }

      foreach (var cardView in cardViews)
      {
        var cardId = cardView.ClientId();
        toDelete.Remove(cardId);
        var layout = LayoutForObjectPosition(cardView.Position);
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
            CardPrefab.Event => ComponentUtils.Instantiate(_eventCardPrefab),
            CardPrefab.Identity => ComponentUtils.Instantiate(_identityCardPrefab),
            CardPrefab.IconCard => ComponentUtils.Instantiate(_iconCardPrefab),
            CardPrefab.Journey => ComponentUtils.Instantiate(_journeyCardPrefab),
            CardPrefab.OfferCost => ComponentUtils.Instantiate(_offerCostCardPrefab),
            _ => ComponentUtils.Instantiate(_cardPrefab),
          };
          card.Initialize(Registry, Mode, TestConfiguration);

          if (cardView.CreatePosition != null)
          {
            LayoutForObjectPosition(cardView.CreatePosition).ApplyTargetTransform(card);
          }
          else
          {
            layout.ApplyTargetTransform(card);
          }

          if (cardView.CreateSound != null)
          {
            Registry.SoundService.Play(cardView.CreateSound);
          }

          Cards[cardId] = card;
        }

        card.SortingKey = (int)cardView.Position.SortingKey;
        card.Render(cardView, sequence);
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

      if (sequence != null && sequence.Duration() > 0)
      {
        // WaitForCompletion takes ~50ms on an empty Sequence, so skip it if
        // there is no duration.
        yield return sequence.WaitForCompletion();
      }
    }

    /// <summary>
    /// Returns the game object for the given game object id.
    /// </summary>
    public Displayable GetGameObject(GameObjectId id)
    {
      if (id.Deck is { } deck)
      {
        return deck switch
        {
          DisplayPlayer.User => Registry.Layout.UserDeck,
          DisplayPlayer.Enemy => Registry.Layout.EnemyDeck,
          _ => throw Errors.UnknownEnumValue(deck),
        };
      }

      if (id.Void is { } voidPile)
      {
        return voidPile switch
        {
          DisplayPlayer.User => Registry.Layout.UserVoid,
          DisplayPlayer.Enemy => Registry.Layout.EnemyVoid,
          _ => throw Errors.UnknownEnumValue(voidPile),
        };
      }

      if (id.Avatar is { } avatar)
      {
        return avatar switch
        {
          DisplayPlayer.User => Registry.Layout.UserStatusDisplay,
          DisplayPlayer.Enemy => Registry.Layout.EnemyStatusDisplay,
          _ => throw Errors.UnknownEnumValue(avatar),
        };
      }

      return GetCard(Errors.CheckNotNull(id.CardId));
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
      var layout = LayoutForObjectPosition(card.CardView.Position);
      layout.Add(card);
    }

    /// <summary>
    /// Moves an object to a new target ObjectLayout, optionally animating the
    /// transition if a sequence is provided.
    /// </summary>
    public void MoveObject(
      Displayable displayable,
      ObjectPosition position,
      Sequence? sequence = null
    )
    {
      var layout = LayoutForObjectPosition(position);
      layout.Add(displayable);
      ApplyAllLayouts(sequence);
    }

    /// <summary>
    /// Jumps an object towards the camera by 'distance' units
    /// </summary>
    public void MoveTowardsCamera(MonoBehaviour component, float distance)
    {
      var towardsCameraDirection = -Registry.MainCamera.transform.forward;
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

      if (
        Registry.Layout.CardOrderSelector.Objects.Count > 0
        || Registry.Layout.CardOrderSelectorVoid.Objects.Count > 0
      )
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
      Registry.Layout.DefaultStack.ApplyLayout(sequence);
      Registry.Layout.TargetingUserStack.ApplyLayout(sequence);
      Registry.Layout.TargetingEnemyStack.ApplyLayout(sequence);
      Registry.Layout.TargetingBothStack.ApplyLayout(sequence);
      Registry.Layout.Browser.ApplyLayout(sequence);
      Registry.Layout.UserDreamwell.ApplyLayout(sequence);
      Registry.Layout.EnemyDreamwell.ApplyLayout(sequence);
      Registry.Layout.DreamwellActivation.ApplyLayout(sequence);
      Registry.Layout.CardOrderSelector.ApplyLayout(sequence);
      Registry.Layout.CardOrderSelectorVoid.ApplyLayout(sequence);
      Registry.Layout.GameModifiersDisplay.ApplyLayout(sequence);
      Registry.Layout.OnScreenStorage.ApplyLayout(sequence);
      Registry.Layout.AboveUserVoid.ApplyLayout(sequence);
      Registry.Layout.AboveEnemyVoid.ApplyLayout(sequence);

      Registry.DreamscapeService.ApplyLayouts(sequence);
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
            var layout = LayoutForObjectPosition(card.CardView.DestroyPosition);
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

    ObjectLayout LayoutForObjectPosition(ObjectPosition objectPosition) =>
      LayoutForPosition(objectPosition.Position);

    public ObjectLayout LayoutForPosition(Position position)
    {
      if (position.Enum == PositionEnum.Drawn)
      {
        return Registry.Layout.DrawnCardsPosition;
      }

      if (position.Enum == PositionEnum.Browser)
      {
        return Registry.Layout.Browser;
      }

      if (position.Enum == PositionEnum.DreamwellActivation)
      {
        return Registry.Layout.DreamwellActivation;
      }

      if (position.Enum == PositionEnum.GameModifier)
      {
        return Registry.Layout.GameModifiersDisplay;
      }

      if (position.Enum == PositionEnum.OnScreenStorage)
      {
        return Registry.Layout.OnScreenStorage;
      }

      if (position.Enum == PositionEnum.QuestDeck)
      {
        return Registry.DreamscapeLayout.QuestDeck;
      }

      if (position.Enum == PositionEnum.DraftPickDisplay)
      {
        return Registry.DreamscapeLayout.DraftPickLayout;
      }

      if (position.Enum == PositionEnum.ShopDisplay)
      {
        return Registry.DreamscapeLayout.ShopLayout;
      }

      if (position.Enum == PositionEnum.Offscreen)
      {
        return Registry.Layout.Offscreen;
      }

      if (position.Enum == PositionEnum.DreamsignDisplay)
      {
        return Registry.DreamscapeLayout.DreamsignDisplay;
      }

      if (position.Enum == PositionEnum.JourneyDisplay)
      {
        return Registry.DreamscapeLayout.JourneyChoiceDisplay;
      }

      if (position.PositionClass == null)
      {
        throw new InvalidOperationException($"Unknown layout position enum: ${position.Enum}");
      }

      if (position.PositionClass.InHand is { } inHand)
      {
        return inHand switch
        {
          DisplayPlayer.User => Registry.Layout.UserHand,
          DisplayPlayer.Enemy => Registry.Layout.EnemyHand,
          _ => throw Errors.UnknownEnumValue(inHand),
        };
      }

      if (position.PositionClass.InDeck is { } inDeck)
      {
        return inDeck switch
        {
          DisplayPlayer.User => Registry.Layout.UserDeck,
          DisplayPlayer.Enemy => Registry.Layout.EnemyDeck,
          _ => throw Errors.UnknownEnumValue(inDeck),
        };
      }

      if (position.PositionClass.OnBattlefield is { } onBattlefield)
      {
        return onBattlefield switch
        {
          DisplayPlayer.User => Registry.Layout.UserBattlefield,
          DisplayPlayer.Enemy => Registry.Layout.EnemyBattlefield,
          _ => throw Errors.UnknownEnumValue(onBattlefield),
        };
      }

      if (position.PositionClass.InVoid is { } inVoid)
      {
        return inVoid switch
        {
          DisplayPlayer.User => Registry.Layout.UserVoid,
          DisplayPlayer.Enemy => Registry.Layout.EnemyVoid,
          _ => throw Errors.UnknownEnumValue(inVoid),
        };
      }

      if (position.PositionClass.InPlayerStatus is { } inPlayerStatus)
      {
        return inPlayerStatus switch
        {
          DisplayPlayer.User => Registry.Layout.UserStatusDisplay,
          DisplayPlayer.Enemy => Registry.Layout.EnemyStatusDisplay,
          _ => throw Errors.UnknownEnumValue(inPlayerStatus),
        };
      }

      if (position.PositionClass.OnStack is { } onStack)
      {
        return onStack switch
        {
          StackType.Default => Registry.Layout.DefaultStack,
          StackType.TargetingUserBattlefield => Registry.Layout.TargetingUserStack,
          StackType.TargetingEnemyBattlefield => Registry.Layout.TargetingEnemyStack,
          StackType.TargetingBothBattlefields => Registry.Layout.TargetingBothStack,
          _ => throw Errors.UnknownEnumValue(onStack),
        };
      }

      if (position.PositionClass.InDreamwell is { } inDreamwell)
      {
        return inDreamwell switch
        {
          DisplayPlayer.User => Registry.Layout.UserDreamwell,
          DisplayPlayer.Enemy => Registry.Layout.EnemyDreamwell,
          _ => throw Errors.UnknownEnumValue(inDreamwell),
        };
      }

      if (position.PositionClass.HiddenWithinCard is { } cardId)
      {
        if (Cards.ContainsKey(cardId))
        {
          return GetCard(cardId).ContainedObjects;
        }
        else
        {
          return Registry.Layout.Offscreen;
        }
      }

      if (position.PositionClass.CardOrderSelector is { } cardOrderSelectorTarget)
      {
        return cardOrderSelectorTarget switch
        {
          CardOrderSelectionTargetDiscriminants.Deck => Registry.Layout.CardOrderSelector,
          CardOrderSelectionTargetDiscriminants.Void => Registry.Layout.CardOrderSelectorVoid,
          _ => throw Errors.UnknownEnumValue(cardOrderSelectorTarget),
        };
      }

      if (position.PositionClass.InBanished is { } _)
      {
        return Registry.Layout.Offscreen;
      }

      if (position.PositionClass.AboveVoid is { } aboveVoid)
      {
        return aboveVoid switch
        {
          DisplayPlayer.User => Registry.Layout.AboveUserVoid,
          DisplayPlayer.Enemy => Registry.Layout.AboveEnemyVoid,
          _ => throw Errors.UnknownEnumValue(aboveVoid),
        };
      }

      if (position.PositionClass.SiteDeck is { } siteDeck)
      {
        return Registry.DreamscapeService.SiteDeckLayout(siteDeck);
      }

      if (position.PositionClass.SiteNpc is { } siteNpc)
      {
        return Registry.DreamscapeService.SiteNpcLayout(siteNpc);
      }

      if (position.PositionClass.TemptingOfferDisplay is { } _)
      {
        return Registry.DreamscapeLayout.TemptingOfferDisplay;
      }

      if (position.PositionClass.DestroyedQuestCards is { } destroyedQuestCards)
      {
        return destroyedQuestCards switch
        {
          QuestEffectCardType.FullCard => Registry.DreamscapeLayout.DestroyedQuestCards,
          QuestEffectCardType.BattlefieldCard => Registry
            .DreamscapeLayout
            .DestroyedQuestCardsBattlefield,
          _ => throw Errors.UnknownEnumValue(destroyedQuestCards),
        };
      }

      if (position.PositionClass.QuestEffect is { } questEffect)
      {
        return questEffect switch
        {
          QuestEffectCardType.FullCard => Registry.DreamscapeLayout.QuestEffectPosition,
          QuestEffectCardType.BattlefieldCard => Registry
            .DreamscapeLayout
            .QuestEffectBattlefieldPosition,
          _ => throw Errors.UnknownEnumValue(questEffect),
        };
      }

      var json = JsonConvert.SerializeObject(position.PositionClass);
      throw new InvalidOperationException($"Unknown layout position: ${json}");
    }
  }
}
