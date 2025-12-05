#nullable enable

using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using Dreamtides.Masonry;
using Dreamtides.Prototype;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.Utils;
using UnityEngine;

public class PrototypeQuestTemptingOfferFlow
{
  const string TemptingOfferGroupKey = "tempting-offer";
  const string TemptingOfferScenarioName = "tempting-offer";
  const string TemptingOfferButtonLabel = "Accept";
  const int TemptingOfferCardsPerOffer = 2;
  const int TemptingOfferMaxOffers = 2;
  const int ImmolateReverseDissolveDelayStepMs = 300;
  const float ImmolateReverseDissolveDurationSeconds = 1f;

  readonly Registry _registry;
  readonly PrototypeCards _prototypeCards;
  readonly Func<CreateOrUpdateCardsRequest, bool, IEnumerator> _createOrUpdateCards;
  readonly Guid _temptingOfferSiteId;
  readonly string[] _spritePaths =
  {
    "Assets/ThirdParty/GameAssets/CardImages/Circular/shutterstock_2155438699.png",
    "Assets/ThirdParty/GameAssets/CardImages/Circular/shutterstock_1486924805.png",
    "Assets/ThirdParty/GameAssets/CardImages/Circular/shutterstock_2421338077.png",
    "Assets/ThirdParty/GameAssets/CardImages/Circular/shutterstock_2419795157.png",
  };

  public PrototypeQuestTemptingOfferFlow(
    Registry registry,
    PrototypeCards prototypeCards,
    Func<CreateOrUpdateCardsRequest, bool, IEnumerator> createOrUpdateCards,
    Guid temptingOfferSiteId
  )
  {
    _registry = registry;
    _prototypeCards = prototypeCards;
    _createOrUpdateCards = createOrUpdateCards;
    _temptingOfferSiteId = temptingOfferSiteId;
  }

  public bool IsTemptingOfferAction(string action) => action == TemptingOfferScenarioName;

  public void HandleTemptingOfferSelection(string clickedId)
  {
    if (int.TryParse(clickedId, out var offerNumber))
    {
      Debug.Log($"Tempting offer accepted for option {offerNumber}");
      _registry.StartCoroutine(ResolveTemptingOfferSelection(offerNumber));
    }
    else
    {
      Debug.Log($"Tempting offer accepted: {clickedId}");
    }
  }

  public IEnumerator PrepareTemptingOfferCards()
  {
    _prototypeCards.ResetGroup(TemptingOfferGroupKey);
    var request = new CreateOrUpdateCardsRequest
    {
      Count = 4,
      Position = new ObjectPosition
      {
        Position = new Position
        {
          PositionClass = new PositionClass { SiteNpc = _temptingOfferSiteId },
        },
        SortingKey = 1,
      },
      Revealed = true,
      GroupKey = TemptingOfferGroupKey,
      Overrides = BuildTemptingOfferOverrides(),
    };
    yield return _createOrUpdateCards(request, true);
  }

  public IEnumerator ShowTemptingOfferCards()
  {
    var request = new CreateOrUpdateCardsRequest
    {
      Count = 4,
      Position = new ObjectPosition
      {
        Position = new Position
        {
          PositionClass = new PositionClass
          {
            TemptingOfferDisplay = BuildTemptingOfferPosition(0, TemptingOfferType.Cost),
          },
        },
        SortingKey = 1,
      },
      Revealed = true,
      GroupKey = TemptingOfferGroupKey,
      Overrides = BuildTemptingOfferOverrides(),
    };
    var allCards = _prototypeCards.CreateOrUpdateCards(request);
    ApplyTemptingOfferPresentation(allCards);

    yield return new WaitForSeconds(0.3f);

    var animation = new MoveCardsWithCustomAnimationCommand
    {
      Animation = MoveCardsCustomAnimation.DefaultAnimation,
      Cards = allCards.Take(4).ToList(),
      Destination = new Position
      {
        PositionClass = new PositionClass
        {
          TemptingOfferDisplay = BuildTemptingOfferPosition(0, TemptingOfferType.Cost),
        },
      },
      PauseDuration = new Milliseconds { MillisecondsValue = 0 },
      StaggerInterval = new Milliseconds { MillisecondsValue = 50 },
    };

    yield return _registry.CardAnimationService.HandleMoveCardsWithCustomAnimation(animation);

    yield return _createOrUpdateCards(request, true);

    var button = _registry.DreamscapeService.CloseSiteButton.GetComponent<CloseBrowserButton>();
    button.CloseAction = new GameAction
    {
      GameActionClass = new GameActionClass
      {
        DebugAction = new DebugAction
        {
          DebugActionClass = new DebugActionClass
          {
            ApplyTestScenarioAction = "closeTemptingOffer",
          },
        },
      },
    };
  }

  public TemptingOfferView? BuildTemptingOfferView(CreateOrUpdateCardsRequest request)
  {
    if (request.GroupKey != TemptingOfferGroupKey)
    {
      return null;
    }
    var offerCount = Math.Min(
      TemptingOfferMaxOffers,
      Math.Max(0, (request.Count + TemptingOfferCardsPerOffer - 1) / TemptingOfferCardsPerOffer)
    );
    if (offerCount == 0)
    {
      return null;
    }
    return new TemptingOfferView { Actions = BuildTemptingOfferActions(offerCount) };
  }

  List<TemptingOfferAction> BuildTemptingOfferActions(int offerCount)
  {
    var actions = new List<TemptingOfferAction>(offerCount);
    for (var number = 0; number < offerCount; number++)
    {
      actions.Add(
        new TemptingOfferAction { Number = number, Button = BuildTemptingOfferButton(number) }
      );
    }
    return actions;
  }

  ButtonView BuildTemptingOfferButton(int offerNumber) =>
    new ButtonView
    {
      Label = TemptingOfferButtonLabel,
      Action = BuildTemptingOfferOnClick(offerNumber),
    };

  OnClickUnion BuildTemptingOfferOnClick(int offerNumber) =>
    new OnClickUnion
    {
      OnClickClass = new OnClickClass
      {
        DebugAction = new DebugAction
        {
          DebugActionClass = new DebugActionClass
          {
            ApplyTestScenarioAction = $"{TemptingOfferScenarioName}/{offerNumber}",
          },
        },
      },
    };

  List<CardOverride> BuildTemptingOfferOverrides()
  {
    var overrides = new List<CardOverride>();
    overrides.Add(
      new CardOverride
      {
        Index = 0,
        Prefab = CardPrefab.Journey,
        Name = "Journey",
        Rules = string.Empty,
        CardType = string.Empty,
        IsFast = false,
        SpritePath = _spritePaths[0],
      }
    );
    overrides.Add(
      new CardOverride
      {
        Index = 1,
        Prefab = CardPrefab.OfferCost,
        Name = "Cost",
        Rules = string.Empty,
        CardType = string.Empty,
        IsFast = false,
        SpritePath = _spritePaths[1],
      }
    );
    overrides.Add(
      new CardOverride
      {
        Index = 2,
        Prefab = CardPrefab.Journey,
        Name = "Journey",
        Rules = string.Empty,
        CardType = string.Empty,
        IsFast = false,
        SpritePath = _spritePaths[2],
      }
    );
    overrides.Add(
      new CardOverride
      {
        Index = 3,
        Prefab = CardPrefab.OfferCost,
        Name = "Cost",
        Rules = string.Empty,
        CardType = string.Empty,
        IsFast = false,
        SpritePath = _spritePaths[3],
      }
    );
    return overrides;
  }

  void ApplyTemptingOfferPresentation(List<CardView> cards)
  {
    var groupCards = cards
      .Where(card => card.Id.StartsWith($"{TemptingOfferGroupKey}-"))
      .Take(4)
      .ToList();
    for (var i = 0; i < groupCards.Count; i++)
    {
      var type = i % 2 == 0 ? TemptingOfferType.Journey : TemptingOfferType.Cost;
      var name = type == TemptingOfferType.Cost ? "Cost" : "Journey";
      ConfigureTemptingOfferCard(groupCards[i], name, type, _spritePaths[i], i);
    }
  }

  static TemptingOfferPosition BuildTemptingOfferPosition(
    int sortingKey,
    TemptingOfferType displayType
  )
  {
    var offerNumber = sortingKey / 2;
    return new TemptingOfferPosition { Number = offerNumber, OfferType = displayType };
  }

  static void ConfigureTemptingOfferCard(
    CardView card,
    string name,
    TemptingOfferType displayType,
    string spritePath,
    int sortingKey
  )
  {
    card.Backless = true;
    card.CardFacing = CardFacing.FaceUp;
    card.Position = new ObjectPosition
    {
      Position = new Position
      {
        PositionClass = new PositionClass
        {
          TemptingOfferDisplay = BuildTemptingOfferPosition(sortingKey, displayType),
        },
      },
      SortingKey = sortingKey,
    };
    if (card.Revealed != null)
    {
      card.Revealed.Name = name;
      card.Revealed.CardType = string.Empty;
      card.Revealed.Cost = null;
      card.Revealed.Produced = null;
      card.Revealed.RulesText = string.Empty;
      card.Revealed.Spark = null;
      card.Revealed.IsFast = false;
      card.Revealed.Image = new DisplayImage { Sprite = new SpriteAddress { Sprite = spritePath } };
      card.Revealed.OutlineColor = null;
      card.Revealed.Actions = card.Revealed.Actions ?? new CardActions();
      card.Revealed.Actions.ButtonAttachment = null;
      card.Revealed.Actions.CanPlay = null;
      card.Revealed.Actions.CanSelectOrder = null;
      card.Revealed.Actions.OnClick = null;
      card.Revealed.Actions.OnPlaySound = null;
      card.Revealed.Actions.PlayEffectPreview = null;
    }
  }

  string? GetJourneyCardId(int offerNumber)
  {
    if (offerNumber < 0 || offerNumber >= TemptingOfferMaxOffers)
    {
      return null;
    }
    var index = offerNumber * TemptingOfferCardsPerOffer;
    return $"{TemptingOfferGroupKey}-{index + 1}";
  }

  string? GetCostCardId(int offerNumber)
  {
    if (offerNumber < 0 || offerNumber >= TemptingOfferMaxOffers)
    {
      return null;
    }
    var index = offerNumber * TemptingOfferCardsPerOffer + 1;
    return $"{TemptingOfferGroupKey}-{index + 1}";
  }

  IEnumerator PlayJourneyDissolve(string journeyCardId) =>
    PlayDissolve(journeyCardId, reverse: false);

  IEnumerator PlayDissolve(
    string cardId,
    bool reverse,
    string color = "#FFC107",
    long startDelay = 0
  )
  {
    var dissolveCommand = BuildDissolveCommand(cardId, reverse, color, startDelay);
    yield return _registry.EffectService.HandleDissolveCommand(dissolveCommand);
  }

  DissolveCardCommand BuildDissolveCommand(
    string cardId,
    bool reverse,
    string color = "#FFC107",
    long startDelay = 0
  ) =>
    new DissolveCardCommand
    {
      Color = Mason.MakeColor(color),
      Material = new MaterialAddress { Material = "Assets/Content/Dissolves/Dissolve15.mat" },
      Reverse = reverse,
      DissolveSpeed = 1f,
      Sound = new AudioClipAddress
      {
        AudioClip =
          "Assets/ThirdParty/WowSound/RPG Magic Sound Effects Pack 3/Fire Magic/RPG3_FireMagicBall_LightImpact03.wav",
      },
      Target = cardId,
      StartDelay = new Milliseconds { MillisecondsValue = startDelay },
      KeepDissolveMaterial = true,
    };

  IEnumerator ResolveTemptingOfferSelection(int offerNumber)
  {
    var journeyCardId = GetJourneyCardId(offerNumber);
    if (journeyCardId == null)
    {
      Debug.LogWarning($"Unable to determine journey card for tempting offer {offerNumber}");
      yield break;
    }
    var costCardId = GetCostCardId(offerNumber);
    if (costCardId == null)
    {
      Debug.LogWarning($"Unable to determine cost card for tempting offer {offerNumber}");
    }
    if (!TryBuildJourneyQuestUpdate(journeyCardId, offerNumber, out var updateCards))
    {
      yield break;
    }
    var update = new UpdateQuestCommand { Quest = new QuestView { Cards = updateCards } };
    _registry.DreamscapeService.HideCloseSiteButton();
    _registry.DreamscapeLayout.TemptingOfferDisplay.HideAcceptButtons();
    var sequence = TweenUtils.Sequence("TemptingOfferJourneyResolution");
    yield return _registry.CardService.HandleUpdateQuestCards(update, sequence);
    _prototypeCards.UpdateGroupCards(TemptingOfferGroupKey, updateCards);
    yield return PlayJourneyDissolve(journeyCardId);
    if (
      TryBuildPostDissolveQuestUpdate(
        journeyCardId,
        out var postDissolveUpdate,
        out var immolateCardIds,
        out var maxReverseDissolveDelayMs
      )
    )
    {
      var postUpdate = new UpdateQuestCommand
      {
        Quest = new QuestView { Cards = postDissolveUpdate },
      };
      yield return _registry.CardService.HandleUpdateQuestCards(postUpdate);
      _prototypeCards.UpdateGroupCards(TemptingOfferGroupKey, postDissolveUpdate);
      if (immolateCardIds.Count > 0)
      {
        var totalWaitSeconds =
          (maxReverseDissolveDelayMs / 1000f) + ImmolateReverseDissolveDurationSeconds;
        yield return new WaitForSeconds(totalWaitSeconds);
        yield return MoveImmolateCardsToQuestDeck(immolateCardIds);
      }
      if (!string.IsNullOrEmpty(costCardId))
      {
        yield return BringCostCardForwardAndDissolve(costCardId);
      }
    }
  }

  bool TryBuildJourneyQuestUpdate(
    string journeyCardId,
    int selectedOfferNumber,
    out List<CardView> updateCards
  )
  {
    updateCards = new List<CardView>();
    var existing = _registry.CardService.GetCardIfExists(journeyCardId);
    if (existing == null)
    {
      Debug.LogWarning($"Unable to find journey card with id {journeyCardId}");
      return false;
    }
    var allIds = _registry.CardService.GetCardIds().ToList();
    var destroyedBase = _registry.DreamscapeLayout.DestroyedQuestCards.Objects.Count;
    var destroyedOffset = 0;
    var sortingKey = _registry.DreamscapeLayout.QuestEffectPosition.Objects.Count;
    updateCards = new List<CardView>(allIds.Count);
    foreach (var id in allIds)
    {
      var card = _registry.CardService.GetCard(id);
      var source = card.CardView;
      if (id == journeyCardId)
      {
        updateCards.Add(
          PrototypeQuestCardViewFactory.CloneCardViewWithPosition(
            source,
            new Position { Enum = PositionEnum.QuestEffect },
            sortingKey
          )
        );
      }
      else if (
        TryGetTemptingOfferIndex(id, out var cardIndex)
        && GetOfferNumber(cardIndex) != selectedOfferNumber
      )
      {
        var destroyedSorting = destroyedBase + destroyedOffset;
        destroyedOffset++;
        updateCards.Add(
          PrototypeQuestCardViewFactory.CloneCardViewWithPosition(
            source,
            new Position { Enum = PositionEnum.DestroyedQuestCards },
            destroyedSorting
          )
        );
      }
      else
      {
        updateCards.Add(source);
      }
    }
    return true;
  }

  bool TryGetTemptingOfferIndex(string cardId, out int index)
  {
    index = -1;
    var prefix = $"{TemptingOfferGroupKey}-";
    if (!cardId.StartsWith(prefix, StringComparison.Ordinal))
    {
      return false;
    }
    var suffix = cardId.Substring(prefix.Length);
    if (!int.TryParse(suffix, out var parsed) || parsed <= 0)
    {
      return false;
    }
    index = parsed - 1;
    return true;
  }

  static int GetOfferNumber(int cardIndex) => cardIndex / TemptingOfferCardsPerOffer;

  bool TryBuildPostDissolveQuestUpdate(
    string journeyCardId,
    out List<CardView> updateCards,
    out List<string> spawnedImmolateCardIds,
    out long maxReverseDissolveDelayMs
  )
  {
    spawnedImmolateCardIds = new List<string>();
    maxReverseDissolveDelayMs = 0;
    var existing = _registry.CardService.GetCardIfExists(journeyCardId);
    if (existing == null)
    {
      Debug.LogWarning($"Unable to find journey card with id {journeyCardId} after dissolve");
      updateCards = new List<CardView>();
      return false;
    }
    var allIds = _registry.CardService.GetCardIds().ToList();
    var destroyedBase = _registry.DreamscapeLayout.DestroyedQuestCards.Objects.Count;
    var destroyedOffset = 0;
    updateCards = new List<CardView>(allIds.Count + 1);
    foreach (var id in allIds)
    {
      var card = _registry.CardService.GetCard(id);
      var source = card.CardView;
      if (id == journeyCardId)
      {
        var destroyedSorting = destroyedBase + destroyedOffset;
        destroyedOffset++;
        updateCards.Add(
          PrototypeQuestCardViewFactory.CloneCardViewWithPosition(
            source,
            new Position { Enum = PositionEnum.DestroyedQuestCards },
            destroyedSorting
          )
        );
      }
      else
      {
        updateCards.Add(source);
      }
    }
    var questEffectSorting = _registry.DreamscapeLayout.QuestEffectPosition.Objects.Count;
    for (var i = 0; i < 3; i++)
    {
      var startDelay = i * ImmolateReverseDissolveDelayStepMs;
      if (startDelay > maxReverseDissolveDelayMs)
      {
        maxReverseDissolveDelayMs = startDelay;
      }
      var immolateCard = BuildImmolateQuestEffectCard(
        questEffectSorting + i,
        out var immolateCardId,
        startDelay: startDelay
      );
      spawnedImmolateCardIds.Add(immolateCardId);
      updateCards.Add(immolateCard);
    }
    return true;
  }

  CardView BuildImmolateQuestEffectCard(int sortingKey, out string cardId, long startDelay = 0)
  {
    var generatedId = $"quest-effect-immolate-{Guid.NewGuid():N}";
    cardId = generatedId;
    return new CardView
    {
      Backless = false,
      CardFacing = CardFacing.FaceUp,
      CreatePosition = null,
      CreateSound = null,
      DestroyPosition = null,
      Id = generatedId,
      Position = new ObjectPosition
      {
        Position = new Position { Enum = PositionEnum.QuestEffect },
        SortingKey = sortingKey,
      },
      Prefab = CardPrefab.Event,
      Revealed = new RevealedCardView
      {
        Actions = new CardActions(),
        CardType = "Event",
        Cost = "2",
        Effects = new CardEffects
        {
          ReverseDissolveOnAppear = BuildDissolveCommand(cardId, true, "#81D4FA", startDelay),
        },
        Image = new DisplayImage
        {
          Sprite = new SpriteAddress
          {
            Sprite = "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1907487244.png",
          },
        },
        InfoZoomData = null,
        IsFast = true,
        Name = "Immolate",
        OutlineColor = null,
        Produced = null,
        RulesText = "{Dissolve} an enemy character.",
        Spark = null,
      },
      RevealedToOpponents = false,
    };
  }

  IEnumerator BringCostCardForwardAndDissolve(string costCardId)
  {
    if (!TryBuildCostQuestUpdate(costCardId, out var updateCards))
    {
      yield break;
    }
    var update = new UpdateQuestCommand { Quest = new QuestView { Cards = updateCards } };
    var sequence = TweenUtils.Sequence("TemptingOfferCostAdvance");
    yield return _registry.CardService.HandleUpdateQuestCards(update, sequence);
    _prototypeCards.UpdateGroupCards(TemptingOfferGroupKey, updateCards);
    yield return FireCostProjectile(costCardId);
    yield return PlayDissolve(costCardId, false, "#B39DDB");
    if (TryBuildCostPostDissolveUpdate(costCardId, out var postUpdateCards))
    {
      var postUpdate = new UpdateQuestCommand { Quest = new QuestView { Cards = postUpdateCards } };
      yield return _registry.CardService.HandleUpdateQuestCards(postUpdate);
      _prototypeCards.UpdateGroupCards(TemptingOfferGroupKey, postUpdateCards);
    }
  }

  IEnumerator MoveImmolateCardsToQuestDeck(List<string> immolateCardIds)
  {
    var allIds = _registry.CardService.GetCardIds().ToList();
    var updateCards = new List<CardView>(allIds.Count);
    var immolateSet = immolateCardIds.ToHashSet();
    var deckBase = _registry.DreamscapeLayout.QuestDeck.Objects.Count;
    var deckOffset = 0;
    var questDeckClones = new List<CardView>();
    foreach (var id in allIds)
    {
      var card = _registry.CardService.GetCard(id);
      var source = card.CardView;
      if (immolateSet.Contains(id))
      {
        var clone = PrototypeQuestCardViewFactory.CloneCardViewWithPositionHidden(
          source,
          new Position { Enum = PositionEnum.QuestDeck },
          deckBase + deckOffset
        );
        updateCards.Add(clone);
        questDeckClones.Add(clone);
        deckOffset++;
      }
      else
      {
        updateCards.Add(source);
      }
    }
    if (questDeckClones.Count == 0)
    {
      yield break;
    }

    var command = new MoveCardsWithCustomAnimationCommand
    {
      Animation = MoveCardsCustomAnimation.MoveToQuestDeckOrDestroy,
      Cards = questDeckClones,
      Destination = new Position { Enum = PositionEnum.QuestDeck },
      PauseDuration = new Milliseconds { MillisecondsValue = 0 },
      StaggerInterval = new Milliseconds { MillisecondsValue = 500 },
    };
    yield return _registry.CardAnimationService.HandleMoveCardsWithCustomAnimation(command);

    var update = new UpdateQuestCommand { Quest = new QuestView { Cards = updateCards } };
    yield return _registry.CardService.HandleUpdateQuestCards(update);
    _prototypeCards.UpdateGroupCards(TemptingOfferGroupKey, updateCards);
  }

  IEnumerator FireCostProjectile(string costCardId)
  {
    var command = new FireProjectileCommand
    {
      HideOnHit = false,
      Projectile = new ProjectileAddress
      {
        Projectile =
          "Assets/ThirdParty/Hovl Studio/AAA Projectiles Vol 1/Prefabs/Dreamtides/Projectile 6 blue fire.prefab",
      },
      SourceId = new GameObjectId { CardId = costCardId },
      TargetId = new GameObjectId { QuestObject = QuestObjectId.EssenceTotal },
      FireSound = new AudioClipAddress
      {
        AudioClip =
          "Assets/ThirdParty/WowSound/RPG Magic Sound Effects Pack 3/Water Magic/RPG3_WaterMagic_Cast01.wav",
      },
      ImpactSound = new AudioClipAddress
      {
        AudioClip =
          "Assets/ThirdParty/WowSound/RPG Magic Sound Effects Pack 3/Water Magic/RPG3_WaterMagic_Impact03.wav",
      },
      TravelDuration = new Milliseconds { MillisecondsValue = 600 },
      ScaleOverride = 1.0f,
    };
    yield return _registry.EffectService.HandleFireProjectileCommand(command);
  }

  bool TryBuildCostQuestUpdate(string costCardId, out List<CardView> updateCards)
  {
    updateCards = new List<CardView>();
    var existing = _registry.CardService.GetCardIfExists(costCardId);
    if (existing == null)
    {
      Debug.LogWarning($"Unable to find cost card with id {costCardId}");
      return false;
    }
    var allIds = _registry.CardService.GetCardIds().ToList();
    var sortingKey = _registry.DreamscapeLayout.QuestEffectPosition.Objects.Count;
    updateCards = new List<CardView>(allIds.Count);
    foreach (var id in allIds)
    {
      var card = _registry.CardService.GetCard(id);
      var source = card.CardView;
      if (id == costCardId)
      {
        updateCards.Add(
          PrototypeQuestCardViewFactory.CloneCardViewWithPosition(
            source,
            new Position { Enum = PositionEnum.QuestEffect },
            sortingKey
          )
        );
      }
      else
      {
        updateCards.Add(source);
      }
    }
    return true;
  }

  bool TryBuildCostPostDissolveUpdate(string costCardId, out List<CardView> updateCards)
  {
    updateCards = new List<CardView>();
    var existing = _registry.CardService.GetCardIfExists(costCardId);
    if (existing == null)
    {
      Debug.LogWarning($"Unable to find cost card with id {costCardId} after dissolve");
      return false;
    }
    var allIds = _registry.CardService.GetCardIds().ToList();
    var destroyedBase = _registry.DreamscapeLayout.DestroyedQuestCards.Objects.Count;
    var destroyedOffset = 0;
    updateCards = new List<CardView>(allIds.Count);
    foreach (var id in allIds)
    {
      var card = _registry.CardService.GetCard(id);
      var source = card.CardView;
      if (id == costCardId)
      {
        updateCards.Add(
          PrototypeQuestCardViewFactory.CloneCardViewWithPosition(
            source,
            new Position { Enum = PositionEnum.DestroyedQuestCards },
            destroyedBase + destroyedOffset
          )
        );
        destroyedOffset++;
      }
      else
      {
        updateCards.Add(source);
      }
    }
    return true;
  }
}
