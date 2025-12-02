#nullable enable

using System.Collections.Generic;
using UnityEngine;
#if UNITY_EDITOR
using UnityEditor;
#endif

namespace Dreamtides.EditorHelpers
{
  [DisallowMultipleComponent]
  public class DeleteChildMeshColliders : MonoBehaviour
  {
    public void DeleteChildColliders()
    {
#if UNITY_EDITOR
      var colliders = CollectTargets();
      if (colliders.Count == 0)
      {
        EditorUtility.DisplayDialog(
          "Delete Child Colliders",
          "No 3D Collider components were found on child objects.",
          "OK"
        );
        return;
      }

      var undoGroup = Undo.GetCurrentGroup();
      Undo.SetCurrentGroupName("Delete Child Colliders");

      for (var i = 0; i < colliders.Count; i++)
      {
        var collider = colliders[i];
        if (collider == null)
        {
          continue;
        }

        Undo.DestroyObjectImmediate(collider);
      }

      Undo.CollapseUndoOperations(undoGroup);
      EditorUtility.SetDirty(this);

      Debug.Log($"Deleted {colliders.Count} 3D Collider components under {name}.");
#else
      Debug.LogWarning("DeleteChildMeshColliders is editor-only and is inactive at runtime.");
#endif
    }

    [SerializeField]
    private bool includeInactive = true;

    private List<Collider> CollectTargets()
    {
      var results = new List<Collider>();
      var colliders = transform.GetComponentsInChildren<Collider>(includeInactive);
      for (var i = 0; i < colliders.Length; i++)
      {
        var collider = colliders[i];
        if (collider == null)
        {
          continue;
        }

        if (collider.transform == transform)
        {
          continue;
        }

        results.Add(collider);
      }

      return results;
    }
  }

#if UNITY_EDITOR
  [CustomEditor(typeof(DeleteChildMeshColliders))]
  public class DeleteChildMeshCollidersEditor : Editor
  {
    public override void OnInspectorGUI()
    {
      DrawDefaultInspector();

      if (GUILayout.Button("Delete Child Colliders"))
      {
        var deleter = (DeleteChildMeshColliders)target;
        deleter.DeleteChildColliders();
      }
    }
  }
#endif
}
