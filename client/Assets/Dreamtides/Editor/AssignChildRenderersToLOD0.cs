#if UNITY_EDITOR

#nullable enable

using System.Collections.Generic;
using UnityEditor;
using UnityEngine;

namespace Dreamtides.Editors
{
  public static class AssignChildRenderersToLOD0
  {
    [MenuItem("Tools/Assign Child Renderers to LOD0")]
    public static void AssignRenderersToLOD0()
    {
      var selected = Selection.activeGameObject;
      if (selected == null)
      {
        EditorUtility.DisplayDialog(
          "Assign Child Renderers to LOD0",
          "Please select a GameObject with an LOD Group component.",
          "OK"
        );
        return;
      }

      var lodGroup = selected.GetComponent<LODGroup>();
      if (lodGroup == null)
      {
        EditorUtility.DisplayDialog(
          "Assign Child Renderers to LOD0",
          "Selected GameObject must have an LOD Group component.",
          "OK"
        );
        return;
      }

      var childRenderers = CollectChildRenderers(selected.transform);
      if (childRenderers.Count == 0)
      {
        EditorUtility.DisplayDialog(
          "Assign Child Renderers to LOD0",
          "No child renderers found.",
          "OK"
        );
        return;
      }

      var undoGroupIndex = Undo.GetCurrentGroup();
      Undo.SetCurrentGroupName("Assign Child Renderers to LOD0");
      Undo.RecordObject(lodGroup, "Assign Child Renderers to LOD0");

      var lods = lodGroup.GetLODs();
      if (lods.Length == 0)
      {
        var newLOD = new LOD(0.6f, childRenderers.ToArray());
        lodGroup.SetLODs(new LOD[] { newLOD });
      }
      else
      {
        lods[0].renderers = childRenderers.ToArray();
        lodGroup.SetLODs(lods);
      }

      lodGroup.RecalculateBounds();

      Undo.CollapseUndoOperations(undoGroupIndex);
      EditorUtility.SetDirty(lodGroup);
      Debug.Log($"Assigned {childRenderers.Count} child renderers to LOD0 on {selected.name}.");
    }

    [MenuItem("Tools/Assign Child Renderers to LOD0", true)]
    private static bool ValidateAssignRenderersToLOD0()
    {
      var selected = Selection.activeGameObject;
      return selected != null && selected.GetComponent<LODGroup>() != null;
    }

    private static List<Renderer> CollectChildRenderers(Transform root)
    {
      var renderers = new List<Renderer>();
      CollectChildRenderersRecursive(root, renderers);
      return renderers;
    }

    private static void CollectChildRenderersRecursive(Transform current, List<Renderer> renderers)
    {
      foreach (Transform child in current)
      {
        var renderer = child.GetComponent<Renderer>();
        if (renderer != null)
        {
          renderers.Add(renderer);
        }

        CollectChildRenderersRecursive(child, renderers);
      }
    }
  }
}
#endif
