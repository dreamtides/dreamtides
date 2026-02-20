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
  /// For v0.1, this component is not added to production scenes. To test
  /// the full ABU pipeline manually:
  ///
  /// 1. Set AGENT_BROWSER_HOME=~/abu/daemon/ in your shell environment.
  /// 2. Start the daemon: cd ~/abu/daemon and run node dist/daemon.js
  ///    (or let the agent-browser CLI start it automatically).
  /// 3. Open the Unity editor with a scene containing this DreamtidesAbuSetup
  ///    component (add it to the Registry prefab or a standalone GameObject).
  /// 4. Enter Play mode.
  /// 5. Run: agent-browser snapshot to get a scene snapshot.
  /// </summary>
  public class DreamtidesAbuSetup : MonoBehaviour
  {
    [SerializeField]
    Registry? _registry;

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
    }
  }
}
