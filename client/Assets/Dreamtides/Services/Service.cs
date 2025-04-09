#nullable enable

using System;
using UnityEngine;

namespace Dreamtides.Services
{
  public class Service : MonoBehaviour
  {
    public enum TestMode
    {
      None,
      Testing,
      Recording
    }

    Registry? _registry = null;
    public Registry Registry => _registry ??
        throw new InvalidOperationException($"{name} not initialized!");

    public void Initialize(Registry registry, TestMode testMode)
    {
      _registry = registry;
      OnInitialize(testMode);
    }

    protected virtual void OnInitialize(TestMode testMode) { }
  }
}
