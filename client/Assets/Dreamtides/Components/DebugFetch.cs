#nullable enable

using System;
using System.Collections;
using Dreamtides.Schema;
using Dreamtides.Services;
using UnityEngine;

namespace Dreamtides.Components
{
  public class DebugFetch : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;

    public void OnClick()
    {
      _registry.ActionService.PerformAction(new GameAction
      {
        DebugAction = DebugAction.DrawCard
      });
    }

    public void PerformSomeAction()
    {
      _registry.ActionService.PerformAction(new GameAction
      {
        DebugAction = DebugAction.PerformSomeAction
      });
    }

    public void TriggerJudgment()
    {
      _registry.ActionService.PerformAction(new GameAction
      {
        DebugAction = DebugAction.TriggerUserJudgment
      });
    }

    public void TriggerEnemyJudgment()
    {
      _registry.ActionService.PerformAction(new GameAction
      {
        DebugAction = DebugAction.TriggerEnemyJudgment
      });
    }
  }
}
