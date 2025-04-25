#nullable enable

using UnityEngine;
#if !UNITY_EDITOR && (UNITY_IOS || UNITY_WEBGL)
using System;
#endif

namespace Dreamtides.Utils
{
  public static class LogUtils
  {
    public static void Log(string tag, string message)
    {
      var consoleMessage = $"[UNITY] [{tag}] {message}";
#if !UNITY_EDITOR && (UNITY_IOS || UNITY_WEBGL)
      Console.WriteLine(consoleMessage);
#endif
      Debug.Log(consoleMessage);
    }

    public static void LogError(string tag, string message)
    {
      var consoleMessage = $"[ERROR] [{tag}] {message}";
#if !UNITY_EDITOR && (UNITY_IOS || UNITY_WEBGL)
      Console.Error.WriteLine(consoleMessage);
#endif
      Debug.LogError(consoleMessage);
    }
  }
}