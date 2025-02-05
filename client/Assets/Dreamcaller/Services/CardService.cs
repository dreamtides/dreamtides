#nullable enable

using Dreamcaller.Components;
using Dreamcaller.Layout;
using UnityEngine;

namespace Dreamcaller.Services
{
  public class CardService : Service
  {
    readonly RaycastHit[] _raycastHitsTempBuffer = new RaycastHit[8];
    [SerializeField] MeshCollider _playCardArea = null!;
    [SerializeField] ObjectLayout _infoZoomLeft = null!;
    [SerializeField] ObjectLayout _infoZoomRight = null!;
    Card? _currentInfoZoom;

    public bool IsPointerOverPlayCardArea()
    {
      var ray = Registry.MainCamera.ScreenPointToRay(Registry.InputService.PointerPosition());
      var hits = Physics.RaycastNonAlloc(ray, _raycastHitsTempBuffer, 100);
      for (var i = 0; i < hits; ++i)
      {
        var hit = _raycastHitsTempBuffer[i];
        if (hit.collider == _playCardArea)
        {
          return true;
        }
      }

      return false;
    }

    /// <summary>
    /// Displays a large format version of the provided card in the info zoom.
    /// </summary>
    public void DisplayInfoZoom(Card card)
    {
      if (!_currentInfoZoom || card.Id != _currentInfoZoom.Id)
      {
        ClearInfoZoom();
        var shouldShowOnLeft = Registry.InputService.PointerPosition().x > Screen.width / 2.0;
        _currentInfoZoom = card.CloneForInfoZoom();
        if (shouldShowOnLeft)
        {
          _infoZoomLeft.Add(_currentInfoZoom);
          _infoZoomLeft.ApplyLayout();
        }
        else
        {
          _infoZoomRight.Add(_currentInfoZoom);
          _infoZoomRight.ApplyLayout();
        }
      }
    }

    public void ClearInfoZoom()
    {
      if (_currentInfoZoom)
      {
        if (_currentInfoZoom.Parent)
        {
          _currentInfoZoom.Parent.RemoveIfPresent(_currentInfoZoom);
        }
        Destroy(_currentInfoZoom.gameObject);
      }
      _currentInfoZoom = null;
    }
  }
}