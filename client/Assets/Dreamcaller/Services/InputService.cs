#nullable enable

using System;
using System.Collections.Generic;
using System.Linq;
using Dreamcaller.Layout;
using UnityEngine;
using UnityEngine.InputSystem;

namespace Dreamcaller.Services
{
  public class InputService : Service
  {
    readonly RaycastHit[] _raycastHitsTempBuffer = new RaycastHit[8];
    Displayable? _lastClicked;
    InputAction _clickAction = null!;
    InputAction _tapPositionAction = null!;

    void Start()
    {
      _clickAction = InputSystem.actions.FindAction("Click");
      _tapPositionAction = InputSystem.actions.FindAction("TapPosition");
    }

    public Vector2 TapPosition() => _tapPositionAction.ReadValue<Vector2>();

    public Vector3 WorldMousePosition(float screenZ)
    {
      var tapScreenPosition = TapPosition();
      return Registry.MainCamera.ScreenToWorldPoint(
          new Vector3(tapScreenPosition.x, tapScreenPosition.y, screenZ));
    }

    void Update()
    {
      HandleDisplayableClickAndDrag();
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
          last.MouseUp();
          break;
      }
    }

    Displayable? FireClick()
    {
      if (Registry.DocumentService.IsAnyPanelOpen() ||
          Registry.DocumentService.MouseOverScreenElement())
      {
        return null;
      }

      var fired = ObjectAtClickPosition();

      if (fired && fired != null)
      {
        fired.MouseDown();
      }

      return fired;
    }

    Displayable? ObjectAtClickPosition()
    {
      var tapScreenPosition = TapPosition();
      var ray = Registry.MainCamera.ScreenPointToRay(tapScreenPosition);
      var hits = Physics.RaycastNonAlloc(ray, _raycastHitsTempBuffer, 100);

      var candidates = new List<Displayable>();
      for (var i = 0; i < hits; ++i)
      {
        var hit = _raycastHitsTempBuffer[i];
        var displayable = hit.collider.GetComponent<Displayable>();
        if (displayable && displayable.CanHandleMouseEvents())
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
