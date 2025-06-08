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
    }

    readonly Guid _userGuid = Guid.Parse("d2da9785-f20e-4879-bed5-35b2e1926faf");
    readonly Guid _testOpponentGuid = Guid.Parse("25e89dde-37d7-464b-8a1c-f985102ca029");
    bool _devModeAutoConnect;
    float _lastConnectAttemptTime;
    Metadata? _metadata;
    string? _testScenario;
    Queue<CommandBatch> _commandQueue = new Queue<CommandBatch>();
    bool _isProcessingCommands = false;
    float? _lastPerformActionTime;

    bool IsTestOpponentClient => Application.dataPath.Contains("test_client");

    public bool Connected { get; private set; }

    protected override void OnInitialize(TestConfiguration? testConfiguration)
    {
      Connected = false;
      _testScenario = testConfiguration?.TestScenario;
      StartCoroutine(InitializeAsync());
    }

    IEnumerator InitializeAsync()
    {
      yield return new WaitForEndOfFrame();
      _metadata = new Metadata
      {
        UserId = IsTestOpponentClient ? _testOpponentGuid : _userGuid
      };
      var request = new ConnectRequest
      {
        Metadata = _metadata,
        PersistentDataPath = Application.persistentDataPath,
        VsOpponent = IsTestOpponentClient ? _userGuid : null,
        TestScenario = _testScenario
      };

      if (Application.isEditor)
      {
        StartCoroutine(DevServerConnectAsync(request, reconnect: false));
      }
      else
      {
        var response = Plugin.Connect(request);
        StartCoroutine(ApplyCommands(response.Commands, animate: false));
      }
    }

    void Update()
    {
      if (_devModeAutoConnect)
      {
        var now = Time.time;
        if (now - _lastConnectAttemptTime > 0.5f)
        {
          StartCoroutine(DevServerConnectAsync(new ConnectRequest
          {
            Metadata = _metadata!,
            PersistentDataPath = Application.persistentDataPath,
            VsOpponent = IsTestOpponentClient ? _userGuid : null,
          }, reconnect: true));
          _lastConnectAttemptTime = now;
        }
      }
      else if (_metadata != null)
      {
        if (Application.isEditor)
        {
          StartCoroutine(DevServerPollAsync(new PollRequest
          {
            Metadata = _metadata!
          }));
        }
        else
        {
          var request = new PollRequest
          {
            Metadata = _metadata!
          };
          var response = Plugin.Poll(request);
          if (response.Commands?.Groups.Count > 0)
          {
            if (_lastPerformActionTime.HasValue)
            {
              var elapsedTime = Time.time - _lastPerformActionTime.Value;
              Registry.LoggingService.LogInfo("ActionService", "Poll response received",
                ("elapsedTime", $"{elapsedTime:F2}s"));
            }
            else
            {
              Registry.LoggingService.LogInfo("ActionService", "Poll response received, unknown time since last PerformAction");
            }
            StartCoroutine(ApplyCommands(response.Commands, animate: true));
          }
        }
      }
    }

    public void PerformAction(GameAction? action)
    {
      if (action == null)
      {
        return;
      }

      Registry.LoggingService.StartSpan("PerformAction");
      Registry.LoggingService.LogInfo("ActionService", "Performing action",
        ("actionType", action.ToString() ?? "null"));
      _lastPerformActionTime = Time.time;

      var request = new PerformActionRequest
      {
        Metadata = _metadata,
        Action = action.Value,
        VsOpponent = IsTestOpponentClient ? _userGuid : null,
        TestScenario = _testScenario
      };
      if (Application.isEditor)
      {
        StartCoroutine(PerformDevServerActionAsync(request));
      }
      else
      {
        var startTime = Time.time;
        var response = Plugin.PerformAction(request);
        var elapsedTime = Time.time - startTime;
        Registry.LoggingService.LogInfo("ActionService", "PerformAction response received",
          ("elapsedTime", $"{elapsedTime:F2}s"));
        Registry.LoggingService.EndSpan("PerformAction");
        StartCoroutine(ApplyCommands(response.Commands, animate: true));
      }
    }

    /// <summary>
    /// Log a client message to the server.
    /// </summary>
    public void Log(ClientLogRequest request)
    {
      if (Application.isEditor)
      {
        StartCoroutine(DevServerLogAsync(request));
      }
      else
      {
        Plugin.Log(request);
      }
    }

    private IEnumerator DevServerConnectAsync(ConnectRequest request, bool reconnect)
    {
      if (reconnect)
      {
        Debug.Log("Reconnecting...");
      }
      else
      {
        Debug.Log("Connecting...");
      }

      yield return SendDevServerRequest<ConnectRequest, ConnectResponse>(
        request,
        "connect",
        UnityWebRequest.kHttpVerbGET,
        response => ApplyCommands(response.Commands, animate: false));
    }

    private IEnumerator DevServerPollAsync(PollRequest request)
    {
      yield return SendDevServerRequest<PollRequest, PollResponse>(
        request,
        "poll",
        UnityWebRequest.kHttpVerbGET,
        response =>
        {
          if (response.Commands?.Groups.Count > 0 && _lastPerformActionTime.HasValue)
          {
            var elapsedTime = Time.time - _lastPerformActionTime.Value;
            Registry.LoggingService.LogInfo("ActionService", "Dev server poll response received",
              ("elapsedTime", $"{elapsedTime:F2}s"));
          }
          return ApplyCommands(response.Commands, animate: true);
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
          Registry.LoggingService.LogInfo("ActionService", "Dev server PerformAction response received");
          Registry.LoggingService.EndSpan("PerformAction");
          return ApplyCommands(response.Commands, animate: true);
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

    IEnumerator ApplyCommands(CommandSequence? commands, bool animate)
    {
      if (commands == null)
      {
        yield break;
      }

      if (_isProcessingCommands)
      {
        Registry.LoggingService.LogInfo("ActionService", "Queueing commands for later processing",
          ("queueSize", _commandQueue.Count.ToString()));
        _commandQueue.Enqueue(new CommandBatch { Commands = commands, Animate = animate });
        yield break;
      }

      _isProcessingCommands = true;
      Registry.LoggingService.StartSpan("ApplyCommands");

      var count = commands.Groups.Sum(group => group.Commands.Count);
      if (count > 1)
      {
        var commandNames = GetCommandNames(commands);
        Registry.LoggingService.LogInfo("ActionService", "Applying commands",
          ("commandCount", count.ToString()), ("commandNames", commandNames));
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

      Registry.LoggingService.EndSpan("ApplyCommands");
      _isProcessingCommands = false;

      if (_commandQueue.Count > 0)
      {
        Registry.LoggingService.LogInfo("ActionService", "Processing queued command sequences",
          ("queueCount", _commandQueue.Count.ToString()));
        var nextBatch = _commandQueue.Dequeue();
        StartCoroutine(ApplyCommands(nextBatch.Commands, nextBatch.Animate));
      }
    }

    IEnumerator ApplyGroup(ParallelCommandGroup group, bool animate)
    {
      if (group.Commands.Count > 1)
      {
        Registry.LoggingService.LogInfo("ActionService", "Applying command group",
          ("commandCount", group.Commands.Count.ToString()));
      }
      var coroutines = new List<Coroutine>();
      foreach (var command in group.Commands)
      {
        if (command.UpdateBattle != null)
        {
          Registry.LoggingService.LogInfo("ActionService", "Applying command: UpdateBattle");

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
          Registry.LoggingService.LogInfo("ActionService", "Applying command: Wait",
            ("duration", $"{command.Wait.MillisecondsValue}ms"));
          coroutines.Add(StartCoroutine(Wait(command.Wait)));
        }

        if (command.FireProjectile != null)
        {
          Registry.LoggingService.LogInfo("ActionService", "Applying command: FireProjectile");
          coroutines.Add(StartCoroutine(
            Registry.EffectService.HandleFireProjectileCommand(command.FireProjectile)));
        }

        if (command.DissolveCard != null)
        {
          Registry.LoggingService.LogInfo("ActionService", "Applying command: DissolveCard");
          coroutines.Add(StartCoroutine(
            Registry.EffectService.HandleDissolveCommand(command.DissolveCard)));
        }

        if (command.DisplayGameMessage != null)
        {
          Registry.LoggingService.LogInfo("ActionService", "Applying command: DisplayGameMessage",
            ("messageType", command.DisplayGameMessage.Value.ToString()));
          coroutines.Add(StartCoroutine(Registry.Layout.GameMessage.Show(command.DisplayGameMessage.Value)));
        }

        if (command.DisplayEffect != null)
        {
          Registry.LoggingService.LogInfo("ActionService", "Applying command: DisplayEffect");
          coroutines.Add(StartCoroutine(Registry.EffectService.HandleDisplayEffectCommand(command.DisplayEffect)));
        }

        if (command.DrawUserCards != null)
        {
          Registry.LoggingService.LogInfo("ActionService", "Applying command: DrawUserCards",
            ("cardCount", command.DrawUserCards.Cards.Count.ToString()));
          coroutines.Add(StartCoroutine(Registry.CardService.HandleDrawUserCards(command.DrawUserCards)));
        }

        if (command.DisplayJudgment != null)
        {
          Registry.LoggingService.LogInfo("ActionService", "Applying command: DisplayJudgment");
          coroutines.Add(StartCoroutine(Registry.JudgmentService.HandleDisplayJudgmentCommand(command.DisplayJudgment)));
        }

        if (command.DisplayDreamwellActivation != null)
        {
          Registry.LoggingService.LogInfo("ActionService", "Applying command: DisplayDreamwellActivation");
          coroutines.Add(StartCoroutine(Registry.DreamwellActivationService.HandleDreamwellActivationCommand(command.DisplayDreamwellActivation)));
        }

        if (command.DisplayEnemyMessage != null)
        {
          Registry.LoggingService.LogInfo("ActionService", "Applying command: DisplayEnemyMessage");
          Registry.Layout.EnemyMessage.Show(command.DisplayEnemyMessage);
        }

        if (command.ToggleThinkingIndicator != null)
        {
          Registry.LoggingService.LogInfo("ActionService", "Applying command: ToggleThinkingIndicator",
            ("show", command.ToggleThinkingIndicator.Show.ToString()));
          Registry.Layout.ThinkingIndicator.SetActive(command.ToggleThinkingIndicator.Show);
        }

        if (command.PlayAudioClip != null)
        {
          Registry.LoggingService.LogInfo("ActionService", "Applying command: PlayAudioClip");
          Registry.SoundService.Play(command.PlayAudioClip.Sound);
          if (command.PlayAudioClip.PauseDuration.MillisecondsValue > 0)
          {
            coroutines.Add(StartCoroutine(Wait(command.PlayAudioClip.PauseDuration)));
          }
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

      return string.Join(", ", commandNames);
    }
  }
}
