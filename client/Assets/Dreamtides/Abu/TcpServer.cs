#nullable enable

using System;
using System.Collections.Concurrent;
using System.IO;
using System.Net;
using System.Net.Sockets;
using System.Threading;
using Newtonsoft.Json;
using UnityEngine;

namespace Abu
{
  /// <summary>
  /// TCP server that listens for incoming connections from the ABU CLI on a
  /// background thread. Received commands are enqueued for main-thread
  /// processing. Responses are written directly to the active client stream.
  /// Uses NDJSON framing (one JSON object per line).
  /// </summary>
  public class TcpServer
  {
    readonly int _port;
    readonly ConcurrentQueue<AbuCommand> _receiveQueue = new ConcurrentQueue<AbuCommand>();
    readonly CancellationTokenSource _cancellation = new CancellationTokenSource();
    readonly object _clientLock = new object();

    TcpListener? _listener;
    TcpClient? _activeClient;
    StreamWriter? _activeWriter;
    Thread? _acceptThread;

    public TcpServer(int port)
    {
      _port = port;
    }

    /// <summary>
    /// Queue of received commands, to be dequeued on the main thread.
    /// </summary>
    public ConcurrentQueue<AbuCommand> ReceiveQueue => _receiveQueue;

    /// <summary>
    /// Start the TCP listener and background accept thread.
    /// </summary>
    public void Start()
    {
      _listener = new TcpListener(IPAddress.Loopback, _port);
      _listener.Start();
      Debug.Log($"[Abu] TCP server listening on port {_port}");

      _acceptThread = new Thread(AcceptLoop) { Name = "AbuTcpAccept", IsBackground = true };
      _acceptThread.Start();
    }

    /// <summary>
    /// Send a response to the currently connected client. If no client is
    /// connected, the response is silently dropped.
    /// </summary>
    public void Send(AbuResponse response)
    {
      var json = JsonConvert.SerializeObject(response);
      lock (_clientLock)
      {
        if (_activeWriter == null)
        {
          return;
        }

        try
        {
          _activeWriter.WriteLine(json);
          _activeWriter.Flush();
        }
        catch (Exception ex)
        {
          Debug.LogWarning($"[Abu] Failed to send response: {ex.Message}");
        }
      }
    }

    /// <summary>
    /// Stop the TCP listener, close any active client, and cancel
    /// background threads.
    /// </summary>
    public void Shutdown()
    {
      _cancellation.Cancel();

      try
      {
        _listener?.Stop();
      }
      catch (Exception)
      {
        // Ignore errors during shutdown
      }

      CloseActiveClient();
      _cancellation.Dispose();
    }

    void AcceptLoop()
    {
      var token = _cancellation.Token;

      while (!token.IsCancellationRequested)
      {
        try
        {
          var client = _listener!.AcceptTcpClient();

          CloseActiveClient();

          lock (_clientLock)
          {
            _activeClient = client;
            _activeWriter = new StreamWriter(client.GetStream()) { AutoFlush = false };
          }

          var readThread = new Thread(() =>
          {
            ReadLoop(client);
            CloseClientIfActive(client);
          })
          {
            Name = "AbuTcpRead",
            IsBackground = true,
          };
          readThread.Start();
        }
        catch (SocketException) when (token.IsCancellationRequested)
        {
          break;
        }
        catch (ObjectDisposedException)
        {
          break;
        }
        catch (Exception ex) when (!token.IsCancellationRequested)
        {
          Debug.LogWarning($"[Abu] Accept error: {ex.Message}");
        }
      }
    }

    void ReadLoop(TcpClient client)
    {
      var token = _cancellation.Token;

      try
      {
        using var reader = new StreamReader(
          client.GetStream(),
          System.Text.Encoding.UTF8,
          detectEncodingFromByteOrderMarks: true,
          bufferSize: 1024,
          leaveOpen: true
        );

        while (!token.IsCancellationRequested && client.Connected)
        {
          string? line;
          try
          {
            line = reader.ReadLine();
          }
          catch (IOException)
          {
            break;
          }

          if (line == null)
          {
            break;
          }

          if (string.IsNullOrWhiteSpace(line))
          {
            continue;
          }

          try
          {
            var command = JsonConvert.DeserializeObject<AbuCommand>(line);
            if (command != null)
            {
              _receiveQueue.Enqueue(command);
            }
            else
            {
              Debug.LogWarning($"[Abu] Received null command from JSON: {line}");
            }
          }
          catch (JsonException ex)
          {
            Debug.LogWarning($"[Abu] Failed to parse command: {ex.Message}");
          }
        }
      }
      catch (Exception ex) when (!token.IsCancellationRequested)
      {
        Debug.LogWarning($"[Abu] Read error: {ex.Message}");
      }
    }

    void CloseClientIfActive(TcpClient client)
    {
      lock (_clientLock)
      {
        if (_activeClient == client)
        {
          CloseActiveClient();
        }
      }
    }

    void CloseActiveClient()
    {
      lock (_clientLock)
      {
        try
        {
          _activeWriter?.Dispose();
        }
        catch (Exception)
        {
          // Ignore
        }

        try
        {
          _activeClient?.Close();
        }
        catch (Exception)
        {
          // Ignore
        }

        _activeWriter = null;
        _activeClient = null;
      }
    }
  }
}
