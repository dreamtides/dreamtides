#nullable enable

using System.Collections.Generic;
using Dreamcaller.Components;
using UnityEngine;

namespace Dreamcaller.Services
{
  public class EnvironmentService : Service
  {
    [SerializeField] List<GameEnvironment> _environments = null!;

    protected override void OnInitialize()
    {
      var randomIndex = Random.Range(0, _environments.Count);
      _environments[randomIndex].Activate(Registry);
    }
  }
}