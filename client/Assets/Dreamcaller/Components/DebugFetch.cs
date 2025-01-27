#nullable enable

using Dreamcaller.Services;
using UnityEngine;

namespace Dreamcaller.Components
{
  public class DebugFetch : MonoBehaviour
  {
    int _scene;
    [SerializeField] Registry _registry = null!;

    public void OnClick()
    {
      var battleView = Plugin.GetScene(_scene++);
      Debug.Log($"Starting layout update {battleView}");
      StartCoroutine(_registry.LayoutUpdateService.UpdateLayout(battleView));
    }
  }
}
