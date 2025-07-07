#nullable enable

using System.Collections;
using System.Collections.Generic;
using Dreamtides.Schema;
using UnityEngine;
using UnityEngine.Networking;
using Newtonsoft.Json;
using System;
using Dreamtides.Utils;
using System.Linq;

namespace Dreamtides.Services
{
  public class ActionService : Service
  {
    private struct CommandBatch
    {
      public CommandSequence? Commands;
      public bool Animate;
      public Action OnComplete;
    }

    readonly Guid _userGuid = Guid.Parse("d2da9785-f20e-4879-bed5-35b2e1926faf");
    bool _devModeAutoConnect;
    float _lastConnectAttemptTime;
    float _lastActionTime;
    Metadata? _metadata;
    Guid? _integrationTestId;
    Guid? _integrationTestEnemyId;
    Queue<CommandBatch> _commandQueue = new Queue<CommandBatch>();
    bool _isProcessingCommands = false;
    Dictionary<Guid, float> _requestStartTimes = new Dictionary<Guid, float>();
    HashSet<Guid> _loggedRequestIds = new HashSet<Guid>();
    Guid? _lastResponseVersion;

    bool UseDevServer => Application.isEditor && !Application.dataPath.Contains("dreamtides_tests");

    public bool Connected { get; private set; }

    public float LastActionTime => _lastActionTime;

    public bool IsProcessingCommands => _isProcessingCommands;

    /// <summary>
    /// Returns the Request ID of the last request for which a 'final' poll
    /// response was received.
    /// </summary>
    public Guid? LastResponseReceived { get; private set; }

    protected override void OnInitialize(TestConfiguration? testConfiguration)
    {
      Connected = false;
      _lastActionTime = Time.time;
      _integrationTestId = testConfiguration?.IntegrationTestId;
      _integrationTestEnemyId = Guid.NewGuid();
      StartCoroutine(InitializeAsync());
    }

    IEnumerator InitializeAsync()
    {
      yield return new WaitForEndOfFrame();
      _metadata = new Metadata
      {
        UserId = _userGuid,
        IntegrationTestId = _integrationTestId,
      };
      yield return PerformConnect(isReconnect: false, startLoggingSpan: true);
    }

    protected override void OnUpdate()
    {
      if (_devModeAutoConnect)
      {
        var now = Time.time;
        if (now - _lastConnectAttemptTime > 0.5f)
        {
          StartCoroutine(DevServerConnectAsync(CreateConnectRequest(), reconnect: true));
          _lastConnectAttemptTime = now;
        }
      }
      else if (_metadata != null)
      {
        if (UseDevServer)
        {
          StartCoroutine(DevServerPollAsync(new PollRequest
          {
            Metadata = Errors.CheckNotNull(_metadata)
          }));
        }
        else
        {
          var request = new PollRequest
          {
            Metadata = Errors.CheckNotNull(_metadata)
          };
          var response = Plugin.Poll(request);
          StartCoroutine(HandlePollResponse(response));
        }
      }
    }

    public void PerformAction(GameAction? action, Guid? requestIdentifier = null)
    {
      if (action == null)
      {
        return;
      }

      _lastActionTime = Time.time;
      var requestId = requestIdentifier ?? Guid.NewGuid();
      Registry.LoggingService.StartSpan(LogSpanName.PerformAction);
      Registry.LoggingService.Log("ActionService", "Performing action",
        ("actionType", GameActionHelper.GetActionName(action.Value)),
        ("requestId", requestId.ToString()));
      _requestStartTimes[requestId] = Time.time;

      var request = new PerformActionRequest
      {
        Metadata = new Metadata
        {
          UserId = Errors.CheckNotNull(_metadata).UserId,
          BattleId = Errors.CheckNotNull(_metadata).BattleId,
          RequestId = requestId,
          IntegrationTestId = _integrationTestId,
        },
        Action = action.Value,
        LastResponseVersion = _lastResponseVersion,
      };
      if (UseDevServer)
      {
        StartCoroutine(PerformDevServerActionAsync(request));
      }
      else
      {
        var startTime = Time.time;
        var response = Plugin.PerformAction(request);
        var elapsedTime = Time.time - startTime;
        Registry.LoggingService.Log("ActionService", "PerformAction completed",
          ("elapsedTime", $"{elapsedTime:F2}s"));
        StartCoroutine(ApplyCommands(response.Commands, animate: true, onComplete: () =>
        {
          Registry.LoggingService.EndSpan(LogSpanName.PerformAction);
        }));
      }
    }

    /// <summary>
    /// Log a client message to the server.
    /// </summary>
    public void Log(ClientLogRequest request)
    {
      if (UseDevServer)
      {
        StartCoroutine(DevServerLogAsync(request));
      }
      else
      {
        Plugin.Log(request);
      }
    }

    public void TriggerReconnect()
    {
      if (_metadata == null) return;

      Debug.Log("Triggering idle reconnect");
      Registry.LoggingService.Log("ActionService", "Triggering idle reconnect");
      StartCoroutine(PerformConnect(isReconnect: true, startLoggingSpan: false));
    }

    private IEnumerator PerformConnect(bool isReconnect, bool startLoggingSpan = false)
    {
      if (_metadata == null) yield break;

      var request = CreateConnectRequest();
      if (startLoggingSpan)
      {
        Registry.LoggingService.StartSpan(LogSpanName.Connect);
      }

      if (UseDevServer)
      {
        yield return DevServerConnectAsync(request, isReconnect);
      }
      else
      {
        var response = Plugin.Connect(request);
        yield return ApplyCommands(response.Commands, animate: false, onComplete: () =>
        {
          if (startLoggingSpan)
          {
            Registry.LoggingService.EndSpan(LogSpanName.Connect);
          }
          if (response.ResponseVersion != null)
          {
            _lastResponseVersion = response.ResponseVersion;
          }
        });
      }
    }

    private IEnumerator DevServerConnectAsync(ConnectRequest request, bool reconnect)
    {
      if (!reconnect)
      {
        Registry.LoggingService.Log("ActionService", "Connecting...");
      }

      yield return SendDevServerRequest<ConnectRequest, ConnectResponse>(
        request,
        "connect",
        UnityWebRequest.kHttpVerbGET,
        response =>
        {
          return ApplyCommands(response.Commands, animate: false, onComplete: () =>
          {
            Registry.LoggingService.EndSpan(LogSpanName.Connect);
            if (response.ResponseVersion != null)
            {
              _lastResponseVersion = response.ResponseVersion;
            }
          });
        });
    }

    private DisplayProperties GetDisplayProperties() => new()
    {
      IsMobileDevice = UnityEngine.Device.Application.isMobilePlatform,
      ScreenHeight = Screen.height,
      ScreenWidth = Screen.width
    };

    private ConnectRequest CreateConnectRequest()
    {
      return new ConnectRequest
      {
        Metadata = Errors.CheckNotNull(_metadata),
        PersistentDataPath = Application.persistentDataPath,
        DisplayProperties = GetDisplayProperties(),
        DebugConfiguration = _integrationTestEnemyId == null ? null : new DebugConfiguration
        {
          Enemy = new PlayerType
          {
            User = _integrationTestEnemyId,
          },
          Seed = 1234567890,
        }
      };
    }

    private IEnumerator HandlePollResponse(PollResponse response)
    {
      if (response.Metadata?.RequestId != null && response.ResponseType == PollResponseType.Final)
      {
        LastResponseReceived = response.Metadata.RequestId;
      }

      if (response.Commands?.Groups.Count > 0)
      {
        Registry.LoggingService.StartSpan(LogSpanName.Poll);
        LogPollResponseTiming(response);
        yield return ApplyCommands(response.Commands, animate: true, onComplete: () =>
        {
          Registry.LoggingService.EndSpan(LogSpanName.Poll);
          if (response.ResponseVersion != null)
          {
            _lastResponseVersion = response.ResponseVersion;
          }
        });
      }
    }

    private IEnumerator DevServerPollAsync(PollRequest request)
    {
      yield return SendDevServerRequest<PollRequest, PollResponse>(
        request,
        "poll",
        UnityWebRequest.kHttpVerbGET,
        response =>
        {
          return HandlePollResponse(response);
        }
      );
    }

    private IEnumerator PerformDevServerActionAsync(PerformActionRequest request)
    {
      yield return SendDevServerRequest<PerformActionRequest, PerformActionResponse>(
        request,
        "perform_action",
        UnityWebRequest.kHttpVerbPOST,
        response =>
        {
          Registry.LoggingService.Log("ActionService", "PerformAction completed");
          return ApplyCommands(response.Commands, animate: true, () =>
          {
            Registry.LoggingService.EndSpan(LogSpanName.PerformAction);
          });
        });
    }

    private IEnumerator DevServerLogAsync(ClientLogRequest request)
    {
      return SendDevServerRequest<ClientLogRequest, ClientLogResponse>(
        request,
        "log",
        UnityWebRequest.kHttpVerbPOST,
        response => Break());
    }

    IEnumerator Break()
    {
      yield break;
    }

    private void LogPollResponseTiming(PollResponse response)
    {
      if (response.Metadata?.RequestId != null &&
        _requestStartTimes.ContainsKey(response.Metadata.RequestId.Value) &&
        !_loggedRequestIds.Contains(response.Metadata.RequestId.Value))
      {
        var requestId = response.Metadata.RequestId.Value;
        var elapsedTime = Time.time - _requestStartTimes[requestId];
        var elapsedTimeMs = (int)(elapsedTime * 1000);
        _loggedRequestIds.Add(requestId);

        if (elapsedTimeMs > 100)
        {
          Registry.LoggingService.LogWarning("ActionService", "Poll response received (WARNING: Slow response)",
            ("elapsedTime", $"{elapsedTimeMs}ms"),
            ("requestId", requestId.ToString()));
        }
        else
        {
          Registry.LoggingService.Log("ActionService", "Poll response received",
            ("elapsedTime", $"{elapsedTimeMs}ms"),
            ("requestId", requestId.ToString()));
        }

        _requestStartTimes.Remove(requestId);
      }
    }

    private IEnumerator SendDevServerRequest<TRequest, TResponse>(
      TRequest request,
      string endpoint,
      string method,
      Func<TResponse, IEnumerator> handleResponse)
      where TResponse : class
    {
      var json = JsonConvert.SerializeObject(request, Converter.Settings);
      var url = $"http://localhost:26598/{endpoint}";
      var webRequest = new UnityWebRequest(url, method);
      webRequest.uploadHandler = new UploadHandlerRaw(System.Text.Encoding.UTF8.GetBytes(json));
      webRequest.downloadHandler = new DownloadHandlerBuffer();
      webRequest.SetRequestHeader("Content-Type", "application/json");

      yield return webRequest.SendWebRequest();

      if (webRequest.result == UnityWebRequest.Result.Success)
      {
        _devModeAutoConnect = false;
        var responseJson = webRequest.downloadHandler.text;
        var response = JsonConvert.DeserializeObject<TResponse>(responseJson, Converter.Settings);
        if (response != null)
        {
          yield return handleResponse(response);
        }
      }
      else
      {
        _devModeAutoConnect = true;
      }
    }

    IEnumerator ApplyCommands(CommandSequence? commands, bool animate, Action onComplete)
    {
      if (commands == null)
      {
        onComplete();
        yield break;
      }

      if (_isProcessingCommands)
      {
        Registry.LoggingService.Log("ActionService", "Queueing commands for later processing",
          ("queueSize", _commandQueue.Count.ToString()));
        _commandQueue.Enqueue(new CommandBatch { Commands = commands, Animate = animate, OnComplete = onComplete });
        yield break;
      }

      _isProcessingCommands = true;
      Registry.LoggingService.StartSpan(LogSpanName.ApplyCommands);

      var count = commands.Groups.Sum(group => group.Commands.Count);
      if (count > 1)
      {
        var commandNames = GetCommandNames(commands);
        Registry.LoggingService.Log("ActionService", "Applying commands",
          ("commandNames", commandNames));
      }

      foreach (var group in commands.Groups)
      {
        yield return ApplyGroup(group, animate);
      }

      if (!Connected)
      {
        yield return new WaitForEndOfFrame();
        Connected = true;
      }

      Registry.LoggingService.EndSpan(LogSpanName.ApplyCommands);
      _isProcessingCommands = false;

      if (_commandQueue.Count > 0)
      {
        Registry.LoggingService.Log("ActionService", "Processing queued command sequences",
          ("queueCount", _commandQueue.Count.ToString()));
        var nextBatch = _commandQueue.Dequeue();
        StartCoroutine(ApplyCommands(nextBatch.Commands, nextBatch.Animate, nextBatch.OnComplete));
      }

      onComplete();
    }

    IEnumerator ApplyGroup(ParallelCommandGroup group, bool animate)
    {
      if (group.Commands.Count > 1)
      {
        Registry.LoggingService.Log("ActionService", "Applying command group",
          ("commandCount", group.Commands.Count.ToString()));
      }
      var coroutines = new List<Coroutine>();

      Registry.CardEffectPreviewService.ClearBattlePreview();
      foreach (var command in group.Commands)
      {
        if (command.UpdateBattle != null)
        {
          Registry.LoggingService.Log("ActionService", "Applying command: UpdateBattle");
          Registry.Layout.UserStatusDisplay.UpdatePlayerView(command.UpdateBattle.Battle.User, animate);
          Registry.Layout.EnemyStatusDisplay.UpdatePlayerView(command.UpdateBattle.Battle.Enemy, animate);
          Registry.DocumentService.RenderScreenOverlay(command.UpdateBattle.Battle.Interface?.ScreenOverlay);
          Registry.Layout.CardOrderSelector.View = command.UpdateBattle.Battle.Interface?.CardOrderSelector;
          Registry.Layout.PrimaryActionButton.SetView(
              command.UpdateBattle.Battle.Interface?.PrimaryActionButton,
              null);
          Registry.Layout.SecondaryActionButton.SetView(command.UpdateBattle.Battle.Interface?.SecondaryActionButton);
          Registry.Layout.IncrementActionButton.SetView(command.UpdateBattle.Battle.Interface?.IncrementButton);
          Registry.Layout.DecrementActionButton.SetView(command.UpdateBattle.Battle.Interface?.DecrementButton);
          Registry.Layout.UndoButton.SetView(command.UpdateBattle.Battle.Interface?.UndoButton);
          Registry.Layout.DevButton.SetView(command.UpdateBattle.Battle.Interface?.DevButton);
          Registry.BottomRightButton.SetView(command.UpdateBattle.Battle.Interface?.BottomRightButton);
          coroutines.Add(StartCoroutine(Registry.LayoutService.UpdateLayout(
              command.UpdateBattle,
              animate ? TweenUtils.Sequence("UpdateLayout") : null)));

          // Must happen after UpdateLayout since cards may be created which are referenced
          Registry.ArrowService.HandleDisplayArrows(command.UpdateBattle.Battle.Arrows ?? new List<DisplayArrow>());

          if (command.UpdateBattle.Battle.Preview.BattlePreviewStateClass?.Active is { } preview)
          {
            Registry.CardEffectPreviewService.DisplayBattlePreview(preview);
          }
          else if (command.UpdateBattle.Battle.Preview.Enum == BattlePreviewStateEnum.None)
          {
            Registry.CardEffectPreviewService.ClearBattlePreview();
          }
        }

        if (command.Wait != null)
        {
          Registry.LoggingService.Log("ActionService", "Applying command: Wait",
            ("duration", $"{command.Wait.MillisecondsValue}ms"));
          coroutines.Add(StartCoroutine(Wait(command.Wait)));
        }

        if (command.FireProjectile != null)
        {
          Registry.LoggingService.Log("ActionService", "Applying command: FireProjectile");
          coroutines.Add(StartCoroutine(
            Registry.EffectService.HandleFireProjectileCommand(command.FireProjectile)));
        }

        if (command.DissolveCard != null)
        {
          Registry.LoggingService.Log("ActionService", "Applying command: DissolveCard");
          coroutines.Add(StartCoroutine(
            Registry.EffectService.HandleDissolveCommand(command.DissolveCard)));
        }

        if (command.DisplayGameMessage != null)
        {
          Registry.LoggingService.Log("ActionService", "Applying command: DisplayGameMessage",
            ("messageType", command.DisplayGameMessage.Value.ToString()));
          coroutines.Add(StartCoroutine(Registry.Layout.GameMessage.Show(command.DisplayGameMessage.Value)));
        }

        if (command.DisplayEffect != null)
        {
          Registry.LoggingService.Log("ActionService", "Applying command: DisplayEffect");
          coroutines.Add(StartCoroutine(Registry.EffectService.HandleDisplayEffectCommand(command.DisplayEffect)));
        }

        if (command.DrawUserCards != null)
        {
          Registry.LoggingService.Log("ActionService", "Applying command: DrawUserCards",
            ("cardCount", command.DrawUserCards.Cards.Count.ToString()));
          coroutines.Add(StartCoroutine(Registry.CardService.HandleDrawUserCards(command.DrawUserCards)));
        }

        if (command.DisplayJudgment != null)
        {
          Registry.LoggingService.Log("ActionService", "Applying command: DisplayJudgment");
          coroutines.Add(StartCoroutine(Registry.JudgmentService.HandleDisplayJudgmentCommand(command.DisplayJudgment)));
        }

        if (command.DisplayDreamwellActivation != null)
        {
          Registry.LoggingService.Log("ActionService", "Applying command: DisplayDreamwellActivation");
          coroutines.Add(StartCoroutine(Registry.DreamwellActivationService.HandleDreamwellActivationCommand(command.DisplayDreamwellActivation)));
        }

        if (command.DisplayEnemyMessage != null)
        {
          Registry.LoggingService.Log("ActionService", "Applying command: DisplayEnemyMessage");
          Registry.Layout.EnemyMessage.Show(command.DisplayEnemyMessage);
        }

        if (command.ToggleThinkingIndicator != null)
        {
          Registry.LoggingService.Log("ActionService", "Applying command: ToggleThinkingIndicator",
            ("show", command.ToggleThinkingIndicator.Show.ToString()));
          Registry.Layout.ThinkingIndicator.SetActive(command.ToggleThinkingIndicator.Show);
        }

        if (command.PlayAudioClip != null)
        {
          Registry.LoggingService.Log("ActionService", "Applying command: PlayAudioClip");
          Registry.SoundService.Play(command.PlayAudioClip.Sound);
          if (command.PlayAudioClip.PauseDuration.MillisecondsValue > 0)
          {
            coroutines.Add(StartCoroutine(Wait(command.PlayAudioClip.PauseDuration)));
          }
        }

        if (command.PlayStudioAnimation != null)
        {
          Registry.LoggingService.Log("ActionService", "Applying command: PlayStudioAnimation");
          Registry.StudioService.PlayStudioAnimation(command.PlayStudioAnimation);
        }
      }

      foreach (var coroutine in coroutines)
      {
        yield return coroutine;
      }
    }

    IEnumerator Wait(Milliseconds milliseconds)
    {
      yield return new WaitForSeconds(milliseconds.ToSeconds());
    }

    private string GetCommandNames(CommandSequence commands)
    {
      var commandNames = new List<string>();

      foreach (var group in commands.Groups)
      {
        foreach (var command in group.Commands)
        {
          if (command.UpdateBattle != null) commandNames.Add("UpdateBattle");
          if (command.Wait != null) commandNames.Add("Wait");
          if (command.FireProjectile != null) commandNames.Add("FireProjectile");
          if (command.DissolveCard != null) commandNames.Add("DissolveCard");
          if (command.DisplayGameMessage != null) commandNames.Add("DisplayGameMessage");
          if (command.DisplayEffect != null) commandNames.Add("DisplayEffect");
          if (command.DrawUserCards != null) commandNames.Add("DrawUserCards");
          if (command.DisplayJudgment != null) commandNames.Add("DisplayJudgment");
          if (command.DisplayDreamwellActivation != null) commandNames.Add("DisplayDreamwellActivation");
          if (command.DisplayEnemyMessage != null) commandNames.Add("DisplayEnemyMessage");
          if (command.ToggleThinkingIndicator != null) commandNames.Add("ToggleThinkingIndicator");
          if (command.PlayAudioClip != null) commandNames.Add("PlayAudioClip");
        }
      }

      return $"[{string.Join(", ", commandNames)}]";
    }
  }
}
