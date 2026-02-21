#nullable enable

using System;
using System.Collections.Generic;
using UnityEngine;

namespace Abu
{
    /// <summary>
    /// Main MonoBehaviour for the ABU bridge. Listens for incoming TCP
    /// connections, dispatches received commands to a registered handler on
    /// the main thread, and sends responses back.
    /// </summary>
    [AddComponentMenu("")]
    public class AbuBridge : MonoBehaviour
    {
        const int DefaultPort = 9999;

        TcpServer? _tcpServer;
        ICommandHandler _commandHandler = new DefaultCommandHandler();
        readonly List<ISceneWalker> _walkers = new List<ISceneWalker>();
        readonly RefRegistry _refRegistry = new RefRegistry();
        ISettledProvider _settledProvider = new DefaultSettledProvider();
        SnapshotCommandHandler? _snapshotCommandHandler;

        /// <summary>
        /// Register a scene walker. Walkers are called in registration order during snapshot.
        /// The snapshot command handler is created on the first walker registration and
        /// automatically sees subsequent walkers via the shared walker list.
        /// </summary>
        public void RegisterWalker(ISceneWalker walker)
        {
            _walkers.Add(walker ?? throw new ArgumentNullException(nameof(walker)));
            if (_snapshotCommandHandler == null)
            {
                RebuildSnapshotHandler();
            }
        }

        /// <summary>
        /// Set the settled provider. Replaces the default frame-counting provider.
        /// </summary>
        public void SetSettledProvider(ISettledProvider provider)
        {
            _settledProvider = provider ?? throw new ArgumentNullException(nameof(provider));
            if (_snapshotCommandHandler != null)
            {
                _snapshotCommandHandler.SetSettledProvider(_settledProvider);
            }
        }

        /// <summary>
        /// Access the ref registry for external use (e.g., by walkers during snapshot).
        /// </summary>
        public RefRegistry RefRegistry => _refRegistry;

        void RebuildSnapshotHandler()
        {
            _snapshotCommandHandler = new SnapshotCommandHandler(
                this,
                _walkers,
                _refRegistry,
                _settledProvider
            );
            _commandHandler = _snapshotCommandHandler;
        }

        void Awake()
        {
            DontDestroyOnLoad(gameObject);
            var portString = Environment.GetEnvironmentVariable("ABU_PORT");
            if (string.IsNullOrEmpty(portString))
            {
                portString = Environment.GetEnvironmentVariable("ABU_WS_PORT");
            }

            var port = DefaultPort;
            if (!string.IsNullOrEmpty(portString) && int.TryParse(portString, out var parsed))
            {
                port = parsed;
            }

            _tcpServer = new TcpServer(port);
            _tcpServer.Start();
        }

        void Update()
        {
            if (_tcpServer == null)
            {
                return;
            }

            while (_tcpServer.ReceiveQueue.TryDequeue(out var command))
            {
                _commandHandler.HandleCommand(
                    command,
                    this,
                    response =>
                    {
                        _tcpServer.Send(response);
                    }
                );
            }
        }

        void OnDestroy()
        {
            _tcpServer?.Shutdown();
            _tcpServer = null;
        }
    }
}
