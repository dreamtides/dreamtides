#nullable enable

using System;
using Dreamcaller.Layout;
using Dreamcaller.Utils;
using UnityEngine;

namespace Dreamcaller.Services
{
  public class Registry : MonoBehaviour
  {
    [SerializeField] ObjectLayout? _userHand;
    public ObjectLayout UserHand => Check(_userHand);

    [SerializeField] ObjectLayout? _offscreen;
    public ObjectLayout Offscreen => Check(_offscreen);

    [SerializeField] LayoutUpdateService? _layoutUpdateService;
    public LayoutUpdateService LayoutUpdateService => Check(_layoutUpdateService);

    void Start()
    {
      foreach (var service in GetComponentsInChildren<Service>())
      {
        service.Initialize(this);
      }
    }

    T Check<T>(T? value) where T : UnityEngine.Object =>
        Errors.CheckNotNull(value, $"{typeof(T).Name} not initialized");
  }
}