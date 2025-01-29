#nullable enable

using System.Collections;
using System.Collections.Generic;
using DG.Tweening;
using Dreamcaller.Schema;
using Dreamcaller.Services;
using UnityEditor.Rendering;
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
      StartCoroutine(ApplyCommands(commands));
    }

    IEnumerator ApplyCommands(CommandSequence commands)
    {
      foreach (var group in commands.Groups)
      {
        yield return ApplyGroup(group);
      }
    }

    IEnumerator ApplyGroup(CommandGroup group)
    {
      var coroutines = new List<Coroutine>();
      foreach (var command in group.Commands)
      {
        if (command.UpdateBattle != null)
        {
          coroutines.Add(StartCoroutine(_registry.LayoutUpdateService.UpdateLayout(command.UpdateBattle)));
        }
      }

      foreach (var coroutine in coroutines)
      {
        yield return coroutine;
      }
    }
  }
}
