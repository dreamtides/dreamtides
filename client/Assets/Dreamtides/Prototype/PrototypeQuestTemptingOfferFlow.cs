#nullable enable

using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using Dreamtides.Masonry;
using Dreamtides.Prototype;
using Dreamtides.Schema;
using Dreamtides.Services;
using UnityEngine;

public class PrototypeQuestTemptingOfferFlow
{
  const string TemptingOfferGroupKey = "tempting-offer";
  const string TemptingOfferScenarioName = "tempting-offer";
  const string TemptingOfferButtonLabel = "Accept";
  const int TemptingOfferCardsPerOffer = 2;
  const int TemptingOfferMaxOffers = 2;

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
      TryDissolveJourneyCard(offerNumber);
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

    var button = _registry.DreamscapeService.CloseButton.GetComponent<CloseBrowserButton>();
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

  void TryDissolveJourneyCard(int offerNumber)
  {
    var journeyCardId = GetJourneyCardId(offerNumber);
    if (journeyCardId == null)
    {
      Debug.LogWarning($"Unable to find journey card for tempting offer {offerNumber}");
      return;
    }
    ApplyJourneyDissolve(journeyCardId);
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

  void ApplyJourneyDissolve(string journeyCardId)
  {
    var dissolveCommand = new DissolveCardCommand
    {
      Color = Mason.MakeColor("#FFC107"),
      Material = new MaterialAddress { Material = "Assets/Content/Dissolves/Dissolve15.mat" },
      Reverse = false,
      Sound = new AudioClipAddress
      {
        AudioClip =
          "Assets/ThirdParty/WowSound/RPG Magic Sound Effects Pack 3/Fire Magic/RPG3_FireMagicBall_LightImpact03.wav",
      },
      Target = journeyCardId,
    };
    _registry.EffectService.StartCoroutine(
      _registry.EffectService.HandleDissolveCommand(dissolveCommand)
    );
  }
}
