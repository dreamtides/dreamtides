#nullable enable

using System.Collections.Generic;
using UnityEngine;
#if UNITY_EDITOR
using UnityEditor;
#endif

namespace Dreamtides.EditorHelpers
{
  [DisallowMultipleComponent]
  public class DeleteSmallestChildren : MonoBehaviour
  {
    [SerializeField]
    private float relativeSizeThreshold = 0.01f;

    public void DeleteSmallest()
    {
#if UNITY_EDITOR
      var camera = Camera.main;
      if (camera == null)
      {
        EditorUtility.DisplayDialog(
          "Delete Smallest Children",
          "No MainCamera found in the scene. Add or tag a camera as MainCamera.",
          "OK"
        );
        return;
      }

      var candidates = CollectDeletableChildren(transform, camera);
      if (candidates.Count == 0)
      {
        EditorUtility.DisplayDialog(
          "Delete Smallest Children",
          "No child objects were found below the threshold.",
          "OK"
        );
        return;
      }

      var undoGroupIndex = Undo.GetCurrentGroup();
      Undo.SetCurrentGroupName("Delete Smallest Children");

      var deletedCount = 0;
      foreach (var candidate in candidates)
      {
        if (candidate == null)
        {
          continue;
        }

        Undo.DestroyObjectImmediate(candidate.gameObject);
        deletedCount++;
      }

      Undo.CollapseUndoOperations(undoGroupIndex);
      EditorUtility.SetDirty(this);
      Debug.Log(
        $"Deleted {deletedCount} child GameObjects under {name} with relative size <= {relativeSizeThreshold}."
      );
#else
      Debug.LogWarning("DeleteSmallestChildren is editor-only and is inactive at runtime.");
#endif
    }

    private List<Transform> CollectDeletableChildren(Transform root, Camera camera)
    {
      var rawCandidates = new List<Transform>();
      CollectCandidatesRecursive(root, root, camera, rawCandidates);
      return FilterCandidates(rawCandidates, root);
    }

    private void CollectCandidatesRecursive(
      Transform root,
      Transform current,
      Camera camera,
      List<Transform> candidates
    )
    {
      foreach (Transform child in current)
      {
        CollectCandidatesRecursive(root, child, camera, candidates);
      }

      if (current == root)
      {
        return;
      }

      if (!TryGetRendererBounds(current, out var bounds))
      {
        return;
      }

      var relativeHeight = CalculateRelativeHeight(bounds, camera);
      var threshold = Mathf.Max(relativeSizeThreshold, 0f);
      if (relativeHeight <= threshold)
      {
        candidates.Add(current);
      }
    }

    private List<Transform> FilterCandidates(List<Transform> candidates, Transform root)
    {
      var filtered = new List<Transform>();
      var blocked = new HashSet<Transform>();

      foreach (var candidate in candidates)
      {
        if (candidate == null)
        {
          continue;
        }

        if (blocked.Contains(candidate))
        {
          continue;
        }

        filtered.Add(candidate);

        var ancestor = candidate.parent;
        while (ancestor != null && ancestor != root)
        {
          blocked.Add(ancestor);
          ancestor = ancestor.parent;
        }
      }

      return filtered;
    }

    private bool TryGetRendererBounds(Transform target, out Bounds bounds)
    {
      bounds = new Bounds();
      var hasBounds = false;

      var meshRenderers = target.GetComponents<MeshRenderer>();
      foreach (var renderer in meshRenderers)
      {
        var rendererBounds = renderer.bounds;
        if (!HasValidBounds(rendererBounds))
        {
          continue;
        }

        if (!hasBounds)
        {
          bounds = rendererBounds;
          hasBounds = true;
        }
        else
        {
          bounds.Encapsulate(rendererBounds);
        }
      }

      var skinnedRenderers = target.GetComponents<SkinnedMeshRenderer>();
      foreach (var renderer in skinnedRenderers)
      {
        var rendererBounds = renderer.bounds;
        if (!HasValidBounds(rendererBounds))
        {
          continue;
        }

        if (!hasBounds)
        {
          bounds = rendererBounds;
          hasBounds = true;
        }
        else
        {
          bounds.Encapsulate(rendererBounds);
        }
      }

      return hasBounds;
    }

    private float CalculateRelativeHeight(Bounds bounds, Camera camera)
    {
      var radius = bounds.extents.magnitude;

      if (radius <= 0f)
      {
        return 0f;
      }

      if (camera.orthographic)
      {
        var orthoSize = Mathf.Max(camera.orthographicSize, 0.0001f);
        return radius / orthoSize;
      }

      var distance = (bounds.center - camera.transform.position).magnitude;
      var clampedDistance = Mathf.Max(distance, 0.0001f);
      var fovRadians = camera.fieldOfView * Mathf.Deg2Rad;
      var tanHalfFov = Mathf.Max(Mathf.Tan(fovRadians * 0.5f), 0.0001f);

      return radius / (clampedDistance * tanHalfFov);
    }

    private bool HasValidBounds(Bounds bounds)
    {
      var size = bounds.size;
      return size.x > 0.001f || size.y > 0.001f || size.z > 0.001f;
    }
  }

#if UNITY_EDITOR
  [CustomEditor(typeof(DeleteSmallestChildren))]
  public class DeleteSmallestChildrenEditor : Editor
  {
    public override void OnInspectorGUI()
    {
      DrawDefaultInspector();

      if (GUILayout.Button("Delete Smallest"))
      {
        var deleter = (DeleteSmallestChildren)target;
        deleter.DeleteSmallest();
      }
    }
  }
#endif
}
