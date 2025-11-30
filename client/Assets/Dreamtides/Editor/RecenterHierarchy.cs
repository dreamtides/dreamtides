#if UNITY_EDITOR

#nullable enable

using System.Collections.Generic;
using UnityEditor;
using UnityEngine;

namespace Dreamtides.Editors
{
  public static class RecenterHierarchy
  {
    [MenuItem("Tools/Recenter Selected Prefab Hierarchy")]
    public static void RecenterSelectedPrefab()
    {
      var selected = Selection.activeGameObject;
      if (selected == null)
      {
        EditorUtility.DisplayDialog(
          "Recenter Hierarchy",
          "Please select a GameObject to recenter.",
          "OK"
        );
        return;
      }

      var prefabStage = UnityEditor.SceneManagement.PrefabStageUtility.GetCurrentPrefabStage();
      if (prefabStage == null)
      {
        EditorUtility.DisplayDialog(
          "Recenter Hierarchy",
          "Please open the prefab in Prefab Mode before recentering.",
          "OK"
        );
        return;
      }

      var root = prefabStage.prefabContentsRoot;
      if (!selected.transform.IsChildOf(root.transform) && selected != root)
      {
        EditorUtility.DisplayDialog(
          "Recenter Hierarchy",
          "Selected object must be part of the prefab being edited.",
          "OK"
        );
        return;
      }

      var undoGroupIndex = Undo.GetCurrentGroup();
      Undo.SetCurrentGroupName("Recenter Hierarchy");
      Undo.RegisterFullObjectHierarchyUndo(root, "Recenter Hierarchy");

      var recenterRoot = selected.transform;
      var center = CalculateBoundsCenter(recenterRoot, out var debugInfo);

      Debug.Log($"[Recenter Debug] {debugInfo}");
      Debug.Log(
        $"[Recenter Debug] Computed center offset: ({center.x:F2}, {center.y:F2}, {center.z:F2})"
      );
      Debug.Log(
        $"[Recenter Debug] Root position: {recenterRoot.position}, rotation: {recenterRoot.rotation.eulerAngles}, scale: {recenterRoot.lossyScale}"
      );

      if (center == Vector3.zero)
      {
        Debug.Log("Hierarchy is already centered at origin.");
        return;
      }

      var directChildren = new List<Transform>();
      foreach (Transform child in recenterRoot)
      {
        directChildren.Add(child);
      }

      Debug.Log($"[Recenter Debug] Moving {directChildren.Count} direct children:");
      foreach (var child in directChildren)
      {
        var oldPos = child.localPosition;
        child.localPosition -= center;
        Debug.Log(
          $"[Recenter Debug]   {child.name}: ({oldPos.x:F2}, {oldPos.y:F2}, {oldPos.z:F2}) -> ({child.localPosition.x:F2}, {child.localPosition.y:F2}, {child.localPosition.z:F2})"
        );
      }

      Undo.CollapseUndoOperations(undoGroupIndex);
      EditorUtility.SetDirty(root);
      Debug.Log(
        $"Hierarchy recentered. Moved {directChildren.Count} objects by offset ({-center.x:F2}, {-center.y:F2}, {-center.z:F2})."
      );
    }

    [MenuItem("Tools/Recenter Selected Prefab Hierarchy", true)]
    private static bool ValidateRecenterSelectedPrefab()
    {
      return Selection.activeGameObject != null;
    }

    private static Vector3 CalculateBoundsCenter(Transform root, out string debugInfo)
    {
      var renderers = root.GetComponentsInChildren<Renderer>();
      if (renderers.Length == 0)
      {
        debugInfo = "No renderers found, using transform center fallback";
        return CalculateTransformCenter(root);
      }

      var bounds = new Bounds();
      var boundsInitialized = false;

      foreach (var renderer in renderers)
      {
        if (!boundsInitialized)
        {
          bounds = renderer.bounds;
          boundsInitialized = true;
        }
        else
        {
          bounds.Encapsulate(renderer.bounds);
        }
      }

      debugInfo =
        $"Found {renderers.Length} renderers. Bounds center (world): ({bounds.center.x:F2}, {bounds.center.y:F2}, {bounds.center.z:F2}), size: ({bounds.size.x:F2}, {bounds.size.y:F2}, {bounds.size.z:F2})";
      return root.InverseTransformPoint(bounds.center);
    }

    private static Vector3 CalculateTransformCenter(Transform root)
    {
      var allTransforms = root.GetComponentsInChildren<Transform>();
      if (allTransforms.Length <= 1)
      {
        return Vector3.zero;
      }

      var sum = Vector3.zero;
      var count = 0;

      foreach (var t in allTransforms)
      {
        if (t != root)
        {
          sum += t.position;
          count++;
        }
      }

      if (count == 0)
      {
        return Vector3.zero;
      }

      var worldCenter = sum / count;
      return root.InverseTransformPoint(worldCenter);
    }
  }
}
#endif
