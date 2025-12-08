#nullable enable

using System.Collections.Generic;
using Dreamtides.Schema;
using Dreamtides.Services;

namespace Dreamtides.TestFakes
{
  public class FakeLoggingService : LoggingService
  {
    public override void StartSpan(LogSpanName name) { }

    public override void Log(string message) { }

    public override void Log(string className, string message) { }

    public override void Log(string message, Dictionary<string, string> arguments) { }

    public override void Log(
      string className,
      string message,
      Dictionary<string, string> arguments
    ) { }

    public override void Log(string message, params (string key, string value)[] keyValuePairs) { }

    public override void Log(
      string className,
      string message,
      params (string key, string value)[] keyValuePairs
    ) { }

    public override void LogInfo(string message) { }

    public override void LogInfo(string className, string message) { }

    public override void LogInfo(string message, Dictionary<string, string> arguments) { }

    public override void LogInfo(
      string className,
      string message,
      Dictionary<string, string> arguments
    ) { }

    public override void LogInfo(
      string message,
      params (string key, string value)[] keyValuePairs
    ) { }

    public override void LogInfo(
      string className,
      string message,
      params (string key, string value)[] keyValuePairs
    ) { }

    public override void LogError(string message) { }

    public override void LogError(string className, string message) { }

    public override void LogError(string message, Dictionary<string, string> arguments) { }

    public override void LogError(
      string className,
      string message,
      Dictionary<string, string> arguments
    ) { }

    public override void LogError(
      string message,
      params (string key, string value)[] keyValuePairs
    ) { }

    public override void LogError(
      string className,
      string message,
      params (string key, string value)[] keyValuePairs
    ) { }

    public override void LogWarning(string message) { }

    public override void LogWarning(string className, string message) { }

    public override void LogWarning(string message, Dictionary<string, string> arguments) { }

    public override void LogWarning(
      string className,
      string message,
      Dictionary<string, string> arguments
    ) { }

    public override void LogWarning(
      string message,
      params (string key, string value)[] keyValuePairs
    ) { }

    public override void LogWarning(
      string className,
      string message,
      params (string key, string value)[] keyValuePairs
    ) { }

    public override void EndSpan(LogSpanName name) { }
  }
}
