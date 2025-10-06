#nullable enable

using System.Collections.Generic;
using Dreamtides.Components;
using UnityEngine;

namespace Dreamtides.Services
{
  public class EnvironmentService : Service
  {
    [SerializeField]
    List<GameEnvironment> _environments = null!;

    protected override void OnInitialize(GameMode mode, TestConfiguration? testConfiguration)
    {
      if (mode == GameMode.Battle)
      {
        var randomIndex = testConfiguration != null ? 0 : Random.Range(0, _environments.Count);
        Debug.Log("Activating environment index: " + randomIndex);
        _environments[randomIndex].Activate(Registry);
      }
    }
  }
}
