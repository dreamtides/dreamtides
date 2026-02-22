#if UNITY_EDITOR

#nullable enable

using System;
using System.IO;
using Dreamtides.Services;
using UnityEditor;
using UnityEngine;

namespace Dreamtides.Editors
{
  /// <summary>
  /// Writes a JSON state file on every play mode transition so that
  /// external tools (abu status) can inspect Unity Editor state.
  /// </summary>
  [InitializeOnLoad]
  internal static class AbuStateWriter
  {
    static AbuStateWriter()
    {
      EditorApplication.playModeStateChanged += OnPlayModeStateChanged;
    }

    private static void OnPlayModeStateChanged(PlayModeStateChange state)
    {
      try
      {
        var data = new AbuState
        {
          version = 1,
          playModeState = state.ToString(),
          gameMode = PlayModeSelection.Current.ToString(),
          unityPid = System.Diagnostics.Process.GetCurrentProcess().Id,
          timestampUtc = DateTime.UtcNow.ToString("yyyy-MM-ddTHH:mm:ssZ"),
        };
        var json = JsonUtility.ToJson(data, prettyPrint: true);
        var path = Path.GetFullPath(
          Path.Combine(Application.dataPath, "..", "..", ".abu-state.json")
        );
        File.WriteAllText(path, json);
      }
      catch (Exception e)
      {
        Debug.LogWarning($"[AbuStateWriter] Failed to write state file: {e.Message}");
      }
    }

    [Serializable]
    private struct AbuState
    {
      public int version;
      public string playModeState;
      public string gameMode;
      public int unityPid;
      public string timestampUtc;
    }
  }
}
#endif
