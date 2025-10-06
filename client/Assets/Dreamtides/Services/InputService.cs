#nullable enable

using System;
using System.Collections.Generic;
using System.Linq;
using Dreamtides.Layout;
using UnityEngine;
using UnityEngine.InputSystem;

namespace Dreamtides.Services
{
  public interface IInputProvider
  {
    /// <summary>
    /// Returns true if the mouse, pointing device, or a finger is currently
    /// pressed down.
    /// </summary>
    bool IsPointerPressed();

    /// <summary>
    /// Returns the current or last-known screen position of the mouse, pointing
    /// device, or finger contacting the touch screen in screen coordinates.
    /// </summary>
    Vector2 PointerPosition();

    Displayable? ObjectAtPointerPosition();
  }

  public class UnityInputProvider : IInputProvider
  {
    readonly RaycastHit[] _raycastHitsTempBuffer = new RaycastHit[8];
    readonly Registry _registry;
    readonly InputAction _clickAction;
    readonly InputAction _tapPositionAction;

    public UnityInputProvider(Registry registry)
    {
      _registry = registry;
      _clickAction = InputSystem.actions.FindAction("Click");
      _tapPositionAction = InputSystem.actions.FindAction("TapPosition");
      _clickAction.Enable();
      _tapPositionAction.Enable();
    }

    // Note: In the Unity 6 editor, there is a bug where this will sometimes start
    // returning 'false' if you toggle the game window to device simulator mode.
    // The only reliable way to prevent this is to always use a separate Unity
    // window for device simulation.
    public bool IsPointerPressed() => _clickAction.IsPressed();

    public Displayable? ObjectAtPointerPosition()
    {
      var allowedContexts = _registry.DocumentService.AllowedContextForClicks();
      var tapScreenPosition = PointerPosition();
      var ray = _registry.Layout.MainCamera.ScreenPointToRay(tapScreenPosition);
      var hits = Physics.RaycastAll(
        ray,
        maxDistance: 256,
        LayerMask.GetMask("Default"),
        QueryTriggerInteraction.Ignore
      );

      var candidates = new List<Displayable>();
      for (var i = 0; i < hits.Length; ++i)
      {
        var hit = hits[i];
        var displayable = hit.collider.GetComponent<Displayable>();
        if (
          displayable
          && displayable.CanHandleMouseEvents()
          && (allowedContexts == null || allowedContexts.Contains(displayable.GameContext))
        )
        {
          candidates.Add(displayable);
        }
      }

      Array.Clear(_raycastHitsTempBuffer, 0, _raycastHitsTempBuffer.Length);
      return candidates.OrderBy(c => c.GameContext).ThenBy(c => c.SortingKey).LastOrDefault();
    }

    public Vector2 PointerPosition() => _tapPositionAction.ReadValue<Vector2>();
  }

  public class InputService : Service
  {
    Displayable? _lastHovered;
    Displayable? _lastClicked;

    public IInputProvider InputProvider { get; set; } = null!;

    protected override void OnInitialize(GameMode _mode, TestConfiguration? testConfiguration)
    {
      InputProvider = new UnityInputProvider(Registry);
    }

    /// <summary>
    /// Returns the current or last-known screen position of the mouse, pointing
    /// device, or finger contacting the touch screen in screen coordinates.
    /// </summary>
    public Vector2 PointerPosition() => InputProvider.PointerPosition();

    /// <summary>
    /// Returns true if the mouse, pointing device, or a finger is currently
    /// pressed down.
    /// </summary>
    public bool IsPointerPressed() => InputProvider.IsPointerPressed();

    public Vector3 WorldPointerPosition(float screenZ)
    {
      var tapScreenPosition = InputProvider.PointerPosition();
      return Registry.Layout.MainCamera.ScreenToWorldPoint(
        new Vector3(tapScreenPosition.x, tapScreenPosition.y, screenZ)
      );
    }

    protected override void OnUpdate()
    {
      HandleDisplayableClickAndDrag();
      HandleDisplayableHover();
    }

    void HandleDisplayableClickAndDrag()
    {
      switch (InputProvider.IsPointerPressed())
      {
        case true when _lastClicked:
          _lastClicked.MouseDrag();
          break;
        case true when !_lastClicked:
          _lastClicked = FireClick();
          break;
        case false when _lastClicked:
          var last = _lastClicked;
          _lastClicked = null;
          var objectAtClickPosition = InputProvider.ObjectAtPointerPosition();
          last.MouseUp(objectAtClickPosition == last);
          break;
      }
    }

    void HandleDisplayableHover()
    {
      if (InputProvider.IsPointerPressed() || UnityEngine.Device.Application.isMobilePlatform)
      {
        return;
      }

      var current = InputProvider.ObjectAtPointerPosition();
      if (current && !_lastHovered)
      {
        current.MouseHoverStart();
        _lastHovered = current;
      }
      else if (!current && _lastHovered)
      {
        _lastHovered.MouseHoverEnd();
        _lastHovered = null;
      }
      else if (current && _lastHovered && current != _lastHovered)
      {
        _lastHovered.MouseHoverEnd();
        current.MouseHoverStart();
        _lastHovered = current;
      }
      else if (current && current == _lastHovered)
      {
        current.MouseHover();
      }
    }

    Displayable? FireClick()
    {
      if (
        Registry.DocumentService.MouseOverDocumentElement()
        || Registry.DocumentService.HasOpenPanels
      )
      {
        return null;
      }

      var fired = InputProvider.ObjectAtPointerPosition();
      if (fired)
      {
        fired.MouseDown();
      }

      return fired;
    }
  }
}
