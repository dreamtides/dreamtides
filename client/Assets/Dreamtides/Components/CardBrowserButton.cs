#nullable enable

using Dreamtides.Layout;
using Dreamtides.Schema;
using Dreamtides.Services;
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
      if (!isSameObject)
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
  }
}
