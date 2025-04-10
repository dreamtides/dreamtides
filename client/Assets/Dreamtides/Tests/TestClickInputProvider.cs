#nullable enable

using System.Collections;
using Dreamtides.Services;
using UnityEngine;

namespace Dreamtides.Tests
{
  public class TestClickInputProvider : IInputProvider
  {
    readonly Vector2 _screenPosition;
    readonly float _clickStartTime;
    readonly float _clickEndTime;

    public static IEnumerator ClickOn(Registry registry, Transform target)
    {
      var screenPosition = registry.Layout.MainCamera.WorldToScreenPoint(target.position);
      registry.InputService.InputProvider = new TestClickInputProvider(screenPosition);
      yield return new WaitForSeconds(0.3f);
      yield return TestUtil.WaitForAnimations();
    }

    public TestClickInputProvider(Vector2 screenPosition)
    {
      _screenPosition = screenPosition;
      _clickStartTime = Time.time + 0.1f;
      _clickEndTime = Time.time + 0.2f;
    }

    public bool IsPointerPressed() => Time.time >= _clickStartTime && Time.time <= _clickEndTime;

    public Vector2 PointerPosition() => _screenPosition;
  }
}