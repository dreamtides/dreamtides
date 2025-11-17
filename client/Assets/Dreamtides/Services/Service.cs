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
    bool _initialized = false;
    GameMode _mode = GameMode.MainMenu;
    TestConfiguration? _testConfiguration = null;

    protected GameMode Mode =>
      _initialized ? _mode : throw new InvalidOperationException($"{name} not initialized!");

    protected TestConfiguration? TestConfiguration =>
      _initialized
        ? _testConfiguration
        : throw new InvalidOperationException($"{name} not initialized!");

    public void Initialize(Registry registry, GameMode mode, TestConfiguration? testConfiguration)
    {
      if (_initialized)
      {
        throw new InvalidOperationException($"{name} already initialized!");
      }

      _initialized = true;
      _registry = registry;
      _mode = mode;
      _testConfiguration = testConfiguration;

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
