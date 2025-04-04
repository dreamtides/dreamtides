#nullable enable

using System;
using System.Collections;
using Dreamcaller.Schema;
using Dreamcaller.Services;
using UnityEngine;

namespace Dreamcaller.Components
{
  public class DebugFetch : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;

    public void OnClick()
    {
      _registry.ActionService.PerformAction(new UserAction
      {
        DebugAction = DebugAction.DrawCard
      });
    }

    public void PerformSomeAction()
    {
      _registry.ActionService.PerformAction(new UserAction
      {
        DebugAction = DebugAction.PerformSomeAction
      });
    }

    public void TriggerJudgment()
    {
      _registry.ActionService.PerformAction(new UserAction
      {
        DebugAction = DebugAction.TriggerUserJudgment
      });
    }

    public void TriggerEnemyJudgment()
    {
      _registry.ActionService.PerformAction(new UserAction
      {
        DebugAction = DebugAction.TriggerEnemyJudgment
      });
    }
  }
}
