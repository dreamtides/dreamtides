#nullable enable

using System;

namespace Abu
{
    /// <summary>
    /// Interface for handling incoming commands from the daemon.
    /// Implementations process commands and invoke the completion callback with the response.
    /// </summary>
    public interface ICommandHandler
    {
        /// <summary>
        /// Handle an incoming command. Invoke the completion callback with the response
        /// when done. For synchronous commands, invoke the callback immediately.
        /// For multi-frame commands (e.g., click, drag), start a coroutine via the
        /// provided AbuBridge and invoke the callback when complete.
        /// </summary>
        void HandleCommand(AbuCommand command, AbuBridge bridge, Action<AbuResponse> onComplete);
    }

    /// <summary>
    /// Default command handler that returns an error for all commands.
    /// Used when no handler has been registered.
    /// </summary>
    public class DefaultCommandHandler : ICommandHandler
    {
        public void HandleCommand(
            AbuCommand command,
            AbuBridge bridge,
            Action<AbuResponse> onComplete
        )
        {
            onComplete(
                new AbuResponse
                {
                    Id = command.Id,
                    Success = false,
                    Error = $"No command handler registered for command: {command.Command}",
                }
            );
        }
    }
}
