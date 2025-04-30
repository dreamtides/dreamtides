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

    bool _devModeAutoConnect;
    float _lastConnectAttemptTime;
    Metadata? _metadata;
    string? _testScenario;

    private Queue<CommandBatch> _commandQueue = new Queue<CommandBatch>();
    private bool _isProcessingCommands = false;

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
        UserId = Application.dataPath.Contains("test_client") ?
            Guid.Parse("25e89dde-37d7-464b-8a1c-f985102ca029") :
            Guid.Parse("d2da9785-f20e-4879-bed5-35b2e1926faf")
      };
      var request = new ConnectRequest
      {
        Metadata = _metadata,
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
            Metadata = _metadata!
          }, reconnect: true));
          _lastConnectAttemptTime = now;
        }
      }
      else if (_metadata != null)
      {
        StartCoroutine(DevServerPollAsync(new PollRequest
        {
          Metadata = _metadata!
        }));
      }
    }

    public void PerformAction(GameAction? action)
    {
      if (action == null)
      {
        return;
      }

      LogUtils.Log("ActionService", $"Performing action: {action}");

      var request = new PerformActionRequest
      {
        Metadata = _metadata,
        Action = action.Value,
        TestScenario = _testScenario
      };
      if (Application.isEditor)
      {
        StartCoroutine(PerformDevServerActionAsync(request));
      }
      else
      {
        var response = Plugin.PerformAction(request);
        StartCoroutine(ApplyCommands(response.Commands, animate: true));
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
        response => ApplyCommands(response.Commands, animate: true)
      );
    }

    private IEnumerator PerformDevServerActionAsync(PerformActionRequest request)
    {
      yield return SendDevServerRequest<PerformActionRequest, PerformActionResponse>(
        request,
        "perform_action",
        UnityWebRequest.kHttpVerbPOST,
        response => ApplyCommands(response.Commands, animate: true));
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
        LogUtils.Log("ActionService", "Queueing commands for later processing");
        _commandQueue.Enqueue(new CommandBatch { Commands = commands, Animate = animate });
        yield break;
      }

      _isProcessingCommands = true;

      var count = commands.Groups.Sum(group => group.Commands.Count);
      if (count > 1)
      {
        LogUtils.Log("ActionService", $"Applying {count} commands");
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

      _isProcessingCommands = false;

      if (_commandQueue.Count > 0)
      {
        LogUtils.Log("ActionService", $"Processing {_commandQueue.Count} queued command sequences");
        var nextBatch = _commandQueue.Dequeue();
        StartCoroutine(ApplyCommands(nextBatch.Commands, nextBatch.Animate));
      }
    }

    IEnumerator ApplyGroup(ParallelCommandGroup group, bool animate)
    {
      if (group.Commands.Count > 1)
      {
        LogUtils.Log("ActionService", $"Applying group with {group.Commands.Count} commands");
      }
      var coroutines = new List<Coroutine>();
      foreach (var command in group.Commands)
      {
        if (command.UpdateBattle != null)
        {
          LogUtils.Log("ActionService", "Applying command: UpdateBattle");
          Registry.Layout.UserStatusDisplay.UpdatePlayerView(command.UpdateBattle.Battle.User, animate);
          Registry.Layout.EnemyStatusDisplay.UpdatePlayerView(command.UpdateBattle.Battle.Enemy, animate);
          Registry.DocumentService.RenderScreenOverlay(command.UpdateBattle.Battle.Interface?.ScreenOverlay);
          Registry.Layout.CardOrderSelector.View = command.UpdateBattle.Battle.Interface?.CardOrderSelector;
          Registry.Layout.PrimaryActionButton.SetView(
              command.UpdateBattle.Battle.Interface?.PrimaryActionButton,
              command.UpdateBattle.Battle.Interface?.PrimaryActionShowOnIdleDuration);
          Registry.Layout.SecondaryActionButton.SetView(command.UpdateBattle.Battle.Interface?.SecondaryActionButton);
          Registry.Layout.IncrementActionButton.SetView(command.UpdateBattle.Battle.Interface?.IncrementButton);
          Registry.Layout.DecrementActionButton.SetView(command.UpdateBattle.Battle.Interface?.DecrementButton);
          Registry.Layout.DevButton.SetView(command.UpdateBattle.Battle.Interface?.DevButton);
          Registry.BottomRightButton.SetView(command.UpdateBattle.Battle.Interface?.BottomRightButton);
          coroutines.Add(StartCoroutine(Registry.LayoutService.UpdateLayout(
              command.UpdateBattle,
              animate ? TweenUtils.Sequence("UpdateLayout") : null)));
        }

        if (command.Wait != null)
        {
          LogUtils.Log("ActionService", "Applying command: Wait");
          coroutines.Add(StartCoroutine(Wait(command.Wait)));
        }

        if (command.FireProjectile != null)
        {
          LogUtils.Log("ActionService", "Applying command: FireProjectile");
          coroutines.Add(StartCoroutine(
            Registry.EffectService.HandleFireProjectileCommand(command.FireProjectile)));
        }

        if (command.DissolveCard != null)
        {
          LogUtils.Log("ActionService", "Applying command: DissolveCard");
          coroutines.Add(StartCoroutine(
            Registry.EffectService.HandleDissolveCommand(command.DissolveCard)));
        }

        if (command.DisplayGameMessage != null)
        {
          LogUtils.Log("ActionService", "Applying command: DisplayGameMessage");
          coroutines.Add(StartCoroutine(Registry.Layout.GameMessage.Show(command.DisplayGameMessage.Value)));
        }

        if (command.DisplayEffect != null)
        {
          LogUtils.Log("ActionService", "Applying command: DisplayEffect");
          coroutines.Add(StartCoroutine(Registry.EffectService.HandleDisplayEffectCommand(command.DisplayEffect)));
        }

        if (command.DrawUserCards != null)
        {
          LogUtils.Log("ActionService", "Applying command: DrawUserCards");
          coroutines.Add(StartCoroutine(Registry.CardService.HandleDrawUserCards(command.DrawUserCards)));
        }

        if (command.DisplayJudgment != null)
        {
          LogUtils.Log("ActionService", "Applying command: DisplayJudgment");
          coroutines.Add(StartCoroutine(Registry.JudgmentService.HandleDisplayJudgmentCommand(command.DisplayJudgment)));
        }

        if (command.DisplayDreamwellActivation != null)
        {
          LogUtils.Log("ActionService", "Applying command: DisplayDreamwellActivation");
          coroutines.Add(StartCoroutine(Registry.DreamwellActivationService.HandleDreamwellActivationCommand(command.DisplayDreamwellActivation)));
        }

        if (command.DisplayArrows != null)
        {
          LogUtils.Log("ActionService", "Applying command: DisplayArrows");
          Registry.ArrowService.HandleDisplayArrowsCommand(command.DisplayArrows);
        }

        if (command.DisplayEnemyMessage != null)
        {
          LogUtils.Log("ActionService", "Applying command: DisplayEnemyMessage");
          Registry.Layout.EnemyMessage.Show(command.DisplayEnemyMessage);
        }

        if (command.ToggleThinkingIndicator != null)
        {
          LogUtils.Log("ActionService", "Applying command: ToggleThinkingIndicator");
          Registry.Layout.ThinkingIndicator.SetActive(command.ToggleThinkingIndicator.Show);
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
  }
}
