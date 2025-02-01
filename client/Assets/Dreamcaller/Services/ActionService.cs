#nullable enable

using System.Collections;
using System.Collections.Generic;
using Dreamcaller.Schema;
using UnityEngine;
using UnityEngine.Networking;
using Newtonsoft.Json;
using System;

namespace Dreamcaller.Services
{
  public class ActionService : Service
  {
    public void Connect(ConnectRequest request)
    {
      if (Application.isEditor)
      {
        StartCoroutine(DevServerConnectCoroutine(request));
      }
      else
      {
        var response = Plugin.Connect(request);
        StartCoroutine(ApplyCommands(response.Commands));
      }
    }

    public void PerformAction(UserAction action)
    {
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
        StartCoroutine(PerformDevServerActionCoroutine(request));
      }
      else
      {
        var response = Plugin.PerformAction(request);
        StartCoroutine(ApplyCommands(response.Commands));
      }
    }

    private IEnumerator DevServerConnectCoroutine(ConnectRequest request)
    {
      yield return SendRequest<ConnectRequest, ConnectResponse>(
        request,
        "connect",
        UnityWebRequest.kHttpVerbGET,
        response => ApplyCommands(response.Commands));
    }

    private IEnumerator PerformDevServerActionCoroutine(PerformActionRequest request)
    {
      yield return SendRequest<PerformActionRequest, PerformActionResponse>(
        request,
        "perform_action",
        UnityWebRequest.kHttpVerbPOST,
        response => ApplyCommands(response.Commands));
    }

    private IEnumerator SendRequest<TRequest, TResponse>(
      TRequest request,
      string endpoint,
      string method,
      System.Func<TResponse, IEnumerator> handleResponse)
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

    IEnumerator ApplyCommands(CommandSequence commands)
    {
      foreach (var group in commands.Groups)
      {
        yield return ApplyGroup(group);
      }
    }

    IEnumerator ApplyGroup(CommandGroup group)
    {
      var coroutines = new List<Coroutine>();
      foreach (var command in group.Commands)
      {
        if (command.UpdateBattle != null)
        {
          coroutines.Add(StartCoroutine(Registry.LayoutUpdateService.UpdateLayout(command.UpdateBattle)));
        }
      }

      foreach (var coroutine in coroutines)
      {
        yield return coroutine;
      }
    }
  }
}
