#nullable enable

using UnityEngine;

namespace Dreamtides.Services
{
  public class IdleReconnectService : Service
  {
    const float IdleTimeoutSeconds = 300.0f;
    bool _hasInitialized = false;
    TestConfiguration? _testConfiguration;
    float _lastReconnectTime = float.NegativeInfinity;
    float _lastActivityTime;

    protected override void OnInitialize(TestConfiguration? testConfiguration)
    {
      _hasInitialized = true;
      _testConfiguration = testConfiguration;
      _lastActivityTime = Time.time;
    }

    protected override void OnUpdate()
    {
      if (!_hasInitialized || !Registry.ActionService.Connected || _testConfiguration != null)
      {
        return;
      }

      var thinking =
        !Registry.ActionService.IsProcessingCommands
        && Registry.ActionService.LastResponseIncremental;
      Registry.Layout.ThinkingIndicator.SetActive(thinking);

      var currentTime = Time.time;
      var hasActivity =
        Registry.ActionService.IsProcessingCommands
        || Registry.InputService.IsPointerPressed()
        || Registry.ActionService.LastActionTime > _lastActivityTime;

      if (hasActivity)
      {
        _lastActivityTime = currentTime;
        return;
      }

      var timeSinceLastActivity = currentTime - _lastActivityTime;
      var timeSinceLastReconnect = currentTime - _lastReconnectTime;

      if (
        timeSinceLastActivity >= IdleTimeoutSeconds
        && timeSinceLastReconnect >= IdleTimeoutSeconds
      )
      {
        _lastReconnectTime = currentTime;
        Registry.ActionService.TriggerReconnect();
      }
    }
  }
}
