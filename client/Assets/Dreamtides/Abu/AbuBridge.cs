#nullable enable

using System;
using System.Collections.Generic;
using System.IO;
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
    IHistoryProvider? _historyProvider;
    IEffectLogProvider? _effectLogProvider;
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
    /// Set the history provider for recording game events between actions.
    /// </summary>
    public void SetHistoryProvider(IHistoryProvider provider)
    {
      _historyProvider = provider;
      if (_snapshotCommandHandler != null)
      {
        _snapshotCommandHandler.SetHistoryProvider(provider);
      }
    }

    /// <summary>
    /// Set the effect log provider for recording visual effect commands.
    /// </summary>
    public void SetEffectLogProvider(IEffectLogProvider provider)
    {
      _effectLogProvider = provider;
      if (_snapshotCommandHandler != null)
      {
        _snapshotCommandHandler.SetEffectLogProvider(provider);
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
      if (_historyProvider != null)
      {
        _snapshotCommandHandler.SetHistoryProvider(_historyProvider);
      }
      if (_effectLogProvider != null)
      {
        _snapshotCommandHandler.SetEffectLogProvider(_effectLogProvider);
      }
      _commandHandler = _snapshotCommandHandler;
    }

    static int ResolveWorktreePort()
    {
      try
      {
        // Application.dataPath gives ".../client/Assets"; go up 2 levels for repo root
        var repoRoot = Path.GetFullPath(Path.Combine(Application.dataPath, "..", ".."));
        var home = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
        var worktreeBase = Path.Combine(home, "dreamtides-worktrees");

        if (
          !repoRoot.StartsWith(worktreeBase + Path.DirectorySeparatorChar)
          && !repoRoot.StartsWith(worktreeBase + "/")
        )
        {
          return DefaultPort;
        }

        // Extract worktree name (first path component after worktreeBase)
        var relative = repoRoot.Substring(worktreeBase.Length + 1);
        var sep = relative.IndexOfAny(new[] { '/', Path.DirectorySeparatorChar });
        var worktreeName = sep >= 0 ? relative.Substring(0, sep) : relative;

        var portsFile = Path.Combine(worktreeBase, ".ports.json");
        if (!File.Exists(portsFile))
        {
          Debug.LogWarning(
            $"[Abu] Worktree '{worktreeName}' detected but {portsFile} not found, using default port"
          );
          return DefaultPort;
        }

        var json = File.ReadAllText(portsFile);
        // Simple JSON parsing without Newtonsoft dependency - parse {"name": port} pairs
        // The file format is: { "alpha": 10000, "beta": 10001 }
        var searchKey = $"\"{worktreeName}\"";
        var keyIndex = json.IndexOf(searchKey, StringComparison.Ordinal);
        if (keyIndex < 0)
        {
          Debug.LogWarning(
            $"[Abu] Worktree '{worktreeName}' not found in {portsFile}, using default port"
          );
          return DefaultPort;
        }

        var colonIndex = json.IndexOf(':', keyIndex + searchKey.Length);
        if (colonIndex < 0)
        {
          return DefaultPort;
        }

        // Extract the number after the colon
        var afterColon = json.Substring(colonIndex + 1).TrimStart();
        var numEnd = 0;
        while (numEnd < afterColon.Length && char.IsDigit(afterColon[numEnd]))
        {
          numEnd++;
        }

        if (numEnd > 0 && int.TryParse(afterColon.Substring(0, numEnd), out var port))
        {
          Debug.Log($"[Abu] Worktree '{worktreeName}' using port {port}");
          return port;
        }
      }
      catch (Exception e)
      {
        Debug.LogWarning($"[Abu] Failed to resolve worktree port: {e.Message}");
      }

      return DefaultPort;
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
      else
      {
        port = ResolveWorktreePort();
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
