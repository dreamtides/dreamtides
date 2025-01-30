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
    [SerializeField] Displayable? _lastClicked;
    InputAction _clickAction;
    InputAction _tapPositionAction;

    void Start()
    {
      _clickAction = InputSystem.actions.FindAction("Click");
      _tapPositionAction = InputSystem.actions.FindAction("TapPosition");
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
      Debug.Log($"Tapped: '{fired}'");

      if (fired && fired != null)
      {
        fired.MouseDown();
      }

      return fired;
    }

    Displayable? ObjectAtClickPosition()
    {
      var tapScreenPosition = _tapPositionAction.ReadValue<Vector2>();
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
        .FirstOrDefault();
    }
  }
}
