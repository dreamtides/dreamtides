#nullable enable

using System;

namespace Abu
{
    /// <summary>
    /// Ref-counted token that suppresses settled detection while any instance is alive.
    /// Acquire a token at the start of a multi-step coroutine and dispose it when done.
    /// </summary>
    public sealed class BusyToken : IDisposable
    {
        static int _activeCount;
        bool _disposed;

        /// <summary>
        /// True when at least one BusyToken has been created and not yet disposed.
        /// </summary>
        public static bool IsAnyActive => _activeCount > 0;

        public BusyToken() => _activeCount++;

        public void Dispose()
        {
            if (!_disposed)
            {
                _disposed = true;
                _activeCount--;
            }
        }
    }
}
