#nullable enable

using Dreamcaller.Components;
using Dreamcaller.Layout;
using Dreamcaller.Masonry;
using Dreamcaller.Schema;
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
      if (_currentInfoZoom && card.Id == _currentInfoZoom.Id)
      {
        return;
      }

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

      if (_currentInfoZoom.CardView.Revealed?.SupplementalCardInfo is { } info)
      {
        var node = Mason.Row("InfoZoom",
          new FlexStyle
          {
            Position = FlexPosition.Absolute,
            Height = Mason.Px(160),
            JustifyContent = shouldShowOnLeft ? FlexJustify.FlexStart : FlexJustify.FlexEnd,
            AlignItems = FlexAlign.FlexStart,
            Inset = new DimensionGroup
            {
              Left = Mason.Px(0),
              Right = Mason.Px(0),
              Bottom = Mason.Px(200),
            }
          },
          info
        );
        Registry.DocumentService.RenderInfoZoom(node);
      }
    }

    public void ClearInfoZoom()
    {
      Registry.DocumentService.ClearInfoZoom();
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