#nullable enable

using UnityEditor;
using UnityEngine;

namespace Dreamtides.UnityInternal
{
  /// <summary>
  /// By creating an assembly named "Unity.InternalAPIEditorBridge.020", we have
  /// access to Unity's internal APIs.
  ///
  /// See https://stackoverflow.com/questions/79563229
  /// </summary>
  public static class GameViewUtils
  {
    public static void SetGameViewResolution(Vector2 resolution)
    {
      var gameView = EditorWindow.GetWindow<GameView>();
      gameView.SetCustomResolution(resolution, "TestResolution");
    }
  }
}
