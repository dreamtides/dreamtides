#if UNITY_EDITOR

#nullable enable

using System;
using System.IO;
using UnityEditor;
using UnityEditor.SceneManagement;
using UnityEngine;

namespace Dreamtides.Editors
{
  /// <summary>
  /// Checks for an abu restart marker file on domain reload and restores
  /// the requested scene, enabling seamless editor restart recovery.
  /// Defers the readiness signal until the editor is no longer compiling
  /// or importing assets.
  /// </summary>
  [InitializeOnLoad]
  internal static class AbuRestartHandler
  {
    static AbuRestartHandler()
    {
      var markerPath = Path.GetFullPath(
        Path.Combine(Application.dataPath, "..", ".abu-restart.json")
      );

      if (!File.Exists(markerPath))
      {
        return;
      }

      try
      {
        var json = File.ReadAllText(markerPath);
        File.Delete(markerPath);
        var marker = JsonUtility.FromJson<RestartMarker>(json);

        if (!string.IsNullOrEmpty(marker.scene))
        {
          EditorSceneManager.OpenScene(marker.scene);
          Debug.Log($"[AbuRestart] Scene restored: {marker.scene}");
        }
      }
      catch (Exception e)
      {
        Debug.LogWarning($"[AbuRestart] Failed to process restart marker: {e.Message}");
      }

      // Defer the readiness signal until the editor finishes compiling
      // and importing assets, so the scene is fully loaded.
      EditorApplication.update += WaitForReady;
    }

    private static void WaitForReady()
    {
      if (EditorApplication.isCompiling || EditorApplication.isUpdating)
      {
        return;
      }

      EditorApplication.update -= WaitForReady;
      Debug.Log("[AbuRestart] Ready");
    }

    [Serializable]
    private struct RestartMarker
    {
      public string scene;
    }
  }
}
#endif
