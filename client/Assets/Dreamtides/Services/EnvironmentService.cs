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

    protected override void OnInitialize(TestConfiguration? testConfiguration)
    {
      var randomIndex = testConfiguration != null ? 0 : Random.Range(0, _environments.Count);
      _environments[randomIndex].Activate(Registry);
    }
  }
}
