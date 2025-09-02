#nullable enable

using System.Collections.Generic;
using System.Linq;
using Dreamtides.Schema;
using UnityEngine;

namespace Dreamtides.Services
{
    public class LoggingService : Service
    {
        private class ActiveSpan
        {
            public LogSpanName Name { get; set; }
            public List<LogEntry> Entries { get; set; } = new();
            public float LastLogTime { get; set; }
        }

        private readonly Stack<ActiveSpan> _spanStack = new();
        private readonly List<LogEntry> _bufferedLogs = new();
        private float? _lastBufferedLogTime;
        private const float SPAN_TIMEOUT_SECONDS = 10f;
    // Reentrancy guard so our own Debug.Log calls don't get re-captured and re-logged infinitely
    private bool _emittingToUnityConsole;

        protected override void OnInitialize(TestConfiguration? testConfiguration)
        {
            Application.logMessageReceived += OnUnityLogMessageReceived;
        }

        protected override void OnUpdate()
        {
            var currentTime = Time.time;

            if (_spanStack.Count > 0)
            {
                var oldestSpan = _spanStack.First();
                if (currentTime - oldestSpan.LastLogTime > SPAN_TIMEOUT_SECONDS)
                {
                    LogWarning($"Log span stack timed out, force closing all spans");
                    while (_spanStack.Count > 0)
                    {
                        var span = _spanStack.Pop();
                        LogWarning($"Force closing span '{span.Name}'");
                    }
                    FlushSpanStack();
                }
            }

            if (_lastBufferedLogTime.HasValue &&
                currentTime - _lastBufferedLogTime.Value > SPAN_TIMEOUT_SECONDS &&
                _bufferedLogs.Count > 0)
            {
                FlushBufferedLogs();
            }
        }

        public void StartSpan(LogSpanName name)
        {
            var span = new ActiveSpan
            {
                Name = name,
                LastLogTime = Time.time
            };
            _spanStack.Push(span);
        }

        public void Log(string message)
        {
            AddLogEntry(Schema.LogType.Debug, message, new Dictionary<string, string>());
        }

        public void Log(string className, string message)
        {
            AddLogEntry(Schema.LogType.Debug, message, new Dictionary<string, string>(), className);
        }

        public void Log(string message, Dictionary<string, string> arguments)
        {
            AddLogEntry(Schema.LogType.Debug, message, arguments);
        }

        public void Log(string className, string message, Dictionary<string, string> arguments)
        {
            AddLogEntry(Schema.LogType.Debug, message, arguments, className);
        }

        public void Log(string message, params (string key, string value)[] keyValuePairs)
        {
            var arguments = keyValuePairs.ToDictionary(kvp => kvp.key, kvp => kvp.value);
            AddLogEntry(Schema.LogType.Debug, message, arguments);
        }

        public void Log(string className, string message, params (string key, string value)[] keyValuePairs)
        {
            var arguments = keyValuePairs.ToDictionary(kvp => kvp.key, kvp => kvp.value);
            AddLogEntry(Schema.LogType.Debug, message, arguments, className);
        }

        public void LogInfo(string message)
        {
            AddLogEntry(Schema.LogType.Info, message, new Dictionary<string, string>());
        }

        public void LogInfo(string className, string message)
        {
            AddLogEntry(Schema.LogType.Info, message, new Dictionary<string, string>(), className);
        }

        public void LogInfo(string message, Dictionary<string, string> arguments)
        {
            AddLogEntry(Schema.LogType.Info, message, arguments);
        }

        public void LogInfo(string className, string message, Dictionary<string, string> arguments)
        {
            AddLogEntry(Schema.LogType.Info, message, arguments, className);
        }

        public void LogInfo(string message, params (string key, string value)[] keyValuePairs)
        {
            var arguments = keyValuePairs.ToDictionary(kvp => kvp.key, kvp => kvp.value);
            AddLogEntry(Schema.LogType.Info, message, arguments);
        }

        public void LogInfo(string className, string message, params (string key, string value)[] keyValuePairs)
        {
            var arguments = keyValuePairs.ToDictionary(kvp => kvp.key, kvp => kvp.value);
            AddLogEntry(Schema.LogType.Info, message, arguments, className);
        }

        public void LogError(string message)
        {
            AddLogEntry(Schema.LogType.Error, message, new Dictionary<string, string>());
        }

        public void LogError(string className, string message)
        {
            AddLogEntry(Schema.LogType.Error, message, new Dictionary<string, string>(), className);
        }

        public void LogError(string message, Dictionary<string, string> arguments)
        {
            AddLogEntry(Schema.LogType.Error, message, arguments);
        }

        public void LogError(string className, string message, Dictionary<string, string> arguments)
        {
            AddLogEntry(Schema.LogType.Error, message, arguments, className);
        }

        public void LogError(string message, params (string key, string value)[] keyValuePairs)
        {
            var arguments = keyValuePairs.ToDictionary(kvp => kvp.key, kvp => kvp.value);
            AddLogEntry(Schema.LogType.Error, message, arguments);
        }

        public void LogError(string className, string message, params (string key, string value)[] keyValuePairs)
        {
            var arguments = keyValuePairs.ToDictionary(kvp => kvp.key, kvp => kvp.value);
            AddLogEntry(Schema.LogType.Error, message, arguments, className);
        }

        public void LogWarning(string message)
        {
            AddLogEntry(Schema.LogType.Warning, message, new Dictionary<string, string>());
        }

        public void LogWarning(string className, string message)
        {
            AddLogEntry(Schema.LogType.Warning, message, new Dictionary<string, string>(), className);
        }

        public void LogWarning(string message, Dictionary<string, string> arguments)
        {
            AddLogEntry(Schema.LogType.Warning, message, arguments);
        }

        public void LogWarning(string className, string message, Dictionary<string, string> arguments)
        {
            AddLogEntry(Schema.LogType.Warning, message, arguments, className);
        }

        public void LogWarning(string message, params (string key, string value)[] keyValuePairs)
        {
            var arguments = keyValuePairs.ToDictionary(kvp => kvp.key, kvp => kvp.value);
            AddLogEntry(Schema.LogType.Warning, message, arguments);
        }

        public void LogWarning(string className, string message, params (string key, string value)[] keyValuePairs)
        {
            var arguments = keyValuePairs.ToDictionary(kvp => kvp.key, kvp => kvp.value);
            AddLogEntry(Schema.LogType.Warning, message, arguments, className);
        }

        public void EndSpan(LogSpanName name)
        {
            if (_spanStack.Count == 0) return;

            var spansToClose = new List<ActiveSpan>();
            var found = false;

            while (_spanStack.Count > 0)
            {
                var span = _spanStack.Pop();
                spansToClose.Add(span);

                if (span.Name == name)
                {
                    found = true;
                    break;
                }
            }

            if (!found)
            {
                LogError($"EndSpan called with '{name}' but span was not found in stack");
                foreach (var span in spansToClose.AsEnumerable().Reverse())
                {
                    _spanStack.Push(span);
                }
                return;
            }

            var rootSpanEntry = BuildNestedSpanStructure(spansToClose);

            if (_spanStack.Count > 0)
            {
                var parentSpan = _spanStack.Peek();
                parentSpan.Entries.Add(rootSpanEntry);
                parentSpan.LastLogTime = Time.time;
            }
            else
            {
                var request = new ClientLogRequest
                {
                    Entry = rootSpanEntry
                };
                Registry.ActionService.Log(request);
            }
        }

        private LogEntry BuildNestedSpanStructure(List<ActiveSpan> spans)
        {
            if (spans.Count == 1)
            {
                return new LogEntry
                {
                    EventSpan = new EventSpan
                    {
                        Name = spans[0].Name,
                        Entries = spans[0].Entries
                    }
                };
            }

            var rootSpan = spans[spans.Count - 1];
            var childSpans = spans.Take(spans.Count - 1).ToList();

            var childSpanEntry = BuildNestedSpanStructure(childSpans);
            rootSpan.Entries.Add(childSpanEntry);

            return new LogEntry
            {
                EventSpan = new EventSpan
                {
                    Name = rootSpan.Name,
                    Entries = rootSpan.Entries
                }
            };
        }

        private void FlushSpanStack()
        {
            if (_spanStack.Count == 0) return;

            var rootSpan = _spanStack.First();
            _spanStack.Clear();

            var logEntry = new LogEntry
            {
                EventSpan = new EventSpan
                {
                    Name = rootSpan.Name,
                    Entries = rootSpan.Entries
                }
            };

            var request = new ClientLogRequest
            {
                Entry = logEntry
            };

            Registry.ActionService.Log(request);
        }

        private void FlushBufferedLogs()
        {
            if (_bufferedLogs.Count == 0) return;

            var logEntry = new LogEntry
            {
                EventSpan = new EventSpan
                {
                    Name = LogSpanName.Untagged,
                    Entries = new List<LogEntry>(_bufferedLogs)
                }
            };

            var request = new ClientLogRequest
            {
                Entry = logEntry
            };

            Registry.ActionService.Log(request);

            _bufferedLogs.Clear();
            _lastBufferedLogTime = null;
        }

        private void AddLogEntry(Schema.LogType logType, string message, Dictionary<string, string> arguments, string? source = null)
        {
            var formattedMessage = FormatMessageWithArguments(message, arguments, source);

            var entry = new LogEntry
            {
                Event = new Schema.Event
                {
                    LogType = logType,
                    Message = formattedMessage
                }
            };

            if (_spanStack.Count > 0)
            {
                var currentSpan = _spanStack.Peek();
                currentSpan.Entries.Add(entry);
                currentSpan.LastLogTime = Time.time;
            }
            else
            {
                _bufferedLogs.Add(entry);
                _lastBufferedLogTime = Time.time;
            }

            // Immediately emit to Unity console (guard against recursive capture)
            if (!_emittingToUnityConsole)
            {
                try
                {
                    _emittingToUnityConsole = true;
                    switch (logType)
                    {
                        case Schema.LogType.Error:
                            Debug.LogError(formattedMessage);
                            break;
                        case Schema.LogType.Warning:
                            Debug.LogWarning(formattedMessage);
                            break;
                        case Schema.LogType.Info:
                        case Schema.LogType.Debug:
                        default:
                            Debug.Log(formattedMessage);
                            break;
                    }
                }
                finally
                {
                    _emittingToUnityConsole = false;
                }
            }
        }

        private static string FormatMessageWithArguments(string message, Dictionary<string, string> arguments, string? source = null)
        {
            var baseMessage = source != null ? $"[{source}]: {message}" : message;

            if (arguments.Count == 0)
            {
                return baseMessage;
            }

            var formattedArguments = arguments.Select(kvp => $"{kvp.Key}: {kvp.Value}");
            return $"{baseMessage} | {string.Join(" | ", formattedArguments)}";
        }

        private void OnUnityLogMessageReceived(string condition, string _stackTrace, UnityEngine.LogType type)
        {
            // If we're currently emitting to the console ourselves, skip to avoid duplication
            if (_emittingToUnityConsole) return;
            var logType = ConvertUnityLogType(type);
            var arguments = new Dictionary<string, string>();
            AddLogEntry(logType, condition, arguments);
        }

        private static Schema.LogType ConvertUnityLogType(UnityEngine.LogType unityLogType)
        {
            return unityLogType switch
            {
                UnityEngine.LogType.Error => Schema.LogType.Error,
                UnityEngine.LogType.Assert => Schema.LogType.Error,
                UnityEngine.LogType.Warning => Schema.LogType.Warning,
                UnityEngine.LogType.Log => Schema.LogType.Info,
                UnityEngine.LogType.Exception => Schema.LogType.Error,
                _ => Schema.LogType.Debug
            };
        }

        private void OnDestroy()
        {
            Application.logMessageReceived -= OnUnityLogMessageReceived;
        }
    }
}