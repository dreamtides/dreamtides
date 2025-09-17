#nullable enable

using System.Collections;
using Dreamtides.Layout;
using Dreamtides.Services;
using UnityEngine;

namespace Dreamtides.TestUtils
{
  public class TestClickInputProvider : IInputProvider
  {
    readonly Displayable _target;
    readonly Vector2 _screenPosition;
    readonly float _clickStartTime;
    readonly float _clickEndTime;

    public static IEnumerator ClickOn(Registry registry, Displayable target)
    {
      var screenPosition = registry.Layout.MainCamera.WorldToScreenPoint(target.transform.position);
      registry.InputService.InputProvider = new TestClickInputProvider(target, screenPosition);
      yield return new WaitForSeconds(0.2f);
      yield return registry.TestHelperService.WaitForIdle(IntegrationTest.TimeoutSeconds);
    }

    public TestClickInputProvider(Displayable target, Vector2 screenPosition)
    {
      _target = target;
      _screenPosition = screenPosition;
      _clickStartTime = Time.time + 0.1f;
      _clickEndTime = Time.time + 0.2f;
    }

    public bool IsPointerPressed() => Time.time >= _clickStartTime && Time.time <= _clickEndTime;

    public Vector2 PointerPosition() => _screenPosition;

    public Displayable? ObjectAtPointerPosition() => _target;
  }
}
