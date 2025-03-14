#nullable enable

using System.Collections;
using System.Collections.Generic;
using Dreamcaller.Schema;
using UnityEngine;
using UnityEngine.Networking;
using Newtonsoft.Json;
using System;
using DG.Tweening;
using Dreamcaller.Utils;

namespace Dreamcaller.Services
{
  public class ActionService : Service
  {
    IEnumerator Start()
    {
      yield return new WaitForEndOfFrame();
      var request = new ConnectRequest
      {
        Metadata = new Metadata
        {
          UserId = Guid.NewGuid()
        }
      };
      if (Application.isEditor)
      {
        StartCoroutine(DevServerConnectAsync(request));
      }
      else
      {
        var response = Plugin.Connect(request);
        StartCoroutine(ApplyCommands(response.Commands, animate: false));
      }
    }

    public void PerformAction(UserAction? action)
    {
      if (action == null)
      {
        return;
      }

      var request = new PerformActionRequest
      {
        Metadata = new Metadata
        {
          UserId = Guid.NewGuid()
        },
        Action = action
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

    private IEnumerator DevServerConnectAsync(ConnectRequest request)
    {
      yield return SendRequest<ConnectRequest, ConnectResponse>(
        request,
        "connect",
        UnityWebRequest.kHttpVerbGET,
        response => ApplyCommands(response.Commands, animate: false));
    }

    private IEnumerator PerformDevServerActionAsync(PerformActionRequest request)
    {
      yield return SendRequest<PerformActionRequest, PerformActionResponse>(
        request,
        "perform_action",
        UnityWebRequest.kHttpVerbPOST,
        response => ApplyCommands(response.Commands, animate: true));
    }

    private IEnumerator SendRequest<TRequest, TResponse>(
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
        var responseJson = webRequest.downloadHandler.text;
        var response = JsonConvert.DeserializeObject<TResponse>(responseJson, Converter.Settings);
        if (response != null)
        {
          yield return handleResponse(response);
        }
      }
      else
      {
        Debug.LogError($"{endpoint} request failed: {webRequest.error}");
      }
    }

    IEnumerator ApplyCommands(CommandSequence commands, bool animate)
    {
      foreach (var group in commands.Groups)
      {
        yield return ApplyGroup(group, animate);
      }
    }

    IEnumerator ApplyGroup(ParallelCommandGroup group, bool animate)
    {
      var coroutines = new List<Coroutine>();
      var sequence = animate ? TweenUtils.Sequence("ApplyGroup") : null;
      foreach (var command in group.Commands)
      {
        if (command.UpdateBattle != null)
        {
          Registry.Layout.UserStatusDisplay.UpdatePlayerView(command.UpdateBattle.Battle.User, animate);
          Registry.Layout.EnemyStatusDisplay.UpdatePlayerView(command.UpdateBattle.Battle.Enemy, animate);
          Registry.DocumentService.RenderScreenOverlay(command.UpdateBattle.Battle.Interface?.ScreenOverlay);
          coroutines.Add(StartCoroutine(Registry.LayoutService.UpdateLayout(command.UpdateBattle, sequence)));
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
