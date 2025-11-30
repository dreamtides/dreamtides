#if UNITY_EDITOR

#nullable enable

using System.Collections.Generic;
using System.Linq;
using System.Text;
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
      var center = CalculateBoundsCenter(recenterRoot);

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

      foreach (var child in directChildren)
      {
        child.localPosition -= center;
      }

      Undo.CollapseUndoOperations(undoGroupIndex);
      EditorUtility.SetDirty(root);
      Debug.Log($"Hierarchy recentered. Moved {directChildren.Count} objects.");
    }

    [MenuItem("Tools/Recenter Selected Prefab Hierarchy", true)]
    private static bool ValidateRecenterSelectedPrefab()
    {
      return Selection.activeGameObject != null;
    }

    [MenuItem("Tools/Analyze Hierarchy Outliers")]
    public static void AnalyzeOutliers()
    {
      var selected = Selection.activeGameObject;
      if (selected == null)
      {
        EditorUtility.DisplayDialog(
          "Analyze Outliers",
          "Please select a GameObject to analyze.",
          "OK"
        );
        return;
      }

      var allRenderers = selected.GetComponentsInChildren<Renderer>();
      if (allRenderers.Length == 0)
      {
        Debug.Log("No renderers found in hierarchy.");
        return;
      }

      var renderers = allRenderers.Where(HasValidBounds).ToList();
      var skippedCount = allRenderers.Length - renderers.Count;

      var log = new StringBuilder();
      log.AppendLine("=== Hierarchy Bounds Analysis ===");
      log.AppendLine(
        $"Total renderers: {allRenderers.Length} ({skippedCount} skipped with zero-size bounds)"
      );
      log.AppendLine();

      if (renderers.Count == 0)
      {
        log.AppendLine("No renderers with valid bounds found.");
        Debug.Log(log.ToString());
        return;
      }

      var totalBounds = new Bounds();
      var boundsInitialized = false;
      foreach (var renderer in renderers)
      {
        if (!boundsInitialized)
        {
          totalBounds = renderer.bounds;
          boundsInitialized = true;
        }
        else
        {
          totalBounds.Encapsulate(renderer.bounds);
        }
      }

      log.AppendLine(
        $"Combined bounds center: ({totalBounds.center.x:F2}, {totalBounds.center.y:F2}, {totalBounds.center.z:F2})"
      );
      log.AppendLine(
        $"Combined bounds size: ({totalBounds.size.x:F2}, {totalBounds.size.y:F2}, {totalBounds.size.z:F2})"
      );
      log.AppendLine(
        $"Combined bounds min: ({totalBounds.min.x:F2}, {totalBounds.min.y:F2}, {totalBounds.min.z:F2})"
      );
      log.AppendLine(
        $"Combined bounds max: ({totalBounds.max.x:F2}, {totalBounds.max.y:F2}, {totalBounds.max.z:F2})"
      );
      log.AppendLine();

      log.AppendLine("=== Renderers at Extreme Bounds ===");
      log.AppendLine();

      var minX = renderers.OrderBy(r => r.bounds.min.x).Take(3).ToList();
      var maxX = renderers.OrderByDescending(r => r.bounds.max.x).Take(3).ToList();
      var minY = renderers.OrderBy(r => r.bounds.min.y).Take(3).ToList();
      var maxY = renderers.OrderByDescending(r => r.bounds.max.y).Take(3).ToList();
      var minZ = renderers.OrderBy(r => r.bounds.min.z).Take(3).ToList();
      var maxZ = renderers.OrderByDescending(r => r.bounds.max.z).Take(3).ToList();

      log.AppendLine("Min X (left edge):");
      foreach (var r in minX)
      {
        var path = GetHierarchyPath(r.transform, selected.transform);
        log.AppendLine(
          $"  {path}: bounds.min.x={r.bounds.min.x:F2}, size=({r.bounds.size.x:F2}, {r.bounds.size.y:F2}, {r.bounds.size.z:F2})"
        );
      }

      log.AppendLine("Max X (right edge):");
      foreach (var r in maxX)
      {
        var path = GetHierarchyPath(r.transform, selected.transform);
        log.AppendLine(
          $"  {path}: bounds.max.x={r.bounds.max.x:F2}, size=({r.bounds.size.x:F2}, {r.bounds.size.y:F2}, {r.bounds.size.z:F2})"
        );
      }

      log.AppendLine("Min Z (back edge):");
      foreach (var r in minZ)
      {
        var path = GetHierarchyPath(r.transform, selected.transform);
        log.AppendLine(
          $"  {path}: bounds.min.z={r.bounds.min.z:F2}, size=({r.bounds.size.x:F2}, {r.bounds.size.y:F2}, {r.bounds.size.z:F2})"
        );
      }

      log.AppendLine("Max Z (front edge):");
      foreach (var r in maxZ)
      {
        var path = GetHierarchyPath(r.transform, selected.transform);
        log.AppendLine(
          $"  {path}: bounds.max.z={r.bounds.max.z:F2}, size=({r.bounds.size.x:F2}, {r.bounds.size.y:F2}, {r.bounds.size.z:F2})"
        );
      }

      log.AppendLine();
      log.AppendLine("=== Largest Bounding Boxes ===");
      var largestByVolume = renderers
        .OrderByDescending(r => r.bounds.size.x * r.bounds.size.y * r.bounds.size.z)
        .Take(10)
        .ToList();

      foreach (var r in largestByVolume)
      {
        var path = GetHierarchyPath(r.transform, selected.transform);
        var size = r.bounds.size;
        var volume = size.x * size.y * size.z;
        log.AppendLine(
          $"  {path}: size=({size.x:F2}, {size.y:F2}, {size.z:F2}), volume={volume:F0}"
        );
      }

      Debug.Log(log.ToString());
    }

    [MenuItem("Tools/Analyze Hierarchy Outliers", true)]
    private static bool ValidateAnalyzeOutliers()
    {
      return Selection.activeGameObject != null;
    }

    private static string GetHierarchyPath(Transform target, Transform root)
    {
      var path = new List<string>();
      var current = target;
      while (current != null && current != root)
      {
        path.Add(current.name);
        current = current.parent;
      }
      path.Reverse();
      return string.Join("/", path);
    }

    private static Vector3 CalculateBoundsCenter(Transform root)
    {
      var renderers = root.GetComponentsInChildren<Renderer>();
      if (renderers.Length == 0)
      {
        return CalculateTransformCenter(root);
      }

      var bounds = new Bounds();
      var boundsInitialized = false;

      foreach (var renderer in renderers)
      {
        if (!HasValidBounds(renderer))
        {
          continue;
        }

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

      if (!boundsInitialized)
      {
        return CalculateTransformCenter(root);
      }

      return root.InverseTransformPoint(bounds.center);
    }

    private static bool HasValidBounds(Renderer renderer)
    {
      var size = renderer.bounds.size;
      return size.x > 0.001f || size.y > 0.001f || size.z > 0.001f;
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
