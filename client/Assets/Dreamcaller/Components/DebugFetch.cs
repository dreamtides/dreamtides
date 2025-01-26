#nullable enable

using Dreamcaller.Services;
using UnityEngine;

namespace Dreamcaller.Components
{
  public class DebugFetch : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;

    public void OnClick()
    {
      var battleView = Plugin.GetScene(0);
      Debug.Log($"Starting layout update {battleView}");
      StartCoroutine(_registry.LayoutUpdateService.UpdateLayout(battleView));
    }
  }
}
