#if UNITY_EDITOR

#nullable enable

using System.Collections.Generic;
using System.Linq;
using UnityEditor;
using UnityEngine;

namespace Dreamtides.EditorHelpers
{
  [DisallowMultipleComponent]
  public class MostExpensiveChildren : MonoBehaviour
  {
    [SerializeField]
    private int topCount = 10;

    [SerializeField]
    private bool includeInactive = true;

    public void LogMostExpensiveRenderers()
    {
      var renderers = transform.GetComponentsInChildren<Renderer>(includeInactive)
        .Where(renderer => renderer != null && renderer.transform != transform)
        .Select(renderer => new RendererCost
        {
          Renderer = renderer,
          TriangleCount = CountTriangles(renderer),
        })
        .Where(cost => cost.TriangleCount > 0)
        .OrderByDescending(cost => cost.TriangleCount)
        .ToList();

      if (renderers.Count == 0)
      {
        EditorUtility.DisplayDialog(
          "Most Expensive Children",
          "No child renderers with triangles were found under this GameObject.",
          "OK"
        );
        return;
      }

      var resultCount = Mathf.Min(Mathf.Max(topCount, 1), renderers.Count);
      var lines = new List<string>();
      lines.Add($"Top {resultCount} renderers under {name} by triangle count:");

      for (var i = 0; i < resultCount; i++)
      {
        var cost = renderers[i];
        lines.Add(
          $"{i + 1}. {GetHierarchyPath(cost.Renderer.transform)} — {cost.TriangleCount} triangles"
        );
      }

      Debug.Log(string.Join("\n", lines));
    }

    public void LogMostExpensiveGameObjects()
    {
      var totals = new Dictionary<Transform, int>();
      AccumulateTriangleCounts(transform, totals);

      totals.Remove(transform);

      var ranked = totals
        .Where(entry => entry.Key != null && entry.Value > 0)
        .OrderByDescending(entry => entry.Value)
        .ToList();

      if (ranked.Count == 0)
      {
        EditorUtility.DisplayDialog(
          "Most Expensive Children",
          "No child GameObjects with triangles were found under this GameObject.",
          "OK"
        );
        return;
      }

      var resultCount = Mathf.Min(Mathf.Max(topCount, 1), ranked.Count);
      var lines = new List<string>();
      lines.Add($"Top {resultCount} child hierarchies under {name} by total triangles:");

      for (var i = 0; i < resultCount; i++)
      {
        var entry = ranked[i];
        lines.Add($"{i + 1}. {GetHierarchyPath(entry.Key)} — {entry.Value} triangles");
      }

      Debug.Log(string.Join("\n", lines));
    }

    private void AccumulateTriangleCounts(Transform current, Dictionary<Transform, int> totals)
    {
      if (!includeInactive && !current.gameObject.activeInHierarchy)
      {
        totals[current] = 0;
        return;
      }

      var selfCount = CountTrianglesForTransform(current);
      var sum = selfCount;

      foreach (Transform child in current)
      {
        AccumulateTriangleCounts(child, totals);
        sum += totals[child];
      }

      totals[current] = sum;
    }

    private int CountTrianglesForTransform(Transform target)
    {
      var renderers = target.GetComponents<Renderer>();
      var total = 0;

      foreach (var renderer in renderers)
      {
        if (renderer == null)
        {
          continue;
        }

        if (!includeInactive && !renderer.gameObject.activeInHierarchy)
        {
          continue;
        }

        total += CountTriangles(renderer);
      }

      return total;
    }

    private int CountTriangles(Renderer renderer)
    {
      Mesh mesh = null;

      var meshRenderer = renderer as MeshRenderer;
      if (meshRenderer != null)
      {
        var meshFilter = renderer.GetComponent<MeshFilter>();
        if (meshFilter != null)
        {
          mesh = meshFilter.sharedMesh;
        }
      }
      else
      {
        var skinned = renderer as SkinnedMeshRenderer;
        if (skinned != null)
        {
          mesh = skinned.sharedMesh;
        }
      }

      if (mesh == null)
      {
        return 0;
      }

      var subMeshCount = mesh.subMeshCount;
      var total = 0;

      for (var i = 0; i < subMeshCount; i++)
      {
        total += (int)mesh.GetIndexCount(i) / 3;
      }

      if (subMeshCount == 0 && mesh.triangles != null)
      {
        total = mesh.triangles.Length / 3;
      }

      return total;
    }

    private string GetHierarchyPath(Transform target)
    {
      var stack = new Stack<string>();
      var current = target;

      while (current != null)
      {
        stack.Push(current.name);
        if (current == transform)
        {
          break;
        }

        current = current.parent;
      }

      return string.Join("/", stack);
    }

    private struct RendererCost
    {
      public Renderer Renderer;
      public int TriangleCount;
    }
  }

  [CustomEditor(typeof(MostExpensiveChildren))]
  public class MostExpensiveChildrenEditor : Editor
  {
    public override void OnInspectorGUI()
    {
      DrawDefaultInspector();

      if (GUILayout.Button("Log Most Expensive Renderers"))
      {
        var finder = (MostExpensiveChildren)target;
        finder.LogMostExpensiveRenderers();
      }

      if (GUILayout.Button("Log Most Expensive GameObjects"))
      {
        var finder = (MostExpensiveChildren)target;
        finder.LogMostExpensiveGameObjects();
      }
    }
  }
}
#endif
