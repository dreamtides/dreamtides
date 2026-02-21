#nullable enable

using System;
using System.Collections;
using System.Collections.Generic;
using UnityEngine;

namespace Abu
{
    /// <summary>
    /// Command handler that processes all ABU commands: snapshot, click, hover, drag,
    /// screenshot, launch, and close. Uses the ref registry and scene walkers.
    /// </summary>
    public class SnapshotCommandHandler : ICommandHandler
    {
        readonly AbuBridge _bridge;
        readonly List<ISceneWalker> _walkers;
        readonly RefRegistry _refRegistry;
        ISettledProvider _settledProvider;

        public SnapshotCommandHandler(
            AbuBridge bridge,
            List<ISceneWalker> walkers,
            RefRegistry refRegistry,
            ISettledProvider settledProvider
        )
        {
            _bridge = bridge;
            _walkers = walkers;
            _refRegistry = refRegistry;
            _settledProvider = settledProvider;
        }

        /// <summary>
        /// Replace the settled provider at runtime.
        /// </summary>
        public void SetSettledProvider(ISettledProvider provider)
        {
            _settledProvider = provider;
        }

        public void HandleCommand(AbuCommand command, AbuBridge bridge, Action<AbuResponse> onComplete)
        {
            switch (command.Command)
            {
                case "snapshot":
                    HandleSnapshot(command, onComplete);
                    break;
                case "click":
                    HandleClick(command, onComplete);
                    break;
                case "hover":
                    HandleHover(command, onComplete);
                    break;
                case "drag":
                    HandleDrag(command, onComplete);
                    break;
                case "screenshot":
                    HandleScreenshot(command, onComplete);
                    break;
                case "launch":
                    HandleLaunch(command, onComplete);
                    break;
                case "close":
                    HandleClose(command, onComplete);
                    break;
                default:
                    onComplete(
                        new AbuResponse
                        {
                            Id = command.Id,
                            Success = false,
                            Error = $"Unknown command: {command.Command}",
                        }
                    );
                    break;
            }
        }

        void HandleSnapshot(AbuCommand command, Action<AbuResponse> onComplete)
        {
            _refRegistry.Clear();
            var snapshotParams = command.Params?.ToObject<SnapshotParams>();
            var compact = snapshotParams?.Compact ?? false;
            var snapshotData = BuildSnapshotData(compact);

            onComplete(
                new AbuResponse
                {
                    Id = command.Id,
                    Success = true,
                    Data = snapshotData,
                }
            );
        }

        SnapshotData BuildSnapshotData(bool compact)
        {
            var rootChildren = new List<AbuSceneNode>();
            foreach (var walker in _walkers)
            {
                var subtree = walker.Walk(_refRegistry);
                rootChildren.Add(subtree);
            }

            var rootNode = new AbuSceneNode
            {
                Role = "application",
                Label = null,
                Interactive = false,
                Children = rootChildren,
            };

            var result = SnapshotFormatter.Format(new List<AbuSceneNode> { rootNode }, compact);
            return new SnapshotData
            {
                Snapshot = result.Snapshot,
                Refs = result.Refs,
            };
        }

        void HandleClick(AbuCommand command, Action<AbuResponse> onComplete)
        {
            DispatchRefAction(
                command,
                onComplete,
                "click",
                cb => { if (cb.OnClick == null) return false; cb.OnClick(); return true; },
                new { clicked = true }
            );
        }

        void HandleHover(AbuCommand command, Action<AbuResponse> onComplete)
        {
            DispatchRefAction(
                command,
                onComplete,
                "hover",
                cb => { if (cb.OnHover == null) return false; cb.OnHover(); return true; },
                new { hovered = true }
            );
        }

        void HandleDrag(AbuCommand command, Action<AbuResponse> onComplete)
        {
            var dragParams = command.Params?.ToObject<DragParams>();
            if (dragParams == null || string.IsNullOrEmpty(dragParams.Source))
            {
                onComplete(
                    new AbuResponse
                    {
                        Id = command.Id,
                        Success = false,
                        Error = "Missing source parameter for drag command",
                    }
                );
                return;
            }

            DispatchRefAction(
                command,
                onComplete,
                "drag",
                dragParams.Source,
                cb => { if (cb.OnDrag == null) return false; cb.OnDrag(dragParams.Target); return true; },
                new { dragged = true }
            );
        }

        /// <summary>
        /// Shared dispatch logic for ref-based actions (click, hover, drag).
        /// Parses RefParams from the command to obtain the ref string.
        /// </summary>
        void DispatchRefAction(
            AbuCommand command,
            Action<AbuResponse> onComplete,
            string actionName,
            Func<RefCallbacks, bool> tryInvoke,
            object responseData
        )
        {
            var refParams = command.Params?.ToObject<RefParams>();
            if (refParams == null || string.IsNullOrEmpty(refParams.Ref))
            {
                onComplete(
                    new AbuResponse
                    {
                        Id = command.Id,
                        Success = false,
                        Error = $"Missing ref parameter for {actionName} command",
                    }
                );
                return;
            }

            DispatchRefAction(command, onComplete, actionName, refParams.Ref, tryInvoke, responseData);
        }

        /// <summary>
        /// Shared dispatch logic for ref-based actions with an explicit ref string.
        /// Looks up the ref in the registry, invokes the callback, and waits for settled.
        /// </summary>
        void DispatchRefAction(
            AbuCommand command,
            Action<AbuResponse> onComplete,
            string actionName,
            string refStr,
            Func<RefCallbacks, bool> tryInvoke,
            object responseData
        )
        {
            if (!_refRegistry.TryGetCallbacks(refStr, out var callbacks))
            {
                onComplete(
                    new AbuResponse
                    {
                        Id = command.Id,
                        Success = false,
                        Error = $"Ref {refStr} not found",
                    }
                );
                return;
            }

            if (!tryInvoke(callbacks))
            {
                onComplete(
                    new AbuResponse
                    {
                        Id = command.Id,
                        Success = false,
                        Error = $"Ref {refStr} does not support {actionName}",
                    }
                );
                return;
            }

            _settledProvider.NotifyActionDispatched();
            _bridge.StartCoroutine(WaitForSettledThenRespond(command.Id, responseData, onComplete));
        }

        void HandleScreenshot(AbuCommand command, Action<AbuResponse> onComplete)
        {
            _bridge.StartCoroutine(CaptureScreenshot(command.Id, onComplete));
        }

        void HandleLaunch(AbuCommand command, Action<AbuResponse> onComplete)
        {
            onComplete(
                new AbuResponse
                {
                    Id = command.Id,
                    Success = true,
                    Data = new { launched = true },
                }
            );
        }

        void HandleClose(AbuCommand command, Action<AbuResponse> onComplete)
        {
            onComplete(
                new AbuResponse
                {
                    Id = command.Id,
                    Success = true,
                    Data = new { closed = true },
                }
            );
        }

        IEnumerator WaitForSettledThenRespond(
            string commandId,
            object data,
            Action<AbuResponse> onComplete
        )
        {
            while (!_settledProvider.IsSettled())
            {
                yield return null;
            }

            _refRegistry.Clear();
            var snapshotData = BuildSnapshotData(false);

            onComplete(
                new AbuResponse
                {
                    Id = commandId,
                    Success = true,
                    Data = new ActionSnapshotData
                    {
                        ActionData = data,
                        Snapshot = snapshotData.Snapshot,
                        Refs = snapshotData.Refs,
                    },
                }
            );
        }

        IEnumerator CaptureScreenshot(string commandId, Action<AbuResponse> onComplete)
        {
            yield return new WaitForEndOfFrame();

            try
            {
                var texture = ScreenCapture.CaptureScreenshotAsTexture();
                var pngBytes = texture.EncodeToPNG();
                UnityEngine.Object.Destroy(texture);
                var base64 = Convert.ToBase64String(pngBytes);

                onComplete(
                    new AbuResponse
                    {
                        Id = commandId,
                        Success = true,
                        Data = new { base64 },
                    }
                );
            }
            catch (Exception ex)
            {
                onComplete(
                    new AbuResponse
                    {
                        Id = commandId,
                        Success = false,
                        Error = $"Screenshot failed: {ex.Message}",
                    }
                );
            }
        }
    }
}
