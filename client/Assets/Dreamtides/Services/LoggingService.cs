#nullable enable

using System.Collections.Generic;
using System.Linq;
using Dreamtides.Schema;
using UnityEngine;

namespace Dreamtides.Services
{
    public class LoggingService : Service
    {
        private readonly Dictionary<string, List<LogEntry>> _activeSpans = new();
        private readonly Dictionary<string, float> _lastLogTimes = new();
        private readonly List<LogEntry> _bufferedLogs = new();
        private float? _lastBufferedLogTime;
        private const float SPAN_TIMEOUT_SECONDS = 10f;

        protected override void OnInitialize(TestConfiguration? testConfiguration)
        {
            Application.logMessageReceived += OnUnityLogMessageReceived;
        }

        private void Update()
        {
            var currentTime = Time.time;
            var spansToClose = new List<string>();

            foreach (var kvp in _lastLogTimes)
            {
                var spanName = kvp.Key;
                var lastLogTime = kvp.Value;

                if (currentTime - lastLogTime > SPAN_TIMEOUT_SECONDS)
                {
                    spansToClose.Add(spanName);
                }
            }

            foreach (var spanName in spansToClose)
            {
                LogWarning($"Log span '{spanName}' was not properly closed, auto-closing after timeout");
                EndSpan(spanName);
            }

            if (_lastBufferedLogTime.HasValue &&
                currentTime - _lastBufferedLogTime.Value > SPAN_TIMEOUT_SECONDS &&
                _bufferedLogs.Count > 0)
            {
                FlushBufferedLogs();
            }
        }

        public void StartSpan(string name)
        {
            if (!_activeSpans.ContainsKey(name))
            {
                _activeSpans[name] = new List<LogEntry>();
                _lastLogTimes[name] = Time.time;
            }
        }

        public void Log(string message)
        {
            AddLogEntry(Schema.LogType.Debug, message, new Dictionary<string, string>());
        }

        public void Log(string className, string message)
        {
            var arguments = new Dictionary<string, string> { ["source"] = className };
            AddLogEntry(Schema.LogType.Debug, message, arguments);
        }

        public void Log(string message, Dictionary<string, string> arguments)
        {
            AddLogEntry(Schema.LogType.Debug, message, arguments);
        }

        public void Log(string className, string message, Dictionary<string, string> arguments)
        {
            var combinedArguments = new Dictionary<string, string>(arguments) { ["source"] = className };
            AddLogEntry(Schema.LogType.Debug, message, combinedArguments);
        }

        public void Log(string message, params (string key, string value)[] keyValuePairs)
        {
            var arguments = keyValuePairs.ToDictionary(kvp => kvp.key, kvp => kvp.value);
            AddLogEntry(Schema.LogType.Debug, message, arguments);
        }

        public void Log(string className, string message, params (string key, string value)[] keyValuePairs)
        {
            var arguments = keyValuePairs.ToDictionary(kvp => kvp.key, kvp => kvp.value);
            arguments["source"] = className;
            AddLogEntry(Schema.LogType.Debug, message, arguments);
        }

        public void LogInfo(string message)
        {
            AddLogEntry(Schema.LogType.Info, message, new Dictionary<string, string>());
        }

        public void LogInfo(string className, string message)
        {
            var arguments = new Dictionary<string, string> { ["source"] = className };
            AddLogEntry(Schema.LogType.Info, message, arguments);
        }

        public void LogInfo(string message, Dictionary<string, string> arguments)
        {
            AddLogEntry(Schema.LogType.Info, message, arguments);
        }

        public void LogInfo(string className, string message, Dictionary<string, string> arguments)
        {
            var combinedArguments = new Dictionary<string, string>(arguments) { ["source"] = className };
            AddLogEntry(Schema.LogType.Info, message, combinedArguments);
        }

        public void LogInfo(string message, params (string key, string value)[] keyValuePairs)
        {
            var arguments = keyValuePairs.ToDictionary(kvp => kvp.key, kvp => kvp.value);
            AddLogEntry(Schema.LogType.Info, message, arguments);
        }

        public void LogInfo(string className, string message, params (string key, string value)[] keyValuePairs)
        {
            var arguments = keyValuePairs.ToDictionary(kvp => kvp.key, kvp => kvp.value);
            arguments["source"] = className;
            AddLogEntry(Schema.LogType.Info, message, arguments);
        }

        public void LogError(string message)
        {
            AddLogEntry(Schema.LogType.Error, message, new Dictionary<string, string>());
        }

        public void LogError(string className, string message)
        {
            var arguments = new Dictionary<string, string> { ["source"] = className };
            AddLogEntry(Schema.LogType.Error, message, arguments);
        }

        public void LogError(string message, Dictionary<string, string> arguments)
        {
            AddLogEntry(Schema.LogType.Error, message, arguments);
        }

        public void LogError(string className, string message, Dictionary<string, string> arguments)
        {
            var combinedArguments = new Dictionary<string, string>(arguments) { ["source"] = className };
            AddLogEntry(Schema.LogType.Error, message, combinedArguments);
        }

        public void LogError(string message, params (string key, string value)[] keyValuePairs)
        {
            var arguments = keyValuePairs.ToDictionary(kvp => kvp.key, kvp => kvp.value);
            AddLogEntry(Schema.LogType.Error, message, arguments);
        }

        public void LogError(string className, string message, params (string key, string value)[] keyValuePairs)
        {
            var arguments = keyValuePairs.ToDictionary(kvp => kvp.key, kvp => kvp.value);
            arguments["source"] = className;
            AddLogEntry(Schema.LogType.Error, message, arguments);
        }

        public void LogWarning(string message)
        {
            AddLogEntry(Schema.LogType.Warning, message, new Dictionary<string, string>());
        }

        public void LogWarning(string className, string message)
        {
            var arguments = new Dictionary<string, string> { ["source"] = className };
            AddLogEntry(Schema.LogType.Warning, message, arguments);
        }

        public void LogWarning(string message, Dictionary<string, string> arguments)
        {
            AddLogEntry(Schema.LogType.Warning, message, arguments);
        }

        public void LogWarning(string className, string message, Dictionary<string, string> arguments)
        {
            var combinedArguments = new Dictionary<string, string>(arguments) { ["source"] = className };
            AddLogEntry(Schema.LogType.Warning, message, combinedArguments);
        }

        public void LogWarning(string message, params (string key, string value)[] keyValuePairs)
        {
            var arguments = keyValuePairs.ToDictionary(kvp => kvp.key, kvp => kvp.value);
            AddLogEntry(Schema.LogType.Warning, message, arguments);
        }

        public void LogWarning(string className, string message, params (string key, string value)[] keyValuePairs)
        {
            var arguments = keyValuePairs.ToDictionary(kvp => kvp.key, kvp => kvp.value);
            arguments["source"] = className;
            AddLogEntry(Schema.LogType.Warning, message, arguments);
        }

        public void EndSpan(string name)
        {
            if (_activeSpans.TryGetValue(name, out var entries))
            {
                _activeSpans.Remove(name);
                _lastLogTimes.Remove(name);

                var logEntry = new LogEntry
                {
                    EventSpan = new EventSpan
                    {
                        Name = name,
                        Entries = entries
                    }
                };

                var request = new ClientLogRequest
                {
                    Entry = logEntry
                };

                Registry.ActionService.Log(request);
            }
        }

        private void FlushBufferedLogs()
        {
            if (_bufferedLogs.Count == 0) return;

            var logEntry = new LogEntry
            {
                EventSpan = new EventSpan
                {
                    Name = "BufferedLogs",
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

        private void AddLogEntry(Schema.LogType logType, string message, Dictionary<string, string> arguments)
        {
            var entry = new LogEntry
            {
                Event = new Schema.Event
                {
                    LogType = logType,
                    Message = message,
                    Arguments = arguments
                }
            };

            if (_activeSpans.Count > 0)
            {
                foreach (var spanName in _activeSpans.Keys.ToList())
                {
                    _activeSpans[spanName].Add(entry);
                    _lastLogTimes[spanName] = Time.time;
                }
            }
            else
            {
                _bufferedLogs.Add(entry);
                _lastBufferedLogTime = Time.time;
            }
        }

        private void OnUnityLogMessageReceived(string condition, string _stackTrace, UnityEngine.LogType type)
        {
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