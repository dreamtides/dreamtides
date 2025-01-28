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
      var commands = Plugin.GetScene(_scene++);
      Debug.Log($"Starting layout update {commands}");
      var battleView = commands.Groups[0].Commands[0].UpdateBattle;
      StartCoroutine(_registry.LayoutUpdateService.UpdateLayout(battleView));
    }
  }
}
