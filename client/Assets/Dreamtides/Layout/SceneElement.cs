#nullable enable

using System;
using Dreamtides.Services;
using UnityEngine;

namespace Dreamtides.Layout
{
  public abstract class SceneElement : MonoBehaviour
  {
    [SerializeField]
    Registry _registry = null!;

    GameMode _mode = GameMode.Quest;
    TestConfiguration? _testConfiguration;

    public Registry Registry =>
      _registry ?? throw new InvalidOperationException($"{name} not initialized!");

    public void Initialize(Registry registry, GameMode mode, TestConfiguration? testConfiguration)
    {
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
        OnUpdate(_mode, _testConfiguration);
      }
    }

    protected virtual void OnUpdate(GameMode mode, TestConfiguration? testConfiguration) { }

#if UNITY_EDITOR
    void Reset()
    {
      var registries = FindObjectsByType<Registry>(
        FindObjectsInactive.Include,
        FindObjectsSortMode.None
      );
      var currentScene = gameObject.scene;
      var count = 0;
      var found = (Registry)null!;
      foreach (var r in registries)
      {
        if (r.gameObject.scene == currentScene)
        {
          count++;
          found = r;
          if (count > 1)
          {
            break;
          }
        }
      }
      if (count == 1)
      {
        _registry = found;
        return;
      }
      if (count == 0)
      {
        throw new InvalidOperationException("No Registry found in this scene.");
      }
      throw new InvalidOperationException("Multiple Registry components found in this scene.");
    }
#endif
  }
}
