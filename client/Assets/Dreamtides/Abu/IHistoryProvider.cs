#nullable enable

using System.Collections.Generic;

namespace Abu
{
    /// <summary>
    /// Interface for providing history entries that describe game events
    /// occurring between an action dispatch and the settled state.
    /// </summary>
    public interface IHistoryProvider
    {
        /// <summary>
        /// Returns accumulated history entries and clears the buffer.
        /// Returns null if no events occurred.
        /// </summary>
        List<string>? TakeHistory();
    }
}
