#nullable enable

using System.IO;
using Dreamtides.Services;
using UnityEditor;
using UnityEditor.SceneManagement;
using UnityEngine;
using UnityEngine.SceneManagement;

namespace Dreamtides.Editors
{
  public static class LoadInMainSceneCommand
  {
    const string MainScenePath = "Assets/Scenes/Main.unity";
    const string PendingSceneKey = "LoadInMainSceneCommand.PendingScene";
    const string PendingLoadKey = "LoadInMainSceneCommand.PendingLoad";
    const string LoadAtTimeKey = "LoadInMainSceneCommand.LoadAtTime";
    const string RestoreSceneKey = "LoadInMainSceneCommand.RestoreScene";
    const string PendingRestoreKey = "LoadInMainSceneCommand.PendingRestore";

    [MenuItem("Tools/Load in Main Scene")]
    public static void LoadInMainScene()
    {
      if (EditorApplication.isPlayingOrWillChangePlaymode)
      {
        EditorUtility.DisplayDialog(
          "Load in Main Scene",
          "Exit Play Mode before using this command.",
          "OK"
        );
        return;
      }

      var activeScene = SceneManager.GetActiveScene();
      var currentScenePath = activeScene.path;
      if (string.IsNullOrEmpty(currentScenePath))
      {
        EditorUtility.DisplayDialog(
          "Load in Main Scene",
          "Current scene must be saved before loading.",
          "OK"
        );
        return;
      }

      if (!File.Exists(MainScenePath))
      {
        EditorUtility.DisplayDialog(
          "Load in Main Scene",
          $"Main scene not found at {MainScenePath}.",
          "OK"
        );
        return;
      }

      if (!EditorSceneManager.SaveCurrentModifiedScenesIfUserWantsTo())
      {
        return;
      }

      ScheduleRestore(currentScenePath);
      EditorSceneManager.OpenScene(MainScenePath, OpenSceneMode.Single);
      SchedulePendingLoad(currentScenePath);
      EditorApplication.isPlaying = true;
    }

    [MenuItem("Tools/Load in Main Scene", true)]
    public static bool ValidateLoadInMainScene()
    {
      return !EditorApplication.isPlayingOrWillChangePlaymode;
    }

    static LoadInMainSceneCommand()
    {
      EditorApplication.playModeStateChanged += OnPlayModeStateChanged;
      EditorApplication.update += OnEditorUpdate;
    }

    static void OnPlayModeStateChanged(PlayModeStateChange change)
    {
      if (change == PlayModeStateChange.EnteredPlayMode && HasPendingLoad())
      {
        SetLoadAtTime(EditorApplication.timeSinceStartup + 0.3f);
      }

      if (change == PlayModeStateChange.ExitingPlayMode)
      {
        ClearPendingLoad();
      }
    }

    static void OnEditorUpdate()
    {
      if (!HasPendingLoad())
      {
        TryRestoreOriginalScene();
        return;
      }

      if (!EditorApplication.isPlaying)
      {
        ClearPendingLoad();
        return;
      }

      var loadAtTime = GetLoadAtTime();
      if (loadAtTime == 0 || EditorApplication.timeSinceStartup < loadAtTime)
      {
        return;
      }

      var scenePath = GetPendingScenePath();
      ClearPendingLoad();
      if (string.IsNullOrEmpty(scenePath))
      {
        return;
      }

      LoadAdditiveScene(scenePath!);
    }

    static void SchedulePendingLoad(string scenePath)
    {
      SessionState.SetBool(PendingLoadKey, true);
      SessionState.SetString(PendingSceneKey, scenePath);
      SessionState.SetFloat(LoadAtTimeKey, 0f);
    }

    static string? GetPendingScenePath()
    {
      var path = SessionState.GetString(PendingSceneKey, string.Empty);
      return string.IsNullOrEmpty(path) ? null : path;
    }

    static bool HasPendingLoad()
    {
      return SessionState.GetBool(PendingLoadKey, defaultValue: false);
    }

    static void ClearPendingLoad()
    {
      SessionState.SetBool(PendingLoadKey, false);
      SessionState.SetString(PendingSceneKey, string.Empty);
      SessionState.SetFloat(LoadAtTimeKey, 0f);
    }

    static void SetLoadAtTime(double time)
    {
      SessionState.SetFloat(LoadAtTimeKey, (float)time);
    }

    static double GetLoadAtTime()
    {
      return SessionState.GetFloat(LoadAtTimeKey, 0f);
    }

    static void LoadAdditiveScene(string scenePath)
    {
      var operation = SceneManager.LoadSceneAsync(scenePath, LoadSceneMode.Additive);
      if (operation == null)
      {
        return;
      }

      operation.completed += _ => RemoveEditingHelpers(scenePath);
    }

    static void RemoveEditingHelpers(string scenePath)
    {
      var scene = SceneManager.GetSceneByPath(scenePath);
      if (!scene.IsValid())
      {
        return;
      }

      var roots = scene.GetRootGameObjects();
      foreach (var root in roots)
      {
        var cameras = root.GetComponentsInChildren<Camera>(true);
        foreach (var camera in cameras)
        {
          if (camera.CompareTag("MainCamera"))
          {
            camera.gameObject.SetActive(false);
          }
        }

        var lights = root.GetComponentsInChildren<Light>(true);
        foreach (var light in lights)
        {
          if (light.type == LightType.Directional)
          {
            light.gameObject.SetActive(false);
          }
        }
      }

      var registry = Object.FindFirstObjectByType<Registry>();
      if (registry != null)
      {
        registry.InitializeDisplayablesInScene(scene);
      }
    }

    static void ScheduleRestore(string scenePath)
    {
      SessionState.SetString(RestoreSceneKey, scenePath);
      SessionState.SetBool(PendingRestoreKey, true);
    }

    static void TryRestoreOriginalScene()
    {
      if (!HasPendingRestore() || EditorApplication.isPlaying)
      {
        return;
      }

      var scenePath = GetRestoreScenePath();
      ClearRestore();
      if (string.IsNullOrEmpty(scenePath) || !File.Exists(scenePath))
      {
        return;
      }

      EditorSceneManager.OpenScene(scenePath, OpenSceneMode.Single);
    }

    static bool HasPendingRestore()
    {
      return SessionState.GetBool(PendingRestoreKey, defaultValue: false);
    }

    static string? GetRestoreScenePath()
    {
      var path = SessionState.GetString(RestoreSceneKey, string.Empty);
      return string.IsNullOrEmpty(path) ? null : path;
    }

    static void ClearRestore()
    {
      SessionState.SetBool(PendingRestoreKey, false);
      SessionState.SetString(RestoreSceneKey, string.Empty);
    }
  }
}
