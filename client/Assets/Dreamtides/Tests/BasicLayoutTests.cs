using System.Collections;
using NUnit.Framework;
using UnityEngine.TestTools;
using Dreamtides.Services;
using Dreamtides.Utils;
using Dreamtides.Components;
using Dreamtides.Schema;
using System.Collections.Generic;
using System;
using System.Linq;

namespace Dreamtides.Tests
{
  public class BasicLayoutTests : IntegrationTest
  {
    [TearDown]
    public void TearDown()
    {
      Registry.TestConfiguration = null;
    }

    [UnityTest]
    public IEnumerator TestBasicLayout()
    {
      yield return Connect();
      yield return PerformAction(SetupFullLayoutAction());
      AssertBoxColliderIsOnScreen(GetBoxCollider(Registry.Layout.UserDeck), "User deck is not on screen");
      AssertBoxColliderIsOnScreen(GetBoxCollider(Registry.Layout.EnemyDeck), "Enemy deck is not on screen");
      AssertBoxColliderIsOnScreen(GetBoxCollider(Registry.Layout.UserVoid), "User void is not on screen");
      AssertBoxColliderIsOnScreen(GetBoxCollider(Registry.Layout.EnemyVoid), "Enemy void is not on screen");

      foreach (var displayable in Registry.Layout.UserHand.Objects)
      {
        var card = ComponentUtils.Get<Card>(displayable);
        AssertSpriteIsOnScreen(card._costBackground, $"Energy Cost of {card.Id}");
      }

      yield return EndTest();
    }

    /// <summary>
    /// Returns a DebugAction::ApplyActionList<> action that sets up a full
    /// layout.
    /// </summary>
    ///
    /// <remarks>
    /// This includes:
    /// - 10 cards in each player's hand.
    /// - 8 cards on each player's battlefield.
    /// - 10 cards in each player's void.
    /// </remarks>
    static GameAction SetupFullLayoutAction()
    {
      return new GameAction
      {
        GameActionClass = new GameActionClass
        {
          DebugAction = new DebugAction
          {
            DebugActionClass = new DebugActionClass
            {
              ApplyActionList = AddCardsToHand(DisplayPlayer.User, 10)
                .Concat(AddCardsToHand(DisplayPlayer.Enemy, 10))
                .Concat(AddCardsToBattlefield(DisplayPlayer.User, 8))
                .Concat(AddCardsToBattlefield(DisplayPlayer.Enemy, 8))
                .Concat(AddCardsToVoid(DisplayPlayer.User, 10))
                .Concat(AddCardsToVoid(DisplayPlayer.Enemy, 10))
                .ToList()
            }
          }
        }
      };
    }

    static List<DebugBattleAction> AddCardsToHand(DisplayPlayer player, int count)
    {
      var list = new List<DebugBattleAction>();
      for (var i = 0; i < count; i++)
      {
        list.Add(AddCardToHand(player));
      }
      return list;
    }

    static DebugBattleAction AddCardToHand(DisplayPlayer player)
    {
      var name = player switch
      {
        DisplayPlayer.User => PlayerName.One,
        DisplayPlayer.Enemy => PlayerName.Two,
        _ => throw new IndexOutOfRangeException("Invalid player")
      };

      return new DebugBattleAction
      {
        AddCardToHand = new AddCardToHand
        {
          Card = CardName.MinstrelOfFallingLight,
          Player = name,
        }
      };
    }

    static List<DebugBattleAction> AddCardsToBattlefield(DisplayPlayer player, int count)
    {
      var list = new List<DebugBattleAction>();
      for (var i = 0; i < count; i++)
      {
        list.Add(AddCardToBattlefield(player));
      }
      return list;
    }

    static DebugBattleAction AddCardToBattlefield(DisplayPlayer player)
    {
      var name = player switch
      {
        DisplayPlayer.User => PlayerName.One,
        DisplayPlayer.Enemy => PlayerName.Two,
        _ => throw new IndexOutOfRangeException("Invalid player")
      };

      return new DebugBattleAction
      {
        AddCardToBattlefield = new AddCardToBattlefield
        {
          Card = CardName.MinstrelOfFallingLight,
          Player = name,
        }
      };
    }

    static List<DebugBattleAction> AddCardsToVoid(DisplayPlayer player, int count)
    {
      var list = new List<DebugBattleAction>();
      for (var i = 0; i < count; i++)
      {
        list.Add(AddCardToVoid(player));
      }
      return list;
    }

    static DebugBattleAction AddCardToVoid(DisplayPlayer player)
    {
      var name = player switch
      {
        DisplayPlayer.User => PlayerName.One,
        DisplayPlayer.Enemy => PlayerName.Two,
        _ => throw new IndexOutOfRangeException("Invalid player")
      };

      return new DebugBattleAction
      {
        AddCardToVoid = new AddCardToVoid
        {
          Card = CardName.MinstrelOfFallingLight,
          Player = name,
        }
      };
    }
  }
}
