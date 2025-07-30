using Dreamtides.Schema;
using System.Collections.Generic;
using System;

namespace Dreamtides.TestUtils
{
  /// <summary>
  /// A builder with helper methods for adding cards to a test battle.
  /// </summary>
  public class TestBattle
  {
    readonly List<DebugBattleAction> _actions = new List<DebugBattleAction>();

    public static TestBattle New() => new();

    public GameAction Build()
    {
      return new GameAction
      {
        GameActionClass = new GameActionClass
        {
          DebugAction = new DebugAction
          {
            DebugActionClass = new DebugActionClass
            {
              ApplyActionList = _actions
            }
          }
        }
      };
    }

    /// <summary>
    /// Returns a DebugAction::ApplyActionList<> action that sets up a full
    /// layout.
    /// </summary>
    ///
    /// <remarks>
    /// This includes:
    /// - 5 cards in each player's hand.
    /// - 8 cards on each player's battlefield.
    /// - 5 cards in each player's void.
    /// </remarks>
    public TestBattle FullLayout()
    {
      RemovePlayerHands();
      AddCardsToHand(DisplayPlayer.User, 5);
      AddCardsToHand(DisplayPlayer.Enemy, 5);
      AddCardsToBattlefield(DisplayPlayer.User, 8);
      AddCardsToBattlefield(DisplayPlayer.Enemy, 8);
      AddCardsToVoid(DisplayPlayer.User, 5);
      AddCardsToVoid(DisplayPlayer.Enemy, 5);
      return this;
    }

    public TestBattle RemovePlayerHands()
    {
      _actions.Add(new DebugBattleAction
      {
        DebugBattleActionClass = new DebugBattleActionClass
        {
          MoveHandToDeck = new MoveHandToDeck { Player = PlayerName.One }
        }
      });
      _actions.Add(new DebugBattleAction
      {
        DebugBattleActionClass = new DebugBattleActionClass
        {
          MoveHandToDeck = new MoveHandToDeck { Player = PlayerName.Two }
        }
      });
      return this;
    }

    public TestBattle SetEnergy(DisplayPlayer player, int energy)
    {
      var name = GetPlayerName(player);
      _actions.Add(new DebugBattleAction
      {
        DebugBattleActionClass = new DebugBattleActionClass
        {
          SetEnergy = new SetEnergy { Player = name, Energy = energy }
        }
      });
      return this;
    }

    public TestBattle SetPoints(DisplayPlayer player, int points)
    {
      var name = GetPlayerName(player);
      _actions.Add(new DebugBattleAction
      {
        DebugBattleActionClass = new DebugBattleActionClass
        {
          SetPoints = new SetPoints { Player = name, Points = points }
        }
      });
      return this;
    }

    public TestBattle AddCardsToHand(DisplayPlayer player, int count)
    {
      for (var i = 0; i < count; i++)
      {
        AddCardToHand(player);
      }
      return this;
    }

    public TestBattle AddCardToHand(DisplayPlayer player, CardName cardName = CardName.TestVanillaCharacter)
    {
      var name = GetPlayerName(player);
      _actions.Add(new DebugBattleAction
      {
        DebugBattleActionClass = new DebugBattleActionClass
        {
          AddCardToHand = new AddCardToHand
          {
            Card = cardName,
            Player = name,
          }
        }
      });
      return this;
    }

    public TestBattle AddCardsToBattlefield(
      DisplayPlayer player,
      int count,
      CardName cardName = CardName.TestVanillaCharacter)
    {
      for (var i = 0; i < count; i++)
      {
        AddCardToBattlefield(player, cardName);
      }
      return this;
    }

    public TestBattle AddCardToBattlefield(DisplayPlayer player, CardName cardName = CardName.TestVanillaCharacter)
    {
      var name = GetPlayerName(player);
      _actions.Add(new DebugBattleAction
      {
        DebugBattleActionClass = new DebugBattleActionClass
        {
          AddCardToBattlefield = new AddCardToBattlefield
          {
            Card = cardName,
            Player = name,
          }
        }
      });
      return this;
    }

    public TestBattle AddCardsToVoid(DisplayPlayer player, int count)
    {
      for (var i = 0; i < count; i++)
      {
        AddCardToVoid(player);
      }
      return this;
    }

    public TestBattle AddCardToVoid(DisplayPlayer player)
    {
      var name = GetPlayerName(player);
      _actions.Add(new DebugBattleAction
      {
        DebugBattleActionClass = new DebugBattleActionClass
        {
          AddCardToVoid = new AddCardToVoid
          {
            Card = CardName.TestVanillaCharacter,
            Player = name,
          }
        }
      });
      return this;
    }

    static PlayerName GetPlayerName(DisplayPlayer player)
    {
      return player switch
      {
        DisplayPlayer.User => PlayerName.One,
        DisplayPlayer.Enemy => PlayerName.Two,
        _ => throw new IndexOutOfRangeException("Invalid player")
      };
    }
  }
}
