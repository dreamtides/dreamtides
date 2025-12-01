#if UNITY_EDITOR

#nullable enable

using System.Collections.Generic;
using UnityEditor;
using UnityEngine;

namespace Dreamtides.EditorHelpers
{
  [DisallowMultipleComponent]
  public class ChunkedCullingLODGenerator : MonoBehaviour
  {
    [SerializeField]
    private float lod0TransitionHeight = 0.55f;

    [SerializeField]
    private float lod1TransitionHeight = 0.42f;

    [SerializeField]
    private float lod2TransitionHeight = 0.24f;

    [SerializeField]
    private float lod3TransitionHeight = 0.1f;

    [SerializeField]
    private float lod1KeepFraction = 0.55f;

    [SerializeField]
    private float lod2KeepFraction = 0.25f;

    [SerializeField]
    private float lod3KeepFraction = 0.1f;

    [SerializeField]
    private int minRenderersForGroup = 3;

    [SerializeField]
    private float maxGroupWorldSize = 60f;

    [SerializeField]
    private float minRendererDensity = 0.002f;

    [SerializeField]
    private float maxRelativeSize = 0.65f;

    [SerializeField]
    private float childCoveragePreference = 0.9f;

    [SerializeField]
    private float globalObjectSizeMultiplier = 1f;

    [SerializeField]
    private List<LODGroup> createdLodGroups = new List<LODGroup>();

    public void CreateChunkedCullingLODs()
    {
      if (!ValidateTransitionOrder())
      {
        EditorUtility.DisplayDialog(
          "Create Chunked Culling LODs",
          "LOD transition heights must decrease from LOD0 through LOD3.",
          "OK"
        );
        return;
      }

      var camera = Camera.main;
      if (camera == null)
      {
        EditorUtility.DisplayDialog(
          "Create Chunked Culling LODs",
          "No MainCamera found in the scene. Add or tag a camera as MainCamera.",
          "OK"
        );
        return;
      }

      var undoGroupIndex = Undo.GetCurrentGroup();
      Undo.SetCurrentGroupName("Create Chunked Culling LODs");

      RemoveExistingLODGroups(transform);

      var rootData = BuildNodeData(transform);
      if (rootData.RendererCount == 0)
      {
        Undo.CollapseUndoOperations(undoGroupIndex);
        EditorUtility.DisplayDialog(
          "Create Chunked Culling LODs",
          "No eligible renderers were found on this GameObject.",
          "OK"
        );
        return;
      }

      var candidates = SelectGroups(rootData, null);
      if (candidates.Count == 0)
      {
        Undo.CollapseUndoOperations(undoGroupIndex);
        EditorUtility.DisplayDialog(
          "Create Chunked Culling LODs",
          "No compact renderer groups were found for LOD creation.",
          "OK"
        );
        return;
      }

      var assignedRenderers = new HashSet<Renderer>();
      var createdCount = 0;

      createdLodGroups.Clear();

      foreach (var candidate in candidates)
      {
        if (candidate.Transform.GetComponent<LODGroup>() != null)
        {
          continue;
        }

        var renderers = CollectRenderers(candidate.Transform, assignedRenderers);
        if (renderers.Count == 0)
        {
          continue;
        }

        var lodGroup = Undo.AddComponent<LODGroup>(candidate.Transform.gameObject);
        Undo.RecordObject(lodGroup, "Configure chunked LODGroup");

        ConfigureLODGroup(lodGroup, renderers, camera);

        createdLodGroups.Add(lodGroup);

        foreach (var renderer in renderers)
        {
          assignedRenderers.Add(renderer);
        }

        createdCount++;
      }

      Undo.CollapseUndoOperations(undoGroupIndex);
      EditorUtility.SetDirty(this);

      Debug.Log(
        $"Created {createdCount} culling-only LODGroups under {name} from {candidates.Count} compact clusters."
      );
    }

    public void RemoveChunkedCullingLODs()
    {
      var undoGroupIndex = Undo.GetCurrentGroup();
      Undo.SetCurrentGroupName("Remove Chunked Culling LODs");

      RemoveExistingLODGroups(transform);

      Undo.CollapseUndoOperations(undoGroupIndex);
      EditorUtility.SetDirty(this);
    }

    private bool ValidateTransitionOrder()
    {
      return lod0TransitionHeight > lod1TransitionHeight
        && lod1TransitionHeight > lod2TransitionHeight
        && lod2TransitionHeight > lod3TransitionHeight;
    }

    private void RemoveExistingLODGroups(Transform root)
    {
      createdLodGroups.Clear();
      var lodGroups = root.GetComponentsInChildren<LODGroup>(true);
      foreach (var lodGroup in lodGroups)
      {
        Undo.DestroyObjectImmediate(lodGroup);
      }
    }

    private NodeData BuildNodeData(Transform root)
    {
      var node = new NodeData
      {
        Transform = root,
        Children = new List<NodeData>(),
        Bounds = new Bounds(),
        HasBounds = false,
        RendererCount = 0,
      };

      var renderers = root.GetComponents<Renderer>();
      foreach (var renderer in renderers)
      {
        if (renderer == null)
        {
          continue;
        }

        if (renderer.GetComponentInParent<LODGroup>() != null)
        {
          continue;
        }

        var bounds = renderer.bounds;
        if (!HasValidBounds(bounds))
        {
          continue;
        }

        if (!node.HasBounds)
        {
          node.Bounds = bounds;
          node.HasBounds = true;
        }
        else
        {
          node.Bounds.Encapsulate(bounds);
        }

        node.RendererCount++;
      }

      foreach (Transform child in root)
      {
        var childNode = BuildNodeData(child);
        if (childNode.RendererCount == 0 && childNode.Children.Count == 0)
        {
          continue;
        }

        node.Children.Add(childNode);

        if (childNode.HasBounds)
        {
          if (!node.HasBounds)
          {
            node.Bounds = childNode.Bounds;
            node.HasBounds = true;
          }
          else
          {
            node.Bounds.Encapsulate(childNode.Bounds);
          }
        }

        node.RendererCount += childNode.RendererCount;
      }

      return node;
    }

    private List<NodeData> SelectGroups(NodeData node, Bounds? parentBounds)
    {
      var childGroups = new List<NodeData>();
      foreach (var child in node.Children)
      {
        childGroups.AddRange(SelectGroups(child, node.HasBounds ? node.Bounds : parentBounds));
      }

      if (node.RendererCount == 0 || !node.HasBounds)
      {
        return childGroups;
      }

      if (!ShouldCreateGroup(node, parentBounds))
      {
        return childGroups;
      }

      if (childGroups.Count == 0)
      {
        return new List<NodeData> { node };
      }

      var childRendererCount = SumRendererCounts(childGroups);
      var coverage = childRendererCount / (float)node.RendererCount;
      var childScore = AverageGroupScore(childGroups);
      var nodeScore = GroupScore(node);

      if (coverage >= childCoveragePreference && childScore <= nodeScore)
      {
        return childGroups;
      }

      return new List<NodeData> { node };
    }

    private bool ShouldCreateGroup(NodeData node, Bounds? parentBounds)
    {
      if (node.RendererCount < minRenderersForGroup)
      {
        return false;
      }

      var size = node.Bounds.size;
      var longestAxis = Mathf.Max(size.x, Mathf.Max(size.y, size.z));
      var volume = Mathf.Max(size.x * size.y * size.z, 0.0001f);
      var density = node.RendererCount / volume;

      var parentLongest = 0f;
      if (parentBounds.HasValue)
      {
        var parentSize = parentBounds.Value.size;
        parentLongest = Mathf.Max(parentSize.x, Mathf.Max(parentSize.y, parentSize.z));
      }

      var relativeSize = parentLongest > 0f ? longestAxis / parentLongest : 1f;

      if (longestAxis > maxGroupWorldSize)
      {
        return false;
      }

      if (density >= minRendererDensity)
      {
        return true;
      }

      return relativeSize <= maxRelativeSize;
    }

    private int SumRendererCounts(List<NodeData> groups)
    {
      var total = 0;
      foreach (var group in groups)
      {
        total += group.RendererCount;
      }
      return total;
    }

    private float GroupScore(NodeData group)
    {
      var size = group.Bounds.size;
      var longestAxis = Mathf.Max(size.x, Mathf.Max(size.y, size.z));
      return longestAxis / Mathf.Max(group.RendererCount, 1);
    }

    private float AverageGroupScore(List<NodeData> groups)
    {
      var weightedSum = 0f;
      var totalWeight = 0;

      foreach (var group in groups)
      {
        var weight = Mathf.Max(group.RendererCount, 1);
        weightedSum += GroupScore(group) * weight;
        totalWeight += weight;
      }

      if (totalWeight == 0)
      {
        return float.MaxValue;
      }

      return weightedSum / totalWeight;
    }

    private List<Renderer> CollectRenderers(Transform root, HashSet<Renderer> assignedRenderers)
    {
      var renderers = new List<Renderer>();
      var allRenderers = root.GetComponentsInChildren<Renderer>(true);

      foreach (var renderer in allRenderers)
      {
        if (renderer == null)
        {
          continue;
        }

        if (assignedRenderers.Contains(renderer))
        {
          continue;
        }

        var parentLOD = renderer.GetComponentInParent<LODGroup>();
        if (parentLOD != null && parentLOD.transform != root)
        {
          continue;
        }

        renderers.Add(renderer);
      }

      return renderers;
    }

    private void ConfigureLODGroup(LODGroup lodGroup, List<Renderer> renderers, Camera camera)
    {
      var rendererInfos = BuildRendererInfos(renderers, camera);
      if (rendererInfos.Count == 0)
      {
        return;
      }

      var lods = BuildLODs(rendererInfos);
      lodGroup.SetLODs(lods);
      lodGroup.fadeMode = LODFadeMode.None;
      lodGroup.animateCrossFading = false;
      lodGroup.RecalculateBounds();
      var multiplier = Mathf.Max(globalObjectSizeMultiplier, 0.01f);
      lodGroup.size *= multiplier;
      EditorUtility.SetDirty(lodGroup);
    }

    private List<RendererInfo> BuildRendererInfos(List<Renderer> renderers, Camera camera)
    {
      var infos = new List<RendererInfo>();

      foreach (var renderer in renderers)
      {
        var relativeHeight = CalculateRelativeHeight(renderer, camera);
        infos.Add(new RendererInfo { Renderer = renderer, RelativeHeight = relativeHeight });
      }

      infos.Sort((a, b) => a.RelativeHeight.CompareTo(b.RelativeHeight));
      return infos;
    }

    private LOD[] BuildLODs(List<RendererInfo> rendererInfos)
    {
      var lod0Renderers = new List<Renderer>();
      foreach (var info in rendererInfos)
      {
        lod0Renderers.Add(info.Renderer);
      }

      var lod1Renderers = FilterRenderersByFraction(rendererInfos, lod1KeepFraction);
      var lod2Renderers = FilterRenderersByFraction(rendererInfos, lod2KeepFraction);
      var lod3Renderers = FilterRenderersByFraction(rendererInfos, lod3KeepFraction);

      return new LOD[]
      {
        new LOD(lod0TransitionHeight, lod0Renderers.ToArray()),
        new LOD(lod1TransitionHeight, lod1Renderers),
        new LOD(lod2TransitionHeight, lod2Renderers),
        new LOD(lod3TransitionHeight, lod3Renderers),
      };
    }

    private Renderer[] FilterRenderersByFraction(
      List<RendererInfo> rendererInfos,
      float keepFraction
    )
    {
      var renderers = new List<Renderer>();
      var clampedFraction = Mathf.Clamp01(keepFraction);
      var countToKeep = Mathf.CeilToInt(rendererInfos.Count * clampedFraction);
      if (countToKeep <= 0 && rendererInfos.Count > 0)
      {
        countToKeep = 1;
      }

      for (var i = rendererInfos.Count - countToKeep; i < rendererInfos.Count; i++)
      {
        if (i >= 0 && i < rendererInfos.Count)
          renderers.Add(rendererInfos[i].Renderer);
      }

      if (renderers.Count == 0 && rendererInfos.Count > 0)
      {
        renderers.Add(rendererInfos[rendererInfos.Count - 1].Renderer);
      }

      return renderers.ToArray();
    }

    private float CalculateRelativeHeight(Renderer renderer, Camera camera)
    {
      var bounds = renderer.bounds;
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

    private struct RendererInfo
    {
      public Renderer Renderer;
      public float RelativeHeight;
    }

    private class NodeData
    {
      public Transform Transform = null!;
      public Bounds Bounds;
      public bool HasBounds;
      public int RendererCount;
      public List<NodeData> Children = null!;
    }
  }

  [CustomEditor(typeof(ChunkedCullingLODGenerator))]
  public class ChunkedCullingLODGeneratorEditor : Editor
  {
    public override void OnInspectorGUI()
    {
      DrawDefaultInspector();

      if (GUILayout.Button("Create LODs"))
      {
        var generator = (ChunkedCullingLODGenerator)target;
        generator.CreateChunkedCullingLODs();
      }

      if (GUILayout.Button("Remove LODs"))
      {
        var generator = (ChunkedCullingLODGenerator)target;
        generator.RemoveChunkedCullingLODs();
      }
    }
  }
}
#endif
