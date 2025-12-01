#if UNITY_EDITOR

#nullable enable

using System.Collections.Generic;
using Dreamtides.Components;
using Dreamtides.Services;
using UnityEditor;
using UnityEngine;

namespace Dreamtides.EditorHelpers
{
  [DisallowMultipleComponent]
  public class DeleteExpensiveChildren : MonoBehaviour
  {
    public void LogTriangleCount()
    {
      var viewport = ResolveViewport();
      if (viewport == null)
      {
        EditorUtility.DisplayDialog(
          "Delete Expensive Children",
          "Unable to locate a GameViewport. Ensure the scene has a Registry or both a Camera and Canvas.",
          "OK"
        );
        return;
      }

      var target = Mathf.Max(targetTriangleCount, 0);
      var report = BuildHierarchyReport(transform, viewport, null);
      var plan = PlanRemovals(transform, viewport, target);
      var lines = new List<string>();
      lines.Add(
        $"DeleteExpensiveChildren on {name}: triangles {report.TotalTriangles} (target {target})."
      );
      if (plan.Count == 0)
      {
        lines.Add("No removals needed to hit target.");
      }
      else
      {
        var preview = BuildPlanPreview(plan);
        lines.Add($"Would remove {plan.Count} objects (showing {preview.Count}):");
        lines.AddRange(preview);
      }

      Debug.Log(string.Join("\n", lines));
    }

    public void DeleteExpensive()
    {
      var viewport = ResolveViewport();
      if (viewport == null)
      {
        EditorUtility.DisplayDialog(
          "Delete Expensive Children",
          "Unable to locate a GameViewport. Ensure the scene has a Registry or both a Camera and Canvas.",
          "OK"
        );
        return;
      }

      var target = Mathf.Max(targetTriangleCount, 0);
      var report = BuildHierarchyReport(transform, viewport, null);
      var startingTriangles = report.TotalTriangles;

      if (startingTriangles <= target)
      {
        Debug.Log(
          $"DeleteExpensiveChildren on {name}: triangles {startingTriangles} already at or below target {target}."
        );
        return;
      }

      var plan = PlanRemovals(transform, viewport, target);

      if (plan.Count == 0)
      {
        EditorUtility.DisplayDialog(
          "Delete Expensive Children",
          "No candidate children with triangles were found to delete.",
          "OK"
        );
        return;
      }

      var preview = BuildPlanPreview(plan);
      if (preview.Count > 0)
      {
        Debug.Log(
          $"DeleteExpensiveChildren on {name}: removing {plan.Count} objects (showing {preview.Count}):\n{string.Join("\n", preview)}"
        );
      }

      var undoGroupIndex = Undo.GetCurrentGroup();
      Undo.SetCurrentGroupName("Delete Expensive Children");

      for (var i = 0; i < plan.Count; i++)
      {
        var candidate = plan[i];
        Undo.DestroyObjectImmediate(candidate.Transform.gameObject);
      }

      Undo.CollapseUndoOperations(undoGroupIndex);
      EditorUtility.SetDirty(this);

      var finalReport = BuildHierarchyReport(transform, viewport, null);
      Debug.Log(
        $"DeleteExpensiveChildren on {name}: triangles {startingTriangles} -> {finalReport.TotalTriangles} (target {target})."
      );
    }

    [SerializeField]
    private int targetTriangleCount = 5000;

    [SerializeField]
    private bool includeInactive = true;

    [SerializeField]
    private float triangleWeight = 1f;

    [SerializeField]
    private float screenSizeWeight = 1f;

    [SerializeField]
    private float particleWeight = 0.25f;

    [SerializeField]
    private float rendererWeight = 0.1f;

    [SerializeField]
    private float minimumScreenFraction = 0.0005f;

    [SerializeField]
    private float granularityPenalty = 0.5f;

    private HierarchyReport BuildHierarchyReport(
      Transform root,
      IGameViewport viewport,
      HashSet<Transform>? excluded
    )
    {
      var aggregates = new Dictionary<Transform, AggregateData>();
      var rootAggregate = AccumulateAggregateData(root, aggregates, excluded);
      var maxima = CalculateMaxima(aggregates, root);
      var candidates = new List<Candidate>();

      foreach (var pair in aggregates)
      {
        if (pair.Key == root)
        {
          continue;
        }

        var data = pair.Value;
        if (data.TriangleCount <= 0)
        {
          continue;
        }

        var screenFraction = data.HasBounds
          ? Mathf.Max(CalculateScreenFraction(data.Bounds, viewport), 0f)
          : 0f;
        if (screenFraction < minimumScreenFraction)
        {
          screenFraction = 0f;
        }

        var score = EvaluateScore(data, screenFraction, maxima);
        var candidate = new Candidate
        {
          Transform = pair.Key,
          TriangleCount = data.TriangleCount,
          ScreenFraction = screenFraction,
          ParticleCost = data.ParticleCost,
          RendererCount = data.RendererCount,
          Score = score,
        };
        candidates.Add(candidate);
      }

      candidates.Sort((a, b) => b.Score.CompareTo(a.Score));

      return new HierarchyReport
      {
        TotalTriangles = rootAggregate.TriangleCount,
        Candidates = candidates,
      };
    }

    private AggregateData AccumulateAggregateData(
      Transform current,
      Dictionary<Transform, AggregateData> aggregates,
      HashSet<Transform>? excluded
    )
    {
      if (excluded != null && excluded.Contains(current))
      {
        var skipped = new AggregateData();
        aggregates[current] = skipped;
        return skipped;
      }

      if (!includeInactive && !current.gameObject.activeInHierarchy)
      {
        var inactive = new AggregateData();
        aggregates[current] = inactive;
        return inactive;
      }

      var triangleCount = 0;
      var rendererCount = 0;
      var particleCost = 0f;
      var hasBounds = false;
      var bounds = new Bounds();
      var nodeCount = 0;

      var renderers = current.GetComponents<Renderer>();
      for (var i = 0; i < renderers.Length; i++)
      {
        var renderer = renderers[i];
        if (renderer == null)
        {
          continue;
        }

        if (!includeInactive && !renderer.gameObject.activeInHierarchy)
        {
          continue;
        }

        var triangles = CountTriangles(renderer);
        triangleCount += triangles;
        rendererCount++;
        if (triangles > 0)
        {
          nodeCount = 1;
        }

        if (TryGetRendererBounds(renderer, out var rendererBounds))
        {
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
      }

      var particleSystems = current.GetComponents<ParticleSystem>();
      for (var i = 0; i < particleSystems.Length; i++)
      {
        var system = particleSystems[i];
        if (system == null)
        {
          continue;
        }

        if (!includeInactive && !system.gameObject.activeInHierarchy)
        {
          continue;
        }

        particleCost += EstimateParticleCost(system);
      }

      for (var i = 0; i < current.childCount; i++)
      {
        var child = current.GetChild(i);
        var childData = AccumulateAggregateData(child, aggregates, excluded);
        triangleCount += childData.TriangleCount;
        rendererCount += childData.RendererCount;
        particleCost += childData.ParticleCost;
        nodeCount += childData.NodeCount;

        if (childData.HasBounds)
        {
          if (!hasBounds)
          {
            bounds = childData.Bounds;
            hasBounds = true;
          }
          else
          {
            bounds.Encapsulate(childData.Bounds);
          }
        }
      }

      var aggregate = new AggregateData
      {
        TriangleCount = triangleCount,
        Bounds = bounds,
        HasBounds = hasBounds,
        ParticleCost = particleCost,
        RendererCount = rendererCount,
        NodeCount = nodeCount,
      };

      aggregates[current] = aggregate;
      return aggregate;
    }

    private Maxima CalculateMaxima(Dictionary<Transform, AggregateData> aggregates, Transform root)
    {
      var maxima = new Maxima();

      foreach (var pair in aggregates)
      {
        if (pair.Key == root)
        {
          continue;
        }

        var data = pair.Value;
        if (data.TriangleCount > maxima.MaxTriangleCount)
        {
          maxima.MaxTriangleCount = data.TriangleCount;
        }

        if (data.ParticleCost > maxima.MaxParticleCost)
        {
          maxima.MaxParticleCost = data.ParticleCost;
        }

        if (data.RendererCount > maxima.MaxRendererCount)
        {
          maxima.MaxRendererCount = data.RendererCount;
        }
      }

      return maxima;
    }

    private float EvaluateScore(AggregateData data, float screenFraction, Maxima maxima)
    {
      var triangleComponent = triangleWeight * data.TriangleCount;
      var screenComponent = data.TriangleCount * screenSizeWeight * screenFraction;
      var particleComponent =
        maxima.MaxParticleCost > 0f
          ? particleWeight * data.ParticleCost / maxima.MaxParticleCost * data.TriangleCount
          : 0f;
      var rendererComponent =
        maxima.MaxRendererCount > 0
          ? rendererWeight * data.RendererCount / maxima.MaxRendererCount * data.TriangleCount
          : 0f;
      var baseScore = triangleComponent + screenComponent + particleComponent + rendererComponent;
      var penalty = 1f + Mathf.Max(data.NodeCount - 1, 0) * granularityPenalty;
      return baseScore / penalty;
    }

    private List<Candidate> PlanRemovals(Transform root, IGameViewport viewport, int target)
    {
      var removed = new HashSet<Transform>();
      var plan = new List<Candidate>();
      var safety = 0;
      var report = BuildHierarchyReport(root, viewport, removed);

      while (
        report.TotalTriangles > target && TrySelectCandidate(report.Candidates, out var candidate)
      )
      {
        plan.Add(candidate);
        removed.Add(candidate.Transform);
        safety++;
        if (safety > 10000)
        {
          break;
        }

        report = BuildHierarchyReport(root, viewport, removed);
        if (report.Candidates.Count == 0)
        {
          break;
        }
      }

      return plan;
    }

    private List<string> BuildPlanPreview(List<Candidate> plan)
    {
      var preview = new List<string>();
      var limit = Mathf.Min(plan.Count, 25);
      for (var i = 0; i < limit; i++)
      {
        var candidate = plan[i];
        var path = GetHierarchyPath(candidate.Transform);
        preview.Add($"{i + 1}. {path} — {candidate.TriangleCount} triangles");
      }

      return preview;
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

    private bool TrySelectCandidate(List<Candidate> candidates, out Candidate candidate)
    {
      candidate = default;
      if (candidates.Count == 0)
      {
        return false;
      }

      candidate = candidates[0];
      return candidate.TriangleCount > 0;
    }

    private IGameViewport? ResolveViewport()
    {
      var registry = GetComponentInParent<Registry>();
      if (registry == null)
      {
        var registries = FindObjectsByType<Registry>(
          FindObjectsInactive.Include,
          FindObjectsSortMode.None
        );

        for (var i = 0; i < registries.Length; i++)
        {
          var candidate = registries[i];
          if (candidate != null && candidate.gameObject.scene == gameObject.scene)
          {
            registry = candidate;
            break;
          }
        }
      }

      if (registry != null && registry.GameViewport != null)
      {
        return registry.GameViewport;
      }

      return RealViewport.CreateForEditor();
    }

    private float CalculateScreenFraction(Bounds bounds, IGameViewport viewport)
    {
      var extents = bounds.extents;
      if (extents == Vector3.zero)
      {
        return 0f;
      }

      var corners = new Vector3[8];
      var index = 0;
      for (var x = -1; x <= 1; x += 2)
      {
        for (var y = -1; y <= 1; y += 2)
        {
          for (var z = -1; z <= 1; z += 2)
          {
            corners[index] = bounds.center + Vector3.Scale(extents, new Vector3(x, y, z));
            index++;
          }
        }
      }

      var minX = float.MaxValue;
      var maxX = float.MinValue;
      var minY = float.MaxValue;
      var maxY = float.MinValue;
      var hasPoint = false;

      for (var i = 0; i < corners.Length; i++)
      {
        var viewportPoint = viewport.WorldToViewportPoint(corners[i]);
        if (viewportPoint.z <= 0f)
        {
          continue;
        }

        hasPoint = true;
        minX = Mathf.Min(minX, viewportPoint.x);
        maxX = Mathf.Max(maxX, viewportPoint.x);
        minY = Mathf.Min(minY, viewportPoint.y);
        maxY = Mathf.Max(maxY, viewportPoint.y);
      }

      if (!hasPoint)
      {
        return 0f;
      }

      minX = Mathf.Clamp01(minX);
      maxX = Mathf.Clamp01(maxX);
      minY = Mathf.Clamp01(minY);
      maxY = Mathf.Clamp01(maxY);

      var width = Mathf.Max(0f, maxX - minX);
      var height = Mathf.Max(0f, maxY - minY);
      return Mathf.Clamp01(width * height);
    }

    private int CountTriangles(Renderer renderer)
    {
      var particleRenderer = renderer as ParticleSystemRenderer;
      if (particleRenderer != null)
      {
        var meshes = new Mesh[particleRenderer.meshCount];
        particleRenderer.GetMeshes(meshes);
        var particleMeshTriangles = 0;
        for (var i = 0; i < meshes.Length; i++)
        {
          var particleMesh = meshes[i];
          if (particleMesh == null)
          {
            continue;
          }

          particleMeshTriangles += CountTriangles(particleMesh);
        }

        if (particleMeshTriangles > 0)
        {
          return particleMeshTriangles;
        }
      }

      var mesh = (Mesh?)null;

      var meshFilter = renderer.GetComponent<MeshFilter>();
      if (meshFilter != null)
      {
        mesh = meshFilter.sharedMesh;
      }

      var skinnedRenderer = renderer as SkinnedMeshRenderer;
      if (skinnedRenderer != null)
      {
        mesh = skinnedRenderer.sharedMesh;
      }

      if (mesh == null && particleRenderer != null)
      {
        mesh = particleRenderer.mesh;
      }

      if (mesh == null)
      {
        return 0;
      }

      return CountTriangles(mesh);
    }

    private int CountTriangles(Mesh mesh)
    {
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

    private bool TryGetRendererBounds(Renderer renderer, out Bounds bounds)
    {
      bounds = renderer.bounds;
      if (!HasValidBounds(bounds))
      {
        return false;
      }

      return true;
    }

    private float EstimateParticleCost(ParticleSystem system)
    {
      var main = system.main;
      var emission = system.emission;
      var burstCount = emission.burstCount;
      var bursts = new ParticleSystem.Burst[burstCount];
      emission.GetBursts(bursts);

      var burstTotal = 0f;
      for (var i = 0; i < bursts.Length; i++)
      {
        var burst = bursts[i];
        burstTotal += burst.count.constantMax;
      }

      var rate = emission.rateOverTime.constantMax + emission.rateOverDistance.constantMax;
      var lifetime = main.startLifetime.constantMax;
      var size = main.startSize.constantMax;
      var maxParticles = main.maxParticles;

      var cost = maxParticles + rate * lifetime + burstTotal;
      cost *= 1f + size * 0.5f;
      return Mathf.Max(cost, 0f);
    }

    private bool HasValidBounds(Bounds bounds)
    {
      var size = bounds.size;
      return size.x > 0.001f || size.y > 0.001f || size.z > 0.001f;
    }

    private struct AggregateData
    {
      public int TriangleCount;
      public Bounds Bounds;
      public bool HasBounds;
      public float ParticleCost;
      public int RendererCount;
      public int NodeCount;
    }

    private struct Candidate
    {
      public Transform Transform;
      public int TriangleCount;
      public float ScreenFraction;
      public float ParticleCost;
      public int RendererCount;
      public float Score;
    }

    private struct HierarchyReport
    {
      public int TotalTriangles;
      public List<Candidate> Candidates;
    }

    private struct Maxima
    {
      public int MaxTriangleCount;
      public float MaxParticleCost;
      public int MaxRendererCount;
    }
  }

  [CustomEditor(typeof(DeleteExpensiveChildren))]
  public class DeleteExpensiveChildrenEditor : Editor
  {
    public override void OnInspectorGUI()
    {
      DrawDefaultInspector();

      if (GUILayout.Button("Delete Expensive Children"))
      {
        var deleter = (DeleteExpensiveChildren)target;
        deleter.DeleteExpensive();
      }

      if (GUILayout.Button("Log Triangle Count"))
      {
        var deleter = (DeleteExpensiveChildren)target;
        deleter.LogTriangleCount();
      }
    }
  }
}
#endif
