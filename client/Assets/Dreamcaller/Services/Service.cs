#nullable enable

using System;
using UnityEngine;

namespace Dreamcaller.Services
{
  public class Service : MonoBehaviour
  {
    Registry? _registry = null;
    public Registry Registry => _registry ??
        throw new InvalidOperationException($"{name} not initialized!");

    public void Initialize(Registry registry)
    {
      _registry = registry;
    }
  }
}
