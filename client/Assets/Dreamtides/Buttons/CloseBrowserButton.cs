#nullable enable

using Dreamtides.Schema;
using Dreamtides.Services;
using UnityEngine;

public class CloseBrowserButton : MonoBehaviour
{
  [SerializeField]
  internal Registry _registry = null!;

  GameAction? _closeAction;
  public GameAction? CloseAction
  {
    set
    {
      gameObject.SetActive(value.HasValue);
      _closeAction = value;
    }
  }

  public void SetActiveIfHasAction(bool active)
  {
    if (_closeAction.HasValue)
    {
      gameObject.SetActive(active);
    }
    else
    {
      gameObject.SetActive(false);
    }
  }

  public void OnClick()
  {
    _registry.SoundService.PlayClickSound();
    if (_closeAction.HasValue)
    {
      _registry.ActionService.PerformAction(_closeAction.Value);
    }
    else
    {
      _registry.LoggingService.LogError("CloseBrowserButton: No close action set");
    }
  }
}
