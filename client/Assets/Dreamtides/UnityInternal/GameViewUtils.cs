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
    ResolutionIPhone12,
    ResolutionIPhoneSE,
    ResolutionIPadPro12,
    ResolutionIPodTouch6,
    ResolutionSamsungNote20,
    ResolutionSamsungZFold2,
    ResolutionPixel5,
  }

  public static class GameViewUtils
  {
    public static void SetGameViewResolution(GameViewResolution resolution)
    {
      // By creating an assembly named "Unity.InternalAPIEditorBridge.020", we have
      // access to Unity's internal APIs.
      //
      // See https://stackoverflow.com/questions/79563229
      var gameView = EditorWindow.GetWindow<GameView>();
      gameView.SetCustomResolution(GetResolution(resolution), "TestResolution");
    }

    public static Vector2 GetResolution(GameViewResolution resolution)
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
        case GameViewResolution.ResolutionIPhone12:
          return new Vector2(1170, 2532);
        case GameViewResolution.ResolutionIPhoneSE:
          return new Vector2(750, 1334);
        case GameViewResolution.ResolutionIPadPro12:
          return new Vector2(2048, 2732);
        case GameViewResolution.ResolutionIPodTouch6:
          return new Vector2(640, 1136);
        case GameViewResolution.ResolutionSamsungNote20:
          return new Vector2(1440, 3088);
        case GameViewResolution.ResolutionSamsungZFold2:
          return new Vector2(960, 2658);
        case GameViewResolution.ResolutionPixel5:
          return new Vector2(1080, 2340);
        default:
          throw new InvalidOperationException($"Invalid game view resolution: {resolution}");
      }
    }
  }
}
