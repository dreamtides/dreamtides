#nullable enable

using System;
using UnityEngine;

namespace Dreamtides.Services
{
  public class Service : MonoBehaviour
  {
    Registry? _registry = null;
    public Registry Registry => _registry ??
        throw new InvalidOperationException($"{name} not initialized!");

    public void Initialize(Registry registry, TestConfiguration? testConfiguration)
    {
      _registry = registry;
      OnInitialize(testConfiguration);
    }

    protected virtual void OnInitialize(TestConfiguration? testConfiguration) { }
  }
}
