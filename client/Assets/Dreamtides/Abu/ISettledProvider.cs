#nullable enable

namespace Abu
{
  /// <summary>
  /// Interface for detecting when the UI has settled after an input action.
  /// The command handler polls this each frame before sending a response.
  /// </summary>
  public interface ISettledProvider
  {
    /// <summary>
    /// Returns true when the UI has settled (all animations complete, no pending processing).
    /// </summary>
    bool IsSettled();

    /// <summary>
    /// Called when an input action is dispatched, to reset the settle detection.
    /// </summary>
    void NotifyActionDispatched();
  }
}
