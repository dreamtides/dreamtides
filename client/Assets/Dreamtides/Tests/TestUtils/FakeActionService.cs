#nullable enable

using System;
using System.Collections.Generic;
using Dreamtides.Schema;
using Dreamtides.Services;

namespace Dreamtides.Tests.TestUtils
{
  public class FakeActionService : ActionService
  {
    readonly List<(GameAction action, Guid? requestIdentifier)> _performedActions = new();

    public IReadOnlyList<(GameAction action, Guid? requestIdentifier)> PerformedActions =>
      _performedActions;

    public override bool Connected { get; protected set; } = true;
    public override float LastActionTime { get; } = 0f;
    public override bool IsProcessingCommands { get; } = false;
    public override bool LastResponseIncremental { get; protected set; } = false;
    public override Guid? LastResponseReceived { get; protected set; } = null;

    public void ClearPerformedActions()
    {
      _performedActions.Clear();
    }

    public override void PerformAction(GameAction? action, Guid? requestIdentifier = null)
    {
      if (action == null)
      {
        return;
      }

      _performedActions.Add((action.Value, requestIdentifier));
    }

    public override void Log(ClientLogRequest request) { }

    public override void TriggerReconnect() { }
  }
}
