#nullable enable

using System;
using System.Collections.Generic;
using System.Linq;
using Dreamtides.Layout;
using UnityEngine;
using UnityEngine.InputSystem;

namespace Dreamtides.Services
{
  public class InputService : Service
  {
    readonly RaycastHit[] _raycastHitsTempBuffer = new RaycastHit[8];
    Displayable? _lastHovered;
    Displayable? _lastClicked;
    InputAction _clickAction = null!;
    InputAction _tapPositionAction = null!;

    void Start()
    {
      _clickAction = InputSystem.actions.FindAction("Click");
      _tapPositionAction = InputSystem.actions.FindAction("TapPosition");
    }

    public Vector2 PointerPosition() => _tapPositionAction.ReadValue<Vector2>();

    public Vector3 WorldPointerPosition(float screenZ)
    {
      var tapScreenPosition = PointerPosition();
      return Registry.Layout.MainCamera.ScreenToWorldPoint(
          new Vector3(tapScreenPosition.x, tapScreenPosition.y, screenZ));
    }

    void Update()
    {
      HandleDisplayableClickAndDrag();
      HandleDisplayableHover();
    }

    void HandleDisplayableClickAndDrag()
    {
      switch (_clickAction.IsPressed())
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
          var objectAtClickPosition = ObjectAtPointerPosition();
          last.MouseUp(objectAtClickPosition == last);
          break;
      }
    }

    void HandleDisplayableHover()
    {
      if (_clickAction.IsPressed() || !Registry.IsLandscape)
      {
        return;
      }

      var current = ObjectAtPointerPosition();
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
      if (Registry.DocumentService.IsAnyPanelOpen() ||
          Registry.DocumentService.IsPointerOverScreenElement())
      {
        return null;
      }

      var fired = ObjectAtPointerPosition();
      if (fired)
      {
        fired.MouseDown();
      }

      return fired;
    }

    Displayable? ObjectAtPointerPosition()
    {
      var allowedContexts = Registry.DocumentService.AllowedContextForClicks();
      var tapScreenPosition = PointerPosition();
      var ray = Registry.Layout.MainCamera.ScreenPointToRay(tapScreenPosition);
      var hits = Physics.RaycastAll(
          ray,
          maxDistance: 256,
          LayerMask.GetMask("Default"),
          QueryTriggerInteraction.Ignore);

      var candidates = new List<Displayable>();
      for (var i = 0; i < hits.Length; ++i)
      {
        var hit = hits[i];
        var displayable = hit.collider.GetComponent<Displayable>();
        if (displayable && displayable.CanHandleMouseEvents() &&
            (allowedContexts == null || allowedContexts.Contains(displayable.GameContext)))
        {
          candidates.Add(displayable);
        }
      }

      Array.Clear(_raycastHitsTempBuffer, 0, _raycastHitsTempBuffer.Length);
      return candidates
        .OrderBy(c => c.GameContext)
        .ThenBy(c => c.SortingKey)
        .LastOrDefault();
    }
  }
}
