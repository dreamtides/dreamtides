#nullable enable

using Dreamtides.Layout;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.Utils;
using UnityEngine;

namespace Dreamtides.Components
{
  public class CardBrowserButton : Displayable
  {
    [SerializeField]
    Registry _registry = null!;

    [SerializeField]
    CardBrowserType _type;

    public override bool CanHandleMouseEvents() => true;

    public override void MouseUp(bool isSameObject)
    {
      if (!isSameObject || IsEmpty())
      {
        return;
      }

      _registry.SoundService.PlayClickSound();
      var action = new GameAction
      {
        GameActionClass = new()
        {
          BattleDisplayAction = new() { BattleDisplayActionClass = new() { BrowseCards = _type } },
        },
      };
      _registry.ActionService.PerformAction(action);
    }

    bool IsEmpty()
    {
      switch (_type)
      {
        case CardBrowserType.UserDeck:
          return _registry.Layout.UserDeck.Objects.Count == 0;
        case CardBrowserType.EnemyDeck:
          return _registry.Layout.EnemyDeck.Objects.Count == 0;
        case CardBrowserType.UserVoid:
          return _registry.Layout.UserVoid.Objects.Count == 0;
        case CardBrowserType.EnemyVoid:
          return _registry.Layout.EnemyVoid.Objects.Count == 0;
        case CardBrowserType.UserStatus:
          return false;
        case CardBrowserType.EnemyStatus:
          return false;
        default:
          throw Errors.UnknownEnumValue(_type);
      }
    }
  }
}
