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

    public void Initialize(Registry registry)
    {
      _registry = registry;
      OnInitialize();
    }

    protected virtual void OnInitialize() { }
  }
}
