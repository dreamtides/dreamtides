#nullable enable

using System;
using System.Collections.Generic;

namespace Abu
{
    /// <summary>
    /// Bundle of action callbacks associated with an interactive node.
    /// </summary>
    public class RefCallbacks
    {
        /// <summary>
        /// Invoked when the ref is clicked. May be null if not clickable.
        /// </summary>
        public Action? OnClick { get; set; }

        /// <summary>
        /// Invoked when the ref is hovered. May be null if not hoverable.
        /// </summary>
        public Action? OnHover { get; set; }

        /// <summary>
        /// Invoked when the ref is dragged. The parameter is the optional target ref string.
        /// May be null if not draggable.
        /// </summary>
        public Action<string?>? OnDrag { get; set; }
    }

    /// <summary>
    /// Maps monotonically assigned ref strings (e1, e2, ...) to action callbacks.
    /// Rebuilt from scratch on every snapshot; invalidated by the next.
    /// </summary>
    public class RefRegistry
    {
        int _nextId = 1;
        readonly Dictionary<string, RefCallbacks> _callbacks = new Dictionary<string, RefCallbacks>();

        /// <summary>
        /// Assign the next ref string and register its callbacks.
        /// Returns the assigned ref string (e.g., "e1", "e2").
        /// </summary>
        public string Register(RefCallbacks callbacks)
        {
            var refStr = $"e{_nextId}";
            _nextId++;
            _callbacks[refStr] = callbacks;
            return refStr;
        }

        /// <summary>
        /// Look up callbacks by ref string.
        /// </summary>
        public bool TryGetCallbacks(string refStr, out RefCallbacks callbacks)
        {
            return _callbacks.TryGetValue(refStr, out callbacks!);
        }

        /// <summary>
        /// Invalidate all refs and reset the counter for the next snapshot.
        /// </summary>
        public void Clear()
        {
            _callbacks.Clear();
            _nextId = 1;
        }
    }
}
