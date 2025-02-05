#nullable enable

using UnityEngine;

namespace Dreamcaller.Utils
{
  public static class LogUtils
  {
    public static void Log(string message)
    {
      var consoleMessage = $"[UNITY]  {message}";
#if !UNITY_EDITOR && (UNITY_IOS || UNITY_WEBGL)
      Console.WriteLine(consoleMessage);
#endif
      Debug.Log(message);
    }

    public static void LogError(string message)
    {
      var consoleMessage = $"[ERROR]  {message}";
#if !UNITY_EDITOR && (UNITY_IOS || UNITY_WEBGL)
      Console.Error.WriteLine(consoleMessage);
#endif
      Debug.LogError(message);
    }
  }
}