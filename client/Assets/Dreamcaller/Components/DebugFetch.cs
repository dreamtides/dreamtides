#nullable enable

using System;
using System.Collections;
using Dreamcaller.Schema;
using Dreamcaller.Services;
using UnityEngine;

namespace Dreamcaller.Components
{
  public class DebugFetch : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;

    IEnumerator Start()
    {
      yield return new WaitForSeconds(0.5f);
      var request = new ConnectRequest
      {
        Metadata = new Metadata
        {
          UserId = Guid.NewGuid()
        }
      };
      _registry.ActionService.Connect(request);
    }

    public void OnClick()
    {
      _registry.ActionService.PerformAction(new UserAction
      {
        DebugAction = DebugAction.DrawCard
      });
    }
  }
}
