#nullable enable

using System.Runtime.CompilerServices;
using Dreamtides.Layout;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.Utils;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Components
{
  public class CardBrowserButton : Displayable
  {
    [SerializeField]
    internal CardBrowserType _type;

    [SerializeField]
    internal string? _debugActionOverride;

    public override bool CanHandleMouseEvents() => true;

    public override void MouseUp(bool isSameObject)
    {
      Debug.Log("CardBrowserButton.MouseUp");
      if (!isSameObject || IsEmpty())
      {
        return;
      }

      Registry.SoundService.PlayClickSound();
      GameAction action;
      if (!string.IsNullOrEmpty(_debugActionOverride))
      {
        action = new GameAction
        {
          GameActionClass = new()
          {
            DebugAction = new()
            {
              DebugActionClass = new() { ApplyTestScenarioAction = _debugActionOverride },
            },
          },
        };
      }
      else
      {
        action = new GameAction
        {
          GameActionClass = new()
          {
            BattleDisplayAction = new BattleDisplayActionClass { BrowseCards = _type },
          },
        };
      }
      Debug.Log("CardBrowserButton.MouseUp: Performing action: " + action);
      Registry.ActionService.PerformAction(action);
    }

    bool IsEmpty()
    {
      switch (_type)
      {
        case CardBrowserType.UserDeck:
          return Registry.BattleLayout.UserDeck.Objects.Count == 0;
        case CardBrowserType.EnemyDeck:
          return Registry.BattleLayout.EnemyDeck.Objects.Count == 0;
        case CardBrowserType.UserVoid:
          return Registry.BattleLayout.UserVoid.Objects.Count == 0;
        case CardBrowserType.EnemyVoid:
          return Registry.BattleLayout.EnemyVoid.Objects.Count == 0;
        case CardBrowserType.UserStatus:
          return false;
        case CardBrowserType.EnemyStatus:
          return false;
        case CardBrowserType.QuestDeck:
          return false;
        default:
          throw Errors.UnknownEnumValue(_type);
      }
    }
  }
}
