#nullable enable

using System;
using UnityEngine;

namespace Dreamtides.Services
{
  public class Service : MonoBehaviour
  {
    Registry? _registry = null;
    public Registry Registry =>
      _registry ?? throw new InvalidOperationException($"{name} not initialized!");

    public void Initialize(Registry registry, GameMode mode, TestConfiguration? testConfiguration)
    {
      _registry = registry;
      OnInitialize(mode, testConfiguration);
    }

    protected virtual void OnInitialize(GameMode mode, TestConfiguration? testConfiguration) { }

    public void Update()
    {
      if (_registry != null)
      {
        OnUpdate();
      }
    }

    protected virtual void OnUpdate() { }
  }
}
