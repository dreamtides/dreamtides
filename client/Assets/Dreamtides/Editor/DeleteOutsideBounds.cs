#if UNITY_EDITOR

#nullable enable

using System.Collections.Generic;
using UnityEditor;
using UnityEngine;

namespace Dreamtides.Editors
{
  public static class DeleteOutsideBounds
  {
    [MenuItem("Tools/Delete Objects Outside Selected Box Collider")]
    public static void DeleteObjectsOutsideBounds()
    {
      var selected = Selection.activeGameObject;
      if (selected == null)
      {
        EditorUtility.DisplayDialog(
          "Delete Outside Bounds",
          "Please select a GameObject with a BoxCollider.",
          "OK"
        );
        return;
      }

      var boxCollider = selected.GetComponent<BoxCollider>();
      if (boxCollider == null)
      {
        EditorUtility.DisplayDialog(
          "Delete Outside Bounds",
          "Selected GameObject must have a BoxCollider component.",
          "OK"
        );
        return;
      }

      var bounds = boxCollider.bounds;

      if (
        !EditorUtility.DisplayDialog(
          "Delete Outside Bounds",
          $"This will delete all objects in the scene that do not overlap with the box collider on {selected.name}.\n\nObjects without bounds (like particle effects) will be kept if their position is within the bounds.\n\nThis action cannot be undone easily. Continue?",
          "Yes, Delete",
          "Cancel"
        )
      )
      {
        return;
      }

      var undoGroupIndex = Undo.GetCurrentGroup();
      Undo.SetCurrentGroupName("Delete Objects Outside Bounds");

      var allRootObjects = new List<GameObject>();
      foreach (
        var root in UnityEngine.SceneManagement.SceneManager.GetActiveScene().GetRootGameObjects()
      )
      {
        allRootObjects.Add(root);
      }

      var deletedCount = 0;
      var keptCount = 0;

      foreach (var rootObj in allRootObjects)
      {
        if (rootObj == selected)
        {
          keptCount++;
          continue;
        }

        var result = ProcessObjectRecursive(rootObj.transform, bounds, selected.transform);
        deletedCount += result.Deleted;
        keptCount += result.Kept;
      }

      Undo.CollapseUndoOperations(undoGroupIndex);
      Debug.Log(
        $"Deleted {deletedCount} objects outside bounds. Kept {keptCount} objects (including the box collider object)."
      );
    }

    [MenuItem("Tools/Delete Objects Outside Selected Box Collider", true)]
    private static bool ValidateDeleteObjectsOutsideBounds()
    {
      var selected = Selection.activeGameObject;
      return selected != null && selected.GetComponent<BoxCollider>() != null;
    }

    private static ProcessResult ProcessObjectRecursive(
      Transform obj,
      Bounds bounds,
      Transform preserveTransform
    )
    {
      if (obj == preserveTransform || obj.IsChildOf(preserveTransform))
      {
        var kept = CountDescendants(obj);
        return new ProcessResult { Kept = kept, Deleted = 0 };
      }

      if (ShouldPreserveObject(obj.gameObject))
      {
        var kept = CountDescendants(obj);
        return new ProcessResult { Kept = kept, Deleted = 0 };
      }

      var shouldKeep = ShouldKeepObject(obj.gameObject, bounds);
      if (shouldKeep)
      {
        var kept = CountDescendants(obj);
        return new ProcessResult { Kept = kept, Deleted = 0 };
      }

      var children = new List<Transform>();
      foreach (Transform child in obj)
      {
        children.Add(child);
      }

      var totalDeleted = 0;
      var totalKept = 0;

      foreach (var child in children)
      {
        var result = ProcessObjectRecursive(child, bounds, preserveTransform);
        totalDeleted += result.Deleted;
        totalKept += result.Kept;
      }

      if (totalKept == 0)
      {
        Undo.DestroyObjectImmediate(obj.gameObject);
        return new ProcessResult { Kept = 0, Deleted = 1 + totalDeleted };
      }

      return new ProcessResult { Kept = 1 + totalKept, Deleted = totalDeleted };
    }

    private static int CountDescendants(Transform root)
    {
      var count = 1;
      foreach (Transform child in root)
      {
        count += CountDescendants(child);
      }
      return count;
    }

    private static bool ShouldPreserveObject(GameObject obj)
    {
      var camera = obj.GetComponent<Camera>();
      if (camera != null && (camera == Camera.main || obj.CompareTag("MainCamera")))
      {
        return true;
      }

      var light = obj.GetComponent<Light>();
      if (light != null && light.type == LightType.Directional)
      {
        return true;
      }

      return false;
    }

    private static bool ShouldKeepObject(GameObject obj, Bounds bounds)
    {
      var hasBounds = false;
      var boundsToCheck = new Bounds();

      var renderer = obj.GetComponent<Renderer>();
      if (renderer != null && HasValidBounds(renderer))
      {
        boundsToCheck = renderer.bounds;
        hasBounds = true;
      }
      else
      {
        var collider = obj.GetComponent<Collider>();
        if (collider != null && !collider.isTrigger)
        {
          boundsToCheck = collider.bounds;
          hasBounds = true;
        }
      }

      if (hasBounds)
      {
        return bounds.Intersects(boundsToCheck);
      }

      var position = obj.transform.position;
      return bounds.Contains(position);
    }

    private static bool HasValidBounds(Renderer renderer)
    {
      var size = renderer.bounds.size;
      return size.x > 0.001f || size.y > 0.001f || size.z > 0.001f;
    }

    private struct ProcessResult
    {
      public int Kept;
      public int Deleted;
    }
  }
}
#endif
