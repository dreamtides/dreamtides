#nullable enable

using System;
using System.Collections;
using System.Collections.Generic;
using Dreamtides.Schema;
using Dreamtides.Services;
using UnityEngine;

public class PrototypeQuestBattleFlow
{
  const string UserIdentityCardId = "identity-user";
  const string EnemyIdentityCardId = "identity-enemy";

  readonly Registry _registry;

  public PrototypeQuestBattleFlow(Registry registry)
  {
    _registry = registry;
  }

  public IEnumerator ApplyBattleStartupRoutine()
  {
    Debug.Log("battle opened");
    var layout = _registry.BattleLayout;
    var cameraPosition = layout.CameraPosition;
    _registry.MainCamera.transform.SetPositionAndRotation(
      cameraPosition.position,
      cameraPosition.rotation
    );
    _registry.CameraAdjuster.AdjustFieldOfView(layout.BattleCameraBounds);

    var identityCards = new List<CardView> { BuildUserIdentityCard(), BuildEnemyIdentityCard() };
    var command = new UpdateQuestCommand { Quest = new QuestView { Cards = identityCards } };
    yield return _registry.CardService.HandleUpdateQuestCards(command);
  }

  CardView BuildUserIdentityCard()
  {
    return new CardView
    {
      Backless = true,
      CardFacing = CardFacing.FaceUp,
      CreatePosition = null,
      CreateSound = null,
      DestroyPosition = null,
      Id = UserIdentityCardId,
      Position = new ObjectPosition
      {
        Position = new Position { Enum = PositionEnum.QuestEffect },
        SortingKey = 0,
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
              Prefab = "Assets/Content/Characters/PirateCaptain/PirateCaptain.prefab",
            },
            StudioType = StudioType.UserIdentityCard,
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
        Position = new Position { Enum = PositionEnum.QuestEffect },
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
