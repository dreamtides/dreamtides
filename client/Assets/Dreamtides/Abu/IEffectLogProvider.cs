#nullable enable

using System.Collections.Generic;

namespace Abu
{
  /// <summary>
  /// Interface for providing effect log entries that describe visual effects
  /// (particles, projectiles, dissolves, trails) occurring between an action
  /// dispatch and the settled state.
  /// </summary>
  public interface IEffectLogProvider
  {
    /// <summary>
    /// Clears any accumulated effect log entries. Called at action dispatch
    /// time to ensure only effects from the current action are recorded.
    /// </summary>
    void ClearEffectLogs();

    /// <summary>
    /// Returns accumulated effect log entries and clears the buffer.
    /// Returns null if no effects occurred.
    /// </summary>
    List<string>? TakeEffectLogs();
  }
}
