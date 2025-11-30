#if UNITY_EDITOR

#nullable enable

using System.Collections.Generic;
using System.Linq;
using UnityEditor;
using UnityEngine;

namespace Dreamtides.Editors
{
  public static class FlattenHierarchy
  {
    [MenuItem("Tools/Flatten Selected Prefab Hierarchy")]
    public static void FlattenSelectedPrefab()
    {
      var selected = Selection.activeGameObject;
      if (selected == null)
      {
        EditorUtility.DisplayDialog(
          "Flatten Hierarchy",
          "Please select a GameObject to flatten.",
          "OK"
        );
        return;
      }

      var prefabStage = UnityEditor.SceneManagement.PrefabStageUtility.GetCurrentPrefabStage();
      if (prefabStage == null)
      {
        EditorUtility.DisplayDialog(
          "Flatten Hierarchy",
          "Please open the prefab in Prefab Mode before flattening.",
          "OK"
        );
        return;
      }

      var root = prefabStage.prefabContentsRoot;
      if (!selected.transform.IsChildOf(root.transform) && selected != root)
      {
        EditorUtility.DisplayDialog(
          "Flatten Hierarchy",
          "Selected object must be part of the prefab being edited.",
          "OK"
        );
        return;
      }

      var undoGroupIndex = Undo.GetCurrentGroup();
      Undo.SetCurrentGroupName("Flatten Hierarchy");
      Undo.RegisterFullObjectHierarchyUndo(root, "Flatten Hierarchy");

      var flattenRoot = selected.transform;

      var unpackedCount = UnpackNestedPrefabs(flattenRoot);
      if (unpackedCount > 0)
      {
        Debug.Log($"Unpacked {unpackedCount} nested prefab instances.");
      }

      var meshObjects = CollectMeshObjects(flattenRoot);

      Debug.Log($"Found {meshObjects.Count} objects with mesh components to preserve.");

      var worldTransforms = CaptureWorldTransforms(meshObjects);
      ReparentToRoot(flattenRoot, meshObjects);
      RestoreWorldTransforms(meshObjects, worldTransforms);
      CleanupEmptyObjects(flattenRoot, meshObjects);

      Undo.CollapseUndoOperations(undoGroupIndex);
      EditorUtility.SetDirty(root);
      Debug.Log(
        $"Hierarchy flattened. {meshObjects.Count} mesh objects are now direct children of {flattenRoot.name}."
      );
    }

    [MenuItem("Tools/Flatten Selected Prefab Hierarchy", true)]
    private static bool ValidateFlattenSelectedPrefab()
    {
      return Selection.activeGameObject != null;
    }

    private static int UnpackNestedPrefabs(Transform root)
    {
      var count = 0;
      var nestedPrefabs = new List<GameObject>();
      CollectNestedPrefabsRecursive(root, nestedPrefabs);

      foreach (var prefabInstance in nestedPrefabs)
      {
        if (prefabInstance != null)
        {
          PrefabUtility.UnpackPrefabInstance(
            prefabInstance,
            PrefabUnpackMode.Completely,
            InteractionMode.AutomatedAction
          );
          count++;
        }
      }

      return count;
    }

    private static void CollectNestedPrefabsRecursive(
      Transform current,
      List<GameObject> nestedPrefabs
    )
    {
      foreach (Transform child in current)
      {
        CollectNestedPrefabsRecursive(child, nestedPrefabs);
      }

      if (PrefabUtility.IsAnyPrefabInstanceRoot(current.gameObject))
      {
        nestedPrefabs.Add(current.gameObject);
      }
    }

    private static List<Transform> CollectMeshObjects(Transform root)
    {
      var result = new List<Transform>();
      CollectMeshObjectsRecursive(root, result);
      return result;
    }

    private static void CollectMeshObjectsRecursive(Transform current, List<Transform> result)
    {
      foreach (Transform child in current)
      {
        CollectMeshObjectsRecursive(child, result);
      }

      if (
        current.GetComponent<MeshRenderer>() != null
        || current.GetComponent<SkinnedMeshRenderer>() != null
        || current.GetComponent<MeshFilter>() != null
      )
      {
        result.Add(current);
      }
    }

    private static Dictionary<Transform, WorldTransformData> CaptureWorldTransforms(
      List<Transform> objects
    )
    {
      var result = new Dictionary<Transform, WorldTransformData>();
      foreach (var t in objects)
      {
        result[t] = new WorldTransformData
        {
          Position = t.position,
          Rotation = t.rotation,
          LossyScale = t.lossyScale,
        };
      }

      return result;
    }

    private static void ReparentToRoot(Transform root, List<Transform> objects)
    {
      foreach (var t in objects)
      {
        if (t != root)
        {
          t.SetParent(root, worldPositionStays: true);
        }
      }
    }

    private static void RestoreWorldTransforms(
      List<Transform> objects,
      Dictionary<Transform, WorldTransformData> worldTransforms
    )
    {
      foreach (var t in objects)
      {
        if (worldTransforms.TryGetValue(t, out var data))
        {
          t.position = data.Position;
          t.rotation = data.Rotation;
          t.localScale = CalculateLocalScale(t, data.LossyScale);
        }
      }
    }

    private static Vector3 CalculateLocalScale(Transform t, Vector3 targetWorldScale)
    {
      if (t.parent == null)
      {
        return targetWorldScale;
      }

      var parentLossyScale = t.parent.lossyScale;
      return new Vector3(
        parentLossyScale.x != 0 ? targetWorldScale.x / parentLossyScale.x : targetWorldScale.x,
        parentLossyScale.y != 0 ? targetWorldScale.y / parentLossyScale.y : targetWorldScale.y,
        parentLossyScale.z != 0 ? targetWorldScale.z / parentLossyScale.z : targetWorldScale.z
      );
    }

    private static void CleanupEmptyObjects(Transform root, List<Transform> meshObjects)
    {
      var meshObjectSet = new HashSet<Transform>(meshObjects) { root };
      var deletedCount = DeleteEmptyObjectsRecursive(root, meshObjectSet);

      if (deletedCount > 0)
      {
        Debug.Log($"Removed {deletedCount} empty intermediate objects.");
      }
    }

    private static int DeleteEmptyObjectsRecursive(Transform current, HashSet<Transform> preserve)
    {
      var deletedCount = 0;

      var children = new List<Transform>();
      foreach (Transform child in current)
      {
        children.Add(child);
      }

      foreach (var child in children)
      {
        deletedCount += DeleteEmptyObjectsRecursive(child, preserve);
      }

      if (preserve.Contains(current))
      {
        return deletedCount;
      }

      if (current.childCount == 0 && IsEmptyTransform(current))
      {
        Undo.DestroyObjectImmediate(current.gameObject);
        deletedCount++;
      }

      return deletedCount;
    }

    private static bool IsEmptyTransform(Transform t)
    {
      var components = t.GetComponents<Component>();
      return components.Length == 1 && components[0] is Transform;
    }

    private struct WorldTransformData
    {
      public Vector3 Position;
      public Quaternion Rotation;
      public Vector3 LossyScale;
    }
  }
}
#endif
