#if UNITY_EDITOR

#nullable enable

using System;
using System.IO;
using Dreamtides.Services;
using UnityEditor;
using UnityEditor.SceneManagement;
using UnityEngine;
using UnityEngine.SceneManagement;

namespace Dreamtides.Editors
{
  /// <summary>
  /// Writes a JSON state file on play mode transitions and scene changes
  /// so that external tools (abu status, abu restart) can inspect Unity
  /// Editor state.
  /// </summary>
  [InitializeOnLoad]
  internal static class AbuStateWriter
  {
    private static string _lastScene = "";

    static AbuStateWriter()
    {
      EditorApplication.playModeStateChanged += OnPlayModeStateChanged;
      EditorSceneManager.activeSceneChangedInEditMode += OnSceneChanged;
      EditorApplication.update += OnUpdate;
    }

    private static void OnPlayModeStateChanged(PlayModeStateChange state)
    {
      WriteState(state.ToString());
    }

    private static void OnSceneChanged(Scene previous, Scene current)
    {
      WriteState(null);
    }

    private static void OnUpdate()
    {
      var currentScene = SceneManager.GetActiveScene().path;
      if (currentScene != _lastScene)
      {
        _lastScene = currentScene;
        WriteState(null);
      }
    }

    private static void WriteState(string? playModeOverride)
    {
      try
      {
        var data = new AbuState
        {
          version = 1,
          playModeState =
            playModeOverride
            ?? (EditorApplication.isPlaying ? "EnteredPlayMode" : "EnteredEditMode"),
          gameMode = PlayModeSelection.Current.ToString(),
          unityPid = System.Diagnostics.Process.GetCurrentProcess().Id,
          timestampUtc = DateTime.UtcNow.ToString("yyyy-MM-ddTHH:mm:ssZ"),
          activeScene = SceneManager.GetActiveScene().path,
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
      public string activeScene;
    }
  }
}
#endif
