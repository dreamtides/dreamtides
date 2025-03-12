#nullable enable

using Dreamcaller.Schema;
using Dreamcaller.Services;
using UnityEngine;

public class CloseBrowserButton : MonoBehaviour
{
  [SerializeField] Registry _registry = null!;

  public void OnClick()
  {
    _registry.SoundService.PlayClickSound();
    var action = new UserAction
    {
      BattleAction = new()
      {
        Enum = BattleActionEnum.CloseCardBrowser
      }
    };
    _registry.ActionService.PerformAction(action);
  }
}
