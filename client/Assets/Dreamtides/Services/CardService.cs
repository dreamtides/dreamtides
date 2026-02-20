#nullable enable

using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.CompilerServices;
using DG.Tweening;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Schema;
using Dreamtides.Utils;
using Newtonsoft.Json;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Services
{
  public class CardService : Service
  {
    [SerializeField]
    internal Card _cardPrefab = null!;

    [SerializeField]
    internal Card _eventCardPrefab = null!;

    [SerializeField]
    internal Card _tokenPrefab = null!;

    [SerializeField]
    internal Card _dreamwellPrefab = null!;

    [SerializeField]
    internal Card _identityCardPrefab = null!;

    [SerializeField]
    internal Card _enemyPrefab = null!;

    [SerializeField]
    internal Card _dreamsignPrefab = null!;

    [SerializeField]
    internal Card _iconCardPrefab = null!;

    [SerializeField]
    internal Card _journeyCardPrefab = null!;

    [SerializeField]
    internal Card _offerCostCardPrefab = null!;

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
      Registry.LoggingService.Log("CardService", $"Applying {cardViews.Count} card updates");

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
        card.ObjectPosition = cardView.Position;
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
          DisplayPlayer.User => Registry.BattleLayout.UserDeck,
          DisplayPlayer.Enemy => Registry.BattleLayout.EnemyDeck,
          _ => throw Errors.UnknownEnumValue(deck),
        };
      }

      if (id.Void is { } voidPile)
      {
        return voidPile switch
        {
          DisplayPlayer.User => Registry.BattleLayout.UserVoid,
          DisplayPlayer.Enemy => Registry.BattleLayout.EnemyVoid,
          _ => throw Errors.UnknownEnumValue(voidPile),
        };
      }

      if (id.Avatar is { } avatar)
      {
        return avatar switch
        {
          DisplayPlayer.User => Registry.BattleLayout.UserStatusDisplay,
          DisplayPlayer.Enemy => Registry.BattleLayout.EnemyStatusDisplay,
          _ => throw Errors.UnknownEnumValue(avatar),
        };
      }

      if (id.QuestObject is { } questObject)
      {
        return questObject switch
        {
          QuestObjectId.EssenceTotal => Registry.DreamscapeLayout.EssenceTotalWorldPosition,
          QuestObjectId.QuestDeck => Registry.DreamscapeLayout.QuestDeck,
          _ => throw Errors.UnknownEnumValue(questObject),
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
      if (Registry.BattleLayout.Browser.Objects.Count > 0)
      {
        Registry.BattleLayout.Browser.Show(Registry, sequence);
      }
      else
      {
        Registry.BattleLayout.Browser.Hide(Registry, sequence);
      }

      if (
        Registry.BattleLayout.CardOrderSelector.Objects.Count > 0
        || Registry.BattleLayout.CardOrderSelectorVoid.Objects.Count > 0
      )
      {
        Registry.BattleLayout.CardOrderSelector.Show(Registry, sequence);
      }
      else
      {
        Registry.BattleLayout.CardOrderSelector.Hide(Registry, sequence);
      }

      Registry.BattleLayout.UserHand.ApplyLayout(sequence);
      Registry.BattleLayout.EnemyHand.ApplyLayout(sequence);
      Registry.BattleLayout.UserDeck.ApplyLayout(sequence);
      Registry.BattleLayout.EnemyDeck.ApplyLayout(sequence);
      Registry.BattleLayout.UserVoid.ApplyLayout(sequence);
      Registry.BattleLayout.EnemyVoid.ApplyLayout(sequence);
      Registry.BattleLayout.UserStatusDisplay.ApplyLayout(sequence);
      Registry.BattleLayout.EnemyStatusDisplay.ApplyLayout(sequence);
      Registry.BattleLayout.UserBattlefield.ApplyLayout(sequence);
      Registry.BattleLayout.EnemyBattlefield.ApplyLayout(sequence);
      Registry.BattleLayout.DrawnCardsPosition.ApplyLayout(sequence);
      Registry.BattleLayout.DefaultStack.ApplyLayout(sequence);
      Registry.BattleLayout.TargetingUserStack.ApplyLayout(sequence);
      Registry.BattleLayout.TargetingEnemyStack.ApplyLayout(sequence);
      Registry.BattleLayout.TargetingBothStack.ApplyLayout(sequence);
      Registry.BattleLayout.Browser.ApplyLayout(sequence);
      Registry.BattleLayout.UserDreamwell.ApplyLayout(sequence);
      Registry.BattleLayout.EnemyDreamwell.ApplyLayout(sequence);
      Registry.BattleLayout.DreamwellActivation.ApplyLayout(sequence);
      Registry.BattleLayout.CardOrderSelector.ApplyLayout(sequence);
      Registry.BattleLayout.CardOrderSelectorVoid.ApplyLayout(sequence);
      Registry.BattleLayout.GameModifiersDisplay.ApplyLayout(sequence);
      Registry.BattleLayout.OnScreenStorage.ApplyLayout(sequence);
      Registry.BattleLayout.AboveUserVoid.ApplyLayout(sequence);
      Registry.BattleLayout.AboveEnemyVoid.ApplyLayout(sequence);

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
        return Registry.BattleLayout.DrawnCardsPosition;
      }

      if (position.Enum == PositionEnum.Browser)
      {
        return Registry.BattleLayout.Browser;
      }

      if (position.Enum == PositionEnum.DreamwellActivation)
      {
        return Registry.BattleLayout.DreamwellActivation;
      }

      if (position.Enum == PositionEnum.GameModifier)
      {
        return Registry.BattleLayout.GameModifiersDisplay;
      }

      if (position.Enum == PositionEnum.OnScreenStorage)
      {
        return Registry.BattleLayout.OnScreenStorage;
      }

      if (position.Enum == PositionEnum.QuestDeck)
      {
        return Registry.DreamscapeLayout.QuestDeck;
      }

      if (position.Enum == PositionEnum.QuestUserIdentityCard)
      {
        return Registry.DreamscapeLayout.QuestUserIdentityCard;
      }

      if (position.Enum == PositionEnum.QuestDeckBrowser)
      {
        return Registry.DreamscapeLayout.QuestDeckBrowser;
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
        return Registry.BattleLayout.Offscreen;
      }

      if (position.Enum == PositionEnum.DreamsignDisplay)
      {
        return Registry.DreamscapeLayout.DreamsignDisplay;
      }

      if (position.Enum == PositionEnum.JourneyDisplay)
      {
        return Registry.DreamscapeLayout.JourneyChoiceDisplay;
      }

      if (position.Enum == PositionEnum.QuestEffect)
      {
        return Registry.DreamscapeLayout.QuestEffectPosition;
      }

      if (position.Enum == PositionEnum.DestroyedQuestCards)
      {
        return Registry.DreamscapeLayout.DestroyedQuestCards;
      }

      if (position.PositionClass == null)
      {
        throw new InvalidOperationException($"Unknown layout position enum: ${position.Enum}");
      }

      if (position.PositionClass.InHand is { } inHand)
      {
        return inHand switch
        {
          DisplayPlayer.User => Registry.BattleLayout.UserHand,
          DisplayPlayer.Enemy => Registry.BattleLayout.EnemyHand,
          _ => throw Errors.UnknownEnumValue(inHand),
        };
      }

      if (position.PositionClass.InDeck is { } inDeck)
      {
        return inDeck switch
        {
          DisplayPlayer.User => Registry.BattleLayout.UserDeck,
          DisplayPlayer.Enemy => Registry.BattleLayout.EnemyDeck,
          _ => throw Errors.UnknownEnumValue(inDeck),
        };
      }

      if (position.PositionClass.OnBattlefield is { } onBattlefield)
      {
        return onBattlefield switch
        {
          DisplayPlayer.User => Registry.BattleLayout.UserBattlefield,
          DisplayPlayer.Enemy => Registry.BattleLayout.EnemyBattlefield,
          _ => throw Errors.UnknownEnumValue(onBattlefield),
        };
      }

      if (position.PositionClass.InVoid is { } inVoid)
      {
        return inVoid switch
        {
          DisplayPlayer.User => Registry.BattleLayout.UserVoid,
          DisplayPlayer.Enemy => Registry.BattleLayout.EnemyVoid,
          _ => throw Errors.UnknownEnumValue(inVoid),
        };
      }

      if (position.PositionClass.InPlayerStatus is { } inPlayerStatus)
      {
        return inPlayerStatus switch
        {
          DisplayPlayer.User => Registry.BattleLayout.UserStatusDisplay,
          DisplayPlayer.Enemy => Registry.BattleLayout.EnemyStatusDisplay,
          _ => throw Errors.UnknownEnumValue(inPlayerStatus),
        };
      }

      if (position.PositionClass.OnStack is { } onStack)
      {
        return onStack switch
        {
          StackType.Default => Registry.BattleLayout.DefaultStack,
          StackType.TargetingUserBattlefield => Registry.BattleLayout.TargetingUserStack,
          StackType.TargetingEnemyBattlefield => Registry.BattleLayout.TargetingEnemyStack,
          StackType.TargetingBothBattlefields => Registry.BattleLayout.TargetingBothStack,
          _ => throw Errors.UnknownEnumValue(onStack),
        };
      }

      if (position.PositionClass.InDreamwell is { } inDreamwell)
      {
        return inDreamwell switch
        {
          DisplayPlayer.User => Registry.BattleLayout.UserDreamwell,
          DisplayPlayer.Enemy => Registry.BattleLayout.EnemyDreamwell,
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
          return Registry.BattleLayout.Offscreen;
        }
      }

      if (position.PositionClass.CardOrderSelector is { } cardOrderSelectorTarget)
      {
        return cardOrderSelectorTarget switch
        {
          CardOrderSelectionTargetDiscriminants.Deck => Registry.BattleLayout.CardOrderSelector,
          CardOrderSelectionTargetDiscriminants.Void => Registry.BattleLayout.CardOrderSelectorVoid,
          _ => throw Errors.UnknownEnumValue(cardOrderSelectorTarget),
        };
      }

      if (position.PositionClass.InBanished is { } _)
      {
        return Registry.BattleLayout.Offscreen;
      }

      if (position.PositionClass.AboveVoid is { } aboveVoid)
      {
        return aboveVoid switch
        {
          DisplayPlayer.User => Registry.BattleLayout.AboveUserVoid,
          DisplayPlayer.Enemy => Registry.BattleLayout.AboveEnemyVoid,
          _ => throw Errors.UnknownEnumValue(aboveVoid),
        };
      }

      if (position.PositionClass.SiteDeck is { } siteDeck)
      {
        return Registry.DreamscapeService.SiteDeckLayout(siteDeck);
      }

      if (position.PositionClass.SiteNpc is { } siteNpc)
      {
        return Registry.DreamscapeService.SiteCharacterOwnedLayout(siteNpc);
      }

      if (position.PositionClass.TemptingOfferDisplay is { } _)
      {
        return Registry.DreamscapeLayout.TemptingOfferDisplay;
      }

      if (position.PositionClass.StartBattleCardOrigin is { } siteId)
      {
        return Registry.DreamscapeService.SiteCardOriginLayout(siteId);
      }

      if (position.PositionClass.StartBattleDisplay is { } _)
      {
        return Registry.DreamscapeLayout.StartBattleLayout;
      }

      var json = JsonConvert.SerializeObject(position.PositionClass);
      throw new InvalidOperationException($"Unknown layout position: ${json}");
    }
  }
}
