#nullable enable

using System.Collections;
using System.Collections.Generic;
using Dreamtides.Schema;
using UnityEngine;
using UnityEngine.Networking;
using Newtonsoft.Json;
using System;
using Dreamtides.Utils;

namespace Dreamtides.Services
{
  public class ActionService : Service
  {
    bool _devModeAutoConnect;
    float _lastConnectAttemptTime;
    Metadata? _metadata;
    string? _testScenario;

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

      var request = new PerformActionRequest
      {
        Metadata = _metadata,
        Action = action,
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

      foreach (var group in commands.Groups)
      {
        yield return ApplyGroup(group, animate);
      }

      if (!Connected)
      {
        yield return new WaitForEndOfFrame();
        Connected = true;
      }
    }

    IEnumerator ApplyGroup(ParallelCommandGroup group, bool animate)
    {
      var coroutines = new List<Coroutine>();
      foreach (var command in group.Commands)
      {
        if (command.UpdateBattle != null)
        {
          Registry.Layout.UserStatusDisplay.UpdatePlayerView(command.UpdateBattle.Battle.User, animate);
          Registry.Layout.EnemyStatusDisplay.UpdatePlayerView(command.UpdateBattle.Battle.Enemy, animate);
          Registry.DocumentService.RenderScreenOverlay(command.UpdateBattle.Battle.Interface?.ScreenOverlay);
          Registry.Layout.CardOrderSelector.View = command.UpdateBattle.Battle.Interface?.CardOrderSelector;
          Registry.Layout.PrimaryActionButton.SetView(command.UpdateBattle.Battle.Interface?.PrimaryActionButton);
          Registry.Layout.SecondaryActionButton.SetView(command.UpdateBattle.Battle.Interface?.SecondaryActionButton);
          Registry.Layout.IncrementActionButton.SetView(command.UpdateBattle.Battle.Interface?.IncrementButton);
          Registry.Layout.DecrementActionButton.SetView(command.UpdateBattle.Battle.Interface?.DecrementButton);
          Registry.BottomRightButton.SetView(command.UpdateBattle.Battle.Interface?.BottomRightButton);
          coroutines.Add(StartCoroutine(Registry.LayoutService.UpdateLayout(
              command.UpdateBattle,
              animate ? TweenUtils.Sequence("UpdateLayout") : null)));
        }

        if (command.Wait != null)
        {
          coroutines.Add(StartCoroutine(Wait(command.Wait)));
        }

        if (command.FireProjectile != null)
        {
          coroutines.Add(StartCoroutine(
            Registry.EffectService.HandleFireProjectileCommand(command.FireProjectile)));
        }

        if (command.DissolveCard != null)
        {
          coroutines.Add(StartCoroutine(
            Registry.EffectService.HandleDissolveCommand(command.DissolveCard)));
        }

        if (command.DisplayGameMessage != null)
        {
          coroutines.Add(StartCoroutine(Registry.Layout.GameMessage.Show(command.DisplayGameMessage.Value)));
        }

        if (command.DisplayEffect != null)
        {
          coroutines.Add(StartCoroutine(Registry.EffectService.HandleDisplayEffectCommand(command.DisplayEffect)));
        }

        if (command.DrawUserCards != null)
        {
          coroutines.Add(StartCoroutine(Registry.CardService.HandleDrawUserCards(command.DrawUserCards)));
        }

        if (command.DisplayJudgment != null)
        {
          coroutines.Add(StartCoroutine(Registry.JudgmentService.HandleDisplayJudgmentCommand(command.DisplayJudgment)));
        }

        if (command.DisplayDreamwellActivation != null)
        {
          coroutines.Add(StartCoroutine(Registry.DreamwellActivationService.HandleDreamwellActivationCommand(command.DisplayDreamwellActivation)));
        }

        if (command.DisplayArrows != null)
        {
          Registry.ArrowService.HandleDisplayArrowsCommand(command.DisplayArrows);
        }

        if (command.DisplayEnemyMessage != null)
        {
          Registry.Layout.EnemyMessage.Show(command.DisplayEnemyMessage);
        }

        if (command.ToggleThinkingIndicator != null)
        {
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
