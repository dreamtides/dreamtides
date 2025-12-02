#nullable enable

using System.Collections.Generic;
using UnityEngine;
#if UNITY_EDITOR
using AmplifyImpostors;
using Dreamtides.Components;
using Dreamtides.Services;
using UnityEditor;
#endif

namespace Dreamtides.EditorHelpers
{
#if UNITY_EDITOR
  [DisallowMultipleComponent]
  public class ImpostorConverter : MonoBehaviour
  {
    [SerializeField]
    private List<GameObject> conversionTargets = new List<GameObject>();

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

    [SerializeField]
    private string skipImpostorTag = "SkipImpostor";

    public void SuggestCandidates()
    {
      var viewport = ResolveViewport();
      if (viewport == null)
      {
        EditorUtility.DisplayDialog(
          "Impostor Converter",
          "Unable to locate a GameViewport. Ensure the scene has a Registry or both a Camera and Canvas.",
          "OK"
        );
        return;
      }

      CleanupNullTargets();
      var blocked = BuildBlockedSet();
      var report = BuildHierarchyReport(transform, viewport, blocked);
      var added = new List<string>();

      for (var i = 0; i < report.Candidates.Count && added.Count < 5; i++)
      {
        var candidate = report.Candidates[i];
        if (blocked.Contains(candidate.Transform))
        {
          continue;
        }

        conversionTargets.Add(candidate.Transform.gameObject);
        AddWithRelativesToBlocked(candidate.Transform, blocked);
        added.Add(GetHierarchyPath(candidate.Transform));
      }

      if (added.Count == 0)
      {
        Debug.Log("ImpostorConverter: No candidates available to suggest.");
      }
      else
      {
        Debug.Log(
          $"ImpostorConverter: Added {added.Count} suggestions:\n{string.Join("\n", added)}"
        );
      }
    }

    public void DryRun()
    {
      CleanupNullTargets();
      var targets = conversionTargets;
      var aggregates = new Dictionary<Transform, AggregateData>();
      var rootAggregate = AccumulateAggregateData(transform, aggregates, new HashSet<Transform>());
      var totalTriangles = 0;
      var willSkip = 0;
      var willConvert = 0;
      for (var i = 0; i < targets.Count; i++)
      {
        var target = targets[i];
        if (target == null)
        {
          continue;
        }

        if (aggregates.TryGetValue(target.transform, out var aggregate))
        {
          totalTriangles += aggregate.TriangleCount;
        }

        if (HasImpostor(target.transform))
        {
          willSkip++;
        }
        else
        {
          willConvert++;
        }
      }
      var lines = new List<string>();
      lines.Add(
        $"ImpostorConverter on {name}: {targets.Count} objects scheduled for impostors ({totalTriangles} triangles out of {rootAggregate.TriangleCount}), {willConvert} will convert, {willSkip} will be skipped."
      );
      var preview = BuildTargetPreview(targets);
      if (preview.Count == 0)
      {
        lines.Add("No targets configured.");
      }
      else
      {
        lines.AddRange(preview);
      }

      Debug.Log(string.Join("\n", lines));
    }

    public void Convert()
    {
      CleanupNullTargets();
      if (conversionTargets.Count == 0)
      {
        EditorUtility.DisplayDialog(
          "Impostor Converter",
          "No targets configured to convert.",
          "OK"
        );
        return;
      }

      var undoGroupIndex = Undo.GetCurrentGroup();
      Undo.SetCurrentGroupName("Convert Targets To Impostors");

      for (var i = 0; i < conversionTargets.Count; i++)
      {
        var target = conversionTargets[i];
        if (target == null)
        {
          continue;
        }

        if (HasImpostor(target.transform))
        {
          Debug.Log(
            $"ImpostorConverter on {name}: skipping already-converted {GetHierarchyPath(target.transform)}."
          );
          continue;
        }

        var success = AmplifyImpostorBaker.Bake(target);
        if (!success)
        {
          Debug.LogWarning(
            $"ImpostorConverter on {name}: failed to bake {GetHierarchyPath(target.transform)}."
          );
        }
        else
        {
          Debug.Log(
            $"ImpostorConverter on {name}: converted {GetHierarchyPath(target.transform)}."
          );
        }
      }

      Undo.CollapseUndoOperations(undoGroupIndex);
      EditorUtility.SetDirty(this);
    }

    public void RemoveImpostors()
    {
      CleanupNullTargets();
      if (conversionTargets.Count == 0)
      {
        Debug.Log("ImpostorConverter: No targets to restore.");
        return;
      }

      var undoGroupIndex = Undo.GetCurrentGroup();
      Undo.SetCurrentGroupName("Remove Impostors");

      for (var i = 0; i < conversionTargets.Count; i++)
      {
        var target = conversionTargets[i];
        if (target == null)
        {
          continue;
        }

        RestoreTarget(target);
      }

      Undo.CollapseUndoOperations(undoGroupIndex);
      EditorUtility.SetDirty(this);
    }

    private void RestoreTarget(GameObject target)
    {
      var impostor = target.GetComponent<AmplifyImpostor>();
      var targetTransform = target.transform;
      var group = targetTransform.parent;
      var impostorObject = impostor != null ? impostor.m_lastImpostor : null;
      var parent = group != null ? group.parent : null;

      Undo.SetTransformParent(targetTransform, parent, "Unparent Original");
      target.SetActive(true);

      if (impostorObject != null)
      {
        Undo.DestroyObjectImmediate(impostorObject);
      }

      if (impostor != null)
      {
        Undo.DestroyObjectImmediate(impostor);
      }

      if (group != null && group.childCount == 0)
      {
        Undo.DestroyObjectImmediate(group.gameObject);
      }
    }

    private List<string> BuildTargetPreview(List<GameObject> targets)
    {
      var preview = new List<string>();
      var limit = Mathf.Min(targets.Count, 25);
      for (var i = 0; i < limit; i++)
      {
        var target = targets[i];
        if (target == null)
        {
          continue;
        }

        var path = GetHierarchyPath(target.transform);
        preview.Add($"{i + 1}. {path}");
      }

      return preview;
    }

    private void CleanupNullTargets()
    {
      for (var i = conversionTargets.Count - 1; i >= 0; i--)
      {
        if (conversionTargets[i] == null)
        {
          conversionTargets.RemoveAt(i);
        }
      }
    }

    private HashSet<Transform> BuildBlockedSet()
    {
      var blocked = new HashSet<Transform>();
      for (var i = 0; i < conversionTargets.Count; i++)
      {
        var target = conversionTargets[i];
        if (target == null)
        {
          continue;
        }

        var transform = target.transform;
        AddWithRelativesToBlocked(transform, blocked);
      }

      return blocked;
    }

    private void AddWithRelativesToBlocked(Transform transform, HashSet<Transform> blocked)
    {
      var current = transform;
      while (current != null)
      {
        blocked.Add(current);
        current = current.parent;
      }

      AddDescendants(transform, blocked);
    }

    private void AddDescendants(Transform root, HashSet<Transform> blocked)
    {
      var stack = new Stack<Transform>();
      stack.Push(root);
      while (stack.Count > 0)
      {
        var current = stack.Pop();
        blocked.Add(current);
        for (var i = 0; i < current.childCount; i++)
        {
          stack.Push(current.GetChild(i));
        }
      }
    }

    private HierarchyReport BuildHierarchyReport(
      Transform root,
      IGameViewport viewport,
      HashSet<Transform> excluded
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

        if (excluded.Contains(pair.Key))
        {
          continue;
        }

        if (ShouldSkip(pair.Key))
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
      HashSet<Transform> excluded
    )
    {
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

    private bool ShouldSkip(Transform candidate)
    {
      if (string.IsNullOrEmpty(skipImpostorTag))
      {
        return HasImpostor(candidate);
      }

      try
      {
        if (candidate.CompareTag(skipImpostorTag))
        {
          return true;
        }
      }
      catch (UnityException)
      {
        return HasImpostor(candidate);
      }

      return HasImpostor(candidate);
    }

    private bool HasImpostor(Transform candidate)
    {
      return candidate.GetComponent<AmplifyImpostor>() != null;
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

  [CustomEditor(typeof(ImpostorConverter))]
  public class ImpostorConverterEditor : Editor
  {
    public override void OnInspectorGUI()
    {
      DrawDefaultInspector();

      if (GUILayout.Button("Suggest 5 Candidates"))
      {
        var converter = (ImpostorConverter)target;
        converter.SuggestCandidates();
      }

      if (GUILayout.Button("Dry Run (Log Plan)"))
      {
        var converter = (ImpostorConverter)target;
        converter.DryRun();
      }

      if (GUILayout.Button("Convert To Impostors"))
      {
        var converter = (ImpostorConverter)target;
        converter.Convert();
      }

      if (GUILayout.Button("Remove Impostors"))
      {
        var converter = (ImpostorConverter)target;
        converter.RemoveImpostors();
      }
    }
  }
#else
  [DisallowMultipleComponent]
  public class ImpostorConverter : MonoBehaviour
  {
    [SerializeField]
    private List<GameObject> conversionTargets = new List<GameObject>();

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

    [SerializeField]
    private string skipImpostorTag = "SkipImpostor";

    public void SuggestCandidates() { }

    public void DryRun() { }

    public void Convert() { }

    public void RemoveImpostors() { }
  }
#endif
}
