#nullable enable

using System;
using UnityEditor;
using UnityEngine;

namespace Dreamtides.UnityInternal
{
  public enum GameViewResolution
  {
    Resolution16x9,
    Resolution16x10,
    Resolution21x9,
    Resolution4x3,
    Resolution5x4,
    Resolution32x9,
  }

  /// <summary>
  /// By creating an assembly named "Unity.InternalAPIEditorBridge.020", we have
  /// access to Unity's internal APIs.
  ///
  /// See https://stackoverflow.com/questions/79563229
  /// </summary>
  public static class GameViewUtils
  {
    public static void SetGameViewResolution(GameViewResolution resolution)
    {
      var gameView = EditorWindow.GetWindow<GameView>();
      gameView.SetCustomResolution(GetResolution(resolution), "TestResolution");
    }

    static Vector2 GetResolution(GameViewResolution resolution)
    {
      switch (resolution)
      {
        case GameViewResolution.Resolution16x9:
          return new Vector2(1920, 1080);
        case GameViewResolution.Resolution16x10:
          return new Vector2(2560, 1600);
        case GameViewResolution.Resolution21x9:
          return new Vector2(3440, 1440);
        case GameViewResolution.Resolution4x3:
          return new Vector2(1600, 1200);
        case GameViewResolution.Resolution5x4:
          return new Vector2(1280, 1024);
        case GameViewResolution.Resolution32x9:
          return new Vector2(5120, 1440);
        default:
          throw new InvalidOperationException($"Invalid game view resolution: {resolution}");
      }
    }
  }
}
