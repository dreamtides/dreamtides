using UnityEditor;
using UnityEngine;

public static class RecenterHierarchy
{
  [MenuItem("Tools/Recenter Selected Hierarchy Around Origin")]
  public static void RecenterSelected()
  {
    var root = Selection.activeGameObject;
    if (root == null)
    {
      Debug.LogWarning("No GameObject selected. Please select the prefab instance or root object.");
      return;
    }

    // Get all renderers to find the visual bounds
    var renderers = root.GetComponentsInChildren<Renderer>();
    if (renderers.Length == 0)
    {
      Debug.LogWarning("Selected object has no Renderers in children.");
      return;
    }

    // Compute combined bounds in world space
    Bounds worldBounds = renderers[0].bounds;
    for (int i = 1; i < renderers.Length; i++)
    {
      worldBounds.Encapsulate(renderers[i].bounds);
    }

    // Convert the visual center to the root's local space
    Transform rootT = root.transform;
    Vector3 localCenter = rootT.InverseTransformPoint(worldBounds.center);

    // Shift all children so that the center moves to (0,0,0)
    OffsetChildren(rootT, localCenter);

    Debug.Log($"Recentered '{root.name}' by {-localCenter} so its contents are around the origin.");
  }

  private static void OffsetChildren(Transform root, Vector3 localOffset)
  {
    // Move every child by the same local offset
    foreach (Transform child in root)
    {
      OffsetRecursive(child, localOffset);
    }
  }

  private static void OffsetRecursive(Transform t, Vector3 localOffset)
  {
    t.localPosition -= localOffset;
    foreach (Transform child in t)
    {
      OffsetRecursive(child, localOffset);
    }
  }
}
