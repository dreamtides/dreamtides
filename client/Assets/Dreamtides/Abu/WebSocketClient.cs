#nullable enable

using System;
using System.Collections.Concurrent;
using System.Net.WebSockets;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using Newtonsoft.Json;
using UnityEngine;

namespace Abu
{
    /// <summary>
    /// WebSocket client that connects to the ABU daemon on a background thread.
    /// Received commands are enqueued for main-thread processing.
    /// Responses are enqueued for background-thread sending.
    /// </summary>
    public class WebSocketClient
    {
        const int ReceiveBufferSize = 65536;
        const int ReconnectDelayMs = 2000;

        readonly string _uri;
        readonly ConcurrentQueue<AbuCommand> _receiveQueue = new ConcurrentQueue<AbuCommand>();
        readonly ConcurrentQueue<string> _sendQueue = new ConcurrentQueue<string>();
        readonly CancellationTokenSource _cancellation = new CancellationTokenSource();
        readonly object _socketLock = new object();
        readonly ManualResetEventSlim _sendSignal = new ManualResetEventSlim(false);

        ClientWebSocket? _webSocket;
        Thread? _receiveThread;
        Thread? _sendThread;

        public WebSocketClient(int port)
        {
            _uri = $"ws://localhost:{port}";
        }

        /// <summary>
        /// Queue of received commands, to be dequeued on the main thread.
        /// </summary>
        public ConcurrentQueue<AbuCommand> ReceiveQueue => _receiveQueue;

        /// <summary>
        /// Start the background threads for receiving and sending.
        /// </summary>
        public void Start()
        {
            _receiveThread = new Thread(ReceiveLoop)
            {
                Name = "AbuWebSocketReceive",
                IsBackground = true,
            };
            _receiveThread.Start();

            _sendThread = new Thread(SendLoop) { Name = "AbuWebSocketSend", IsBackground = true };
            _sendThread.Start();
        }

        /// <summary>
        /// Enqueue a response to be sent to the daemon.
        /// </summary>
        public void Send(AbuResponse response)
        {
            var json = JsonConvert.SerializeObject(response);
            _sendQueue.Enqueue(json);
            _sendSignal.Set();
        }

        /// <summary>
        /// Shut down the WebSocket and stop background threads.
        /// </summary>
        public void Shutdown()
        {
            _cancellation.Cancel();
            _sendSignal.Set();

            lock (_socketLock)
            {
                try
                {
                    if (_webSocket != null && _webSocket.State == WebSocketState.Open)
                    {
                        _webSocket
                            .CloseAsync(
                                WebSocketCloseStatus.NormalClosure,
                                "Shutting down",
                                CancellationToken.None
                            )
                            .Wait(TimeSpan.FromSeconds(2));
                    }
                }
                catch (Exception)
                {
                    // Ignore errors during shutdown
                }

                _webSocket?.Dispose();
                _webSocket = null;
            }

            _cancellation.Dispose();
            _sendSignal.Dispose();
        }

        void ReceiveLoop()
        {
            var token = _cancellation.Token;

            while (!token.IsCancellationRequested)
            {
                try
                {
                    ConnectAsync(token).Wait();
                    ReceiveMessagesAsync(token).Wait();
                }
                catch (Exception ex) when (!token.IsCancellationRequested)
                {
                    Debug.LogWarning($"[Abu] WebSocket connection lost: {ex.Message}");
                    CleanupSocket();

                    try
                    {
                        Task.Delay(ReconnectDelayMs, token).Wait();
                    }
                    catch (Exception)
                    {
                        // Cancellation during delay
                        break;
                    }
                }
                catch (Exception)
                {
                    // Cancellation requested
                    break;
                }
            }
        }

        async Task ConnectAsync(CancellationToken token)
        {
            var ws = new ClientWebSocket();
            Debug.Log($"[Abu] Connecting to {_uri}...");
            await ws.ConnectAsync(new Uri(_uri), token);
            lock (_socketLock)
            {
                _webSocket = ws;
            }
            Debug.Log("[Abu] WebSocket connected.");
        }

        async Task ReceiveMessagesAsync(CancellationToken token)
        {
            var buffer = new byte[ReceiveBufferSize];
            var messageBuilder = new StringBuilder();

            while (!token.IsCancellationRequested)
            {
                ClientWebSocket? ws;
                lock (_socketLock)
                {
                    ws = _webSocket;
                }

                if (ws == null || ws.State != WebSocketState.Open)
                {
                    return;
                }

                var segment = new ArraySegment<byte>(buffer);
                var result = await ws.ReceiveAsync(segment, token);

                if (result.MessageType == WebSocketMessageType.Close)
                {
                    Debug.LogWarning("[Abu] WebSocket closed by server.");
                    return;
                }

                messageBuilder.Append(Encoding.UTF8.GetString(buffer, 0, result.Count));

                if (result.EndOfMessage)
                {
                    var json = messageBuilder.ToString();
                    messageBuilder.Clear();

                    try
                    {
                        var command = JsonConvert.DeserializeObject<AbuCommand>(json);
                        if (command != null)
                        {
                            _receiveQueue.Enqueue(command);
                        }
                        else
                        {
                            Debug.LogWarning($"[Abu] Received null command from JSON: {json}");
                        }
                    }
                    catch (JsonException ex)
                    {
                        Debug.LogWarning($"[Abu] Failed to parse command: {ex.Message}");
                    }
                }
            }
        }

        void SendLoop()
        {
            var token = _cancellation.Token;

            while (!token.IsCancellationRequested)
            {
                try
                {
                    _sendSignal.Wait(token);
                    _sendSignal.Reset();
                }
                catch (OperationCanceledException)
                {
                    break;
                }

                while (_sendQueue.TryDequeue(out var json))
                {
                    ClientWebSocket? ws;
                    lock (_socketLock)
                    {
                        ws = _webSocket;
                    }

                    if (ws == null || ws.State != WebSocketState.Open)
                    {
                        break;
                    }

                    try
                    {
                        var bytes = Encoding.UTF8.GetBytes(json);
                        var segment = new ArraySegment<byte>(bytes);
                        ws.SendAsync(segment, WebSocketMessageType.Text, true, token)
                            .Wait(token);
                    }
                    catch (Exception ex) when (!token.IsCancellationRequested)
                    {
                        Debug.LogWarning($"[Abu] Failed to send message: {ex.Message}");
                    }
                    catch (Exception)
                    {
                        return;
                    }
                }
            }
        }

        void CleanupSocket()
        {
            lock (_socketLock)
            {
                try
                {
                    _webSocket?.Dispose();
                }
                catch (Exception)
                {
                    // Ignore
                }
                _webSocket = null;
            }
        }
    }
}
