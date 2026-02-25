#nullable enable

using Abu;
using Dreamtides.Services;
using UnityEngine;

namespace Dreamtides.Abu
{
  /// <summary>
  /// Wires the Dreamtides scene walker and settled provider to the ABU bridge.
  /// Attach this to a GameObject in the Dreamtides scene to enable ABU
  /// integration. Does not extend the Dreamtides Service base class.
  ///
  /// When active, Unity runs a TCP server on port 9999 (configurable via the
  /// ABU_PORT environment variable) that accepts NDJSON commands from the
  /// Python CLI.
  ///
  /// For v0.1, this component is not added to production scenes. To test
  /// the full ABU pipeline manually:
  ///
  /// 1. Open the Unity editor with a scene containing this DreamtidesAbuSetup
  ///    component (add it to the Registry prefab or a standalone GameObject).
  /// 2. Enter Play mode.
  /// 3. Run: python3 scripts/abu/abu.py snapshot to get a scene snapshot.
  /// </summary>
  public class DreamtidesAbuSetup : MonoBehaviour
  {
    [SerializeField]
    Registry? _registry;

    HistoryRecorder? _historyRecorder;
    EffectLogRecorder? _effectLogRecorder;

    void Start()
    {
      if (_registry == null)
      {
        _registry = FindFirstObjectByType<Registry>();
      }

      if (_registry == null)
      {
        Debug.LogError("[DreamtidesAbuSetup] Registry not found. ABU integration disabled.");
        return;
      }

      var bridge = FindFirstObjectByType<AbuBridge>();
      if (bridge == null)
      {
        // AbuBridge.Awake() calls DontDestroyOnLoad on itself, so no need
        // to call it here after AddComponent.
        var bridgeObject = new GameObject("AbuBridge");
        bridge = bridgeObject.AddComponent<AbuBridge>();
      }

      var walker = new DreamtidesSceneWalker(_registry);
      bridge.RegisterWalker(walker);

      var settledProvider = new DreamtidesSettledProvider(_registry.ActionService);
      bridge.SetSettledProvider(settledProvider);

      _historyRecorder = new HistoryRecorder(_registry.CardService);
      _registry.ActionService.OnCommandProcessed += _historyRecorder.OnCommand;
      bridge.SetHistoryProvider(_historyRecorder);

      _effectLogRecorder = new EffectLogRecorder(_registry.CardService);
      _registry.ActionService.OnCommandProcessed += _effectLogRecorder.OnCommand;
      bridge.SetEffectLogProvider(_effectLogRecorder);
    }

    void OnDestroy()
    {
      if (_registry != null)
      {
        if (_historyRecorder != null)
        {
          _registry.ActionService.OnCommandProcessed -= _historyRecorder.OnCommand;
        }

        if (_effectLogRecorder != null)
        {
          _registry.ActionService.OnCommandProcessed -= _effectLogRecorder.OnCommand;
        }
      }
    }
  }
}
