#nullable enable

using System.Collections;
using Dreamtides.Layout;
using Dreamtides.Services;
using UnityEngine;

namespace Dreamtides.Tests
{
  public class TestDragInputProvider : IInputProvider
  {
    const float _dragDuration = 0.3f;
    readonly Displayable _target;
    readonly Vector2 _startPosition;
    readonly Vector2 _endPosition;
    readonly float _dragStartTime;
    readonly float _dragEndTime;

    public static IEnumerator DragTo(Registry registry, Displayable source, Displayable target)
    {
      var startPosition = registry.Layout.MainCamera.WorldToScreenPoint(source.transform.position);
      var endPosition = registry.Layout.MainCamera.WorldToScreenPoint(target.transform.position);
      registry.InputService.InputProvider = new TestDragInputProvider(source, startPosition, endPosition);
      yield return new WaitForSeconds(0.1f + _dragDuration);
      yield return registry.TestHelperService.WaitForIdle();
    }

    public TestDragInputProvider(Displayable target, Vector2 startPosition, Vector2 endPosition)
    {
      _target = target;
      _startPosition = startPosition;
      _endPosition = endPosition;
      _dragStartTime = Time.time + 0.1f;
      _dragEndTime = Time.time + 0.1f + _dragDuration;
    }

    public bool IsPointerPressed() => Time.time >= _dragStartTime && Time.time <= _dragEndTime;

    public Vector2 PointerPosition()
    {
      if (Time.time < _dragStartTime)
      {
        return _startPosition;
      }
      else if (Time.time > _dragEndTime)
      {
        return _endPosition;
      }
      else
      {
        var t = (Time.time - _dragStartTime) / (_dragEndTime - _dragStartTime);
        return Vector2.Lerp(_startPosition, _endPosition, t);
      }
    }

    public Displayable? ObjectAtPointerPosition() => _target;
  }
}