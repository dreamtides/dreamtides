#nullable enable

using System.Linq;
using Dreamcaller.Layout;
using UnityEditor;
using UnityEngine;

namespace Dreamcaller.Editors
{
  [CustomEditor(typeof(StandardObjectLayout), editorForChildClasses: true)]
  public sealed class StandardObjectLayoutEditor : Editor
  {
    public override void OnInspectorGUI()
    {
      DrawDefaultInspector();
      GUILayout.Space(10);
      var display = (StandardObjectLayout)target;

      if (GUILayout.Button("Toggle Update Continuously"))
      {
        display.DebugUpdateContinuously = !display.DebugUpdateContinuously;
      }

      if (GUILayout.Button("Delete First"))
      {
        var card = display.Objects.First();
        display.RemoveIfPresent(card);
        Destroy(card.gameObject);
      }
    }
  }
}