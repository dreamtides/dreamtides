#nullable enable

using Dreamcaller.Layout;
using Dreamcaller.Schema;
using Dreamcaller.Services;
using UnityEngine;

namespace Dreamcaller.Components
{
  public class CardBrowserButton : Displayable
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] CardBrowserType _type;

    public override bool CanHandleMouseEvents() => true;

    public override void MouseUp()
    {
      Debug.Log($"CardBrowserButton {_type} clicked");
      var action = new UserAction
      {
        BattleAction = new BattleAction
        {
          BrowseCards = _type
        }
      };
      _registry.ActionService.PerformAction(action);
    }
  }
}