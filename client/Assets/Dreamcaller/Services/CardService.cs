#nullable enable

using UnityEngine;

namespace Dreamcaller.Services
{
  public class CardService : Service
  {
    readonly RaycastHit[] _raycastHitsTempBuffer = new RaycastHit[8];
    [SerializeField] BoxCollider _playCardArea = null!;

    public bool IsTouchOverPlayCardArea()
    {
      var ray = Registry.MainCamera.ScreenPointToRay(Registry.InputService.TapPosition());
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

    public void ClearInfoZoom() { }
  }
}