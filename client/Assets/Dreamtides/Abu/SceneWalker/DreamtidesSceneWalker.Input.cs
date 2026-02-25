#nullable enable

using Abu;
using Dreamtides.Buttons;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Masonry;
using Dreamtides.Services;
using UnityEngine;
using UnityEngine.UIElements;

namespace Dreamtides.Abu
{
  public partial class DreamtidesSceneWalker
  {
    // ── Displayable callbacks ─────────────────────────────────────────

    RefCallbacks BuildDisplayableCallbacks(Displayable displayable)
    {
      var callbacks = new RefCallbacks();
      callbacks.OnClick = () =>
      {
        var originalProvider = _registry.InputService.InputProvider;
        var fakeInput = new DisplayableClickInputProvider(displayable);
        try
        {
          _registry.InputService.InputProvider = fakeInput;
          fakeInput.Phase = ClickPhase.Pressed;
          _registry.InputService.Update();
          fakeInput.Phase = ClickPhase.Released;
          _registry.InputService.Update();
        }
        finally
        {
          _registry.InputService.InputProvider = originalProvider;
        }
      };
      callbacks.OnHover = () =>
      {
        displayable.MouseHoverStart();
      };
      return callbacks;
    }

    static RefCallbacks BuildCanvasButtonCallbacks(CanvasButton button)
    {
      return new RefCallbacks
      {
        OnClick = () => button.OnClick(),
        OnHover = () => button.MouseHoverStart(),
      };
    }

    RefCallbacks BuildUiToolkitCallbacks(VisualElement element)
    {
      var callbacks = new RefCallbacks();
      if (element is INodeCallbacks nodeCallbacks)
      {
        var cb = nodeCallbacks.Callbacks.Value;
        callbacks.OnClick = () =>
        {
          using var clickEvent = ClickEvent.GetPooled();
          cb.OnClick(clickEvent);
        };
        callbacks.OnHover = () =>
        {
          using var enterEvent = MouseEnterEvent.GetPooled();
          cb.OnMouseEnter(enterEvent);
        };
      }

      return callbacks;
    }

    // ── Input simulation ──────────────────────────────────────────────

    enum ClickPhase
    {
      Pressed,
      Released,
    }

    sealed class DisplayableClickInputProvider : IInputProvider
    {
      readonly Displayable _target;

      public ClickPhase Phase { get; set; } = ClickPhase.Pressed;

      public DisplayableClickInputProvider(Displayable target)
      {
        _target = target;
      }

      public bool IsMobilePlatform => false;

      public bool IsPointerPressed() => Phase == ClickPhase.Pressed;

      public Vector2 PointerPosition() => Vector2.zero;

      public Displayable? ObjectAtPointerPosition(MouseEventType eventType)
      {
        return eventType switch
        {
          MouseEventType.MouseDown => _target,
          MouseEventType.MouseUp => _target,
          MouseEventType.MouseHover => null,
          _ => null,
        };
      }
    }
  }
}
