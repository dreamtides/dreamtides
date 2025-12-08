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

    public override bool CanHandleMouseEvents() => true;

    public override void MouseUp(bool isSameObject)
    {
      if (!isSameObject || IsEmpty())
      {
        return;
      }

      Registry.SoundService.PlayClickSound();
      var action = new GameAction
      {
        GameActionClass = new()
        {
          DebugAction = new()
          {
            DebugActionClass = new() { ApplyTestScenarioAction = "browseQuestDeck" },
          },
        },
      };
      Registry.ActionService.PerformAction(action);
    }

    bool IsEmpty()
    {
      switch (_type)
      {
        case CardBrowserType.UserDeck:
          return Registry.Layout.UserDeck.Objects.Count == 0;
        case CardBrowserType.EnemyDeck:
          return Registry.Layout.EnemyDeck.Objects.Count == 0;
        case CardBrowserType.UserVoid:
          return Registry.Layout.UserVoid.Objects.Count == 0;
        case CardBrowserType.EnemyVoid:
          return Registry.Layout.EnemyVoid.Objects.Count == 0;
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
