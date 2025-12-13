#nullable enable

using System.Collections;
using System.Collections.Generic;
using System.Linq;
using Dreamtides.Prototype;
using Dreamtides.Schema;
using Dreamtides.Services;
using UnityEngine;

public class PrototypeQuestBattleFlow
{
  const string UserIdentityCardId = "identity-user";
  const string EnemyIdentityCardId = "identity-enemy";
  const string QuestDeckGroupKey = "quest";
  const string UserDreamsignsGroupKey = "dreamsigns";

  readonly Registry _registry;
  readonly PrototypeCards _prototypeCards;
  CardView? _userIdentityCard;

  public PrototypeQuestBattleFlow(Registry registry, PrototypeCards prototypeCards)
  {
    _registry = registry;
    _prototypeCards = prototypeCards;
  }

  public CardView GetOrCreateQuestUserIdentityCard(StudioType studioType)
  {
    if (_userIdentityCard == null)
    {
      _userIdentityCard = BuildUserIdentityCard(
        new ObjectPosition
        {
          Position = new Position { Enum = PositionEnum.QuestUserIdentityCard },
          SortingKey = 0,
        },
        studioType
      );
    }
    return _userIdentityCard;
  }

  public IEnumerator ApplyBattleStartupRoutine()
  {
    var layout = _registry.BattleLayout;
    var cameraPosition = layout.CameraPosition;
    _registry.MainCamera.transform.SetPositionAndRotation(
      cameraPosition.position,
      cameraPosition.rotation
    );
    _registry.CameraAdjuster.AdjustFieldOfView(layout.BattleCameraBounds);

    var enemyIdentityCardAtOrigin = BuildEnemyIdentityCardAtOrigin();
    var allCards = new List<CardView> { enemyIdentityCardAtOrigin };
    if (_userIdentityCard != null)
    {
      allCards.Add(_userIdentityCard);
    }

    allCards.AddRange(_prototypeCards.GetGroupCards(QuestDeckGroupKey));
    allCards.AddRange(_prototypeCards.GetGroupCards(UserDreamsignsGroupKey));

    var command = new UpdateQuestCommand { Quest = new QuestView { Cards = allCards } };
    yield return _registry.CardService.HandleUpdateQuestCards(command);

    yield return new WaitForSeconds(0.3f);

    var enemyIdentityCardAtDisplay = BuildEnemyIdentityCard();
    var animation = new MoveCardsWithCustomAnimationCommand
    {
      Animation = MoveCardsCustomAnimation.DefaultAnimation,
      Cards = new List<CardView> { enemyIdentityCardAtDisplay },
      Destination = new Position
      {
        PositionClass = new PositionClass
        {
          StartBattleDisplay = StartBattleDisplayType.EnemyIdentityCard,
        },
      },
      PauseDuration = new Milliseconds { MillisecondsValue = 0 },
      StaggerInterval = new Milliseconds { MillisecondsValue = 50 },
    };

    yield return _registry.CardAnimationService.HandleMoveCardsWithCustomAnimation(animation);

    var allIds = _registry.CardService.GetCardIds().ToList();
    var updateCards = new List<CardView>(allIds.Count);
    foreach (var id in allIds)
    {
      var card = _registry.CardService.GetCard(id);
      var source = card.CardView;
      if (id == EnemyIdentityCardId)
      {
        updateCards.Add(enemyIdentityCardAtDisplay);
      }
      else
      {
        updateCards.Add(source);
      }
    }

    var update = new UpdateQuestCommand { Quest = new QuestView { Cards = updateCards } };
    yield return _registry.CardService.HandleUpdateQuestCards(update);

    _registry.DreamscapeLayout.StartBattleLayout.ShowButton();
  }

  CardView BuildUserIdentityCard(ObjectPosition position, StudioType studioType)
  {
    return new CardView
    {
      Backless = true,
      CardFacing = CardFacing.FaceUp,
      CreatePosition = null,
      CreateSound = null,
      DestroyPosition = null,
      Id = UserIdentityCardId,
      Position = position,
      Prefab = CardPrefab.Identity,
      Revealed = new RevealedCardView
      {
        Actions = new CardActions(),
        CardType = "",
        Cost = null,
        Effects = new CardEffects(),
        Image = new DisplayImage
        {
          Prefab = new DisplayPrefabImage
          {
            Prefab = new PrefabAddress
            {
              Prefab = "Assets/Content/Characters/PirateCaptain/PirateCaptain.prefab",
            },
            StudioType = studioType,
          },
        },
        InfoZoomData = null,
        IsFast = false,
        Name = "Blackbeard",
        OutlineColor = null,
        Produced = null,
        RulesText = "At the end of your turn, if you played no characters this turn, draw a card.",
        Spark = null,
      },
      RevealedToOpponents = true,
    };
  }

  CardView BuildEnemyIdentityCardAtOrigin()
  {
    return new CardView
    {
      Backless = true,
      CardFacing = CardFacing.FaceUp,
      CreatePosition = null,
      CreateSound = null,
      DestroyPosition = null,
      Id = EnemyIdentityCardId,
      Position = new ObjectPosition
      {
        Position = new Position
        {
          PositionClass = new PositionClass { StartBattleCardOrigin = PrototypeQuest.BattleSiteId },
        },
        SortingKey = 1,
      },
      Prefab = CardPrefab.Identity,
      Revealed = new RevealedCardView
      {
        Actions = new CardActions(),
        CardType = "",
        Cost = null,
        Effects = new CardEffects(),
        Image = new DisplayImage
        {
          Prefab = new DisplayPrefabImage
          {
            Prefab = new PrefabAddress
            {
              Prefab = "Assets/Content/Characters/WarriorKing/WarriorKing.prefab",
            },
            StudioType = StudioType.EnemyIdentityCard,
          },
        },
        InfoZoomData = null,
        IsFast = false,
        Name = "The Black Knight",
        OutlineColor = null,
        Produced = null,
        RulesText = "Whenever you discard your second card in a turn, draw a card.",
        Spark = null,
      },
      RevealedToOpponents = true,
    };
  }

  CardView BuildEnemyIdentityCard()
  {
    return new CardView
    {
      Backless = true,
      CardFacing = CardFacing.FaceUp,
      CreatePosition = null,
      CreateSound = null,
      DestroyPosition = null,
      Id = EnemyIdentityCardId,
      Position = new ObjectPosition
      {
        Position = new Position
        {
          PositionClass = new PositionClass
          {
            StartBattleDisplay = StartBattleDisplayType.EnemyIdentityCard,
          },
        },
        SortingKey = 1,
      },
      Prefab = CardPrefab.Identity,
      Revealed = new RevealedCardView
      {
        Actions = new CardActions(),
        CardType = "",
        Cost = null,
        Effects = new CardEffects(),
        Image = new DisplayImage
        {
          Prefab = new DisplayPrefabImage
          {
            Prefab = new PrefabAddress
            {
              Prefab = "Assets/Content/Characters/WarriorKing/WarriorKing.prefab",
            },
            StudioType = StudioType.EnemyIdentityCard,
          },
        },
        InfoZoomData = null,
        IsFast = false,
        Name = "The Black Knight",
        OutlineColor = null,
        Produced = null,
        RulesText = "Whenever you discard your second card in a turn, draw a card.",
        Spark = null,
      },
      RevealedToOpponents = true,
    };
  }
}
