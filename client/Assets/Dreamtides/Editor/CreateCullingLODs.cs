#if UNITY_EDITOR

#nullable enable

using System.Collections.Generic;
using UnityEditor;
using UnityEngine;

namespace Dreamtides.Editors
{
  public static class CreateCullingLODs
  {
    private const float Lod0TransitionHeight = 0.55f;
    private const float Lod1TransitionHeight = 0.42f;
    private const float Lod2TransitionHeight = 0.24f;
    private const float Lod3TransitionHeight = 0.1f;

    private const float Lod1KeepFraction = 0.55f;
    private const float Lod2KeepFraction = 0.25f;
    private const float Lod3KeepFraction = 0.1f;

    [MenuItem("Tools/Create Culling LODs")]
    public static void CreateCullingLODsForSelection()
    {
      var selected = Selection.activeGameObject;
      if (selected == null)
      {
        EditorUtility.DisplayDialog(
          "Create Culling LODs",
          "Please select a GameObject to build LODs for.",
          "OK"
        );
        return;
      }

      if (!ValidateTransitionOrder())
      {
        EditorUtility.DisplayDialog(
          "Create Culling LODs",
          "LOD transition heights must decrease from LOD0 through LOD3.",
          "OK"
        );
        return;
      }

      var camera = Camera.main;
      if (camera == null)
      {
        EditorUtility.DisplayDialog(
          "Create Culling LODs",
          "No MainCamera found in the scene. Add or tag a camera as MainCamera.",
          "OK"
        );
        return;
      }

      var rendererInfos = BuildRendererInfos(selected.transform, camera);
      if (rendererInfos.Count == 0)
      {
        EditorUtility.DisplayDialog(
          "Create Culling LODs",
          "No eligible child renderers were found on the selected GameObject.",
          "OK"
        );
        return;
      }

      var undoGroupIndex = Undo.GetCurrentGroup();
      Undo.SetCurrentGroupName("Create Culling LODs");

      var lodGroup = selected.GetComponent<LODGroup>();
      if (lodGroup == null)
      {
        lodGroup = Undo.AddComponent<LODGroup>(selected);
      }

      Undo.RecordObject(lodGroup, "Configure LODGroup");

      var lods = BuildLODs(rendererInfos);
      lodGroup.SetLODs(lods);
      lodGroup.fadeMode = LODFadeMode.None;
      lodGroup.animateCrossFading = false;
      lodGroup.RecalculateBounds();

      Undo.CollapseUndoOperations(undoGroupIndex);
      EditorUtility.SetDirty(lodGroup);

      Debug.Log(
        $"Created culling-only LODs on {selected.name} using {rendererInfos.Count} renderers."
      );
    }

    [MenuItem("Tools/Create Culling LODs", true)]
    private static bool ValidateCreateCullingLODs()
    {
      return Selection.activeGameObject != null;
    }

    private static bool ValidateTransitionOrder()
    {
      return Lod0TransitionHeight > Lod1TransitionHeight
        && Lod1TransitionHeight > Lod2TransitionHeight
        && Lod2TransitionHeight > Lod3TransitionHeight;
    }

    private static List<RendererInfo> BuildRendererInfos(Transform root, Camera camera)
    {
      var renderers = CollectEligibleRenderers(root);
      var infos = new List<RendererInfo>();

      foreach (var renderer in renderers)
      {
        var relativeHeight = CalculateRelativeHeight(renderer, camera);
        infos.Add(new RendererInfo { Renderer = renderer, RelativeHeight = relativeHeight });
      }

      infos.Sort((a, b) => a.RelativeHeight.CompareTo(b.RelativeHeight));
      return infos;
    }

    private static List<Renderer> CollectEligibleRenderers(Transform root)
    {
      var renderers = new List<Renderer>();
      var allRenderers = root.GetComponentsInChildren<Renderer>(true);

      foreach (var renderer in allRenderers)
      {
        if (renderer == null)
        {
          continue;
        }

        var parentLOD = renderer.GetComponentInParent<LODGroup>();
        if (parentLOD != null && parentLOD.gameObject != root.gameObject)
        {
          continue;
        }

        renderers.Add(renderer);
      }

      return renderers;
    }

    private static LOD[] BuildLODs(List<RendererInfo> rendererInfos)
    {
      var lod0Renderers = new List<Renderer>();
      foreach (var info in rendererInfos)
      {
        lod0Renderers.Add(info.Renderer);
      }

      var lod1Renderers = FilterRenderersByFraction(rendererInfos, Lod1KeepFraction);
      var lod2Renderers = FilterRenderersByFraction(rendererInfos, Lod2KeepFraction);
      var lod3Renderers = FilterRenderersByFraction(rendererInfos, Lod3KeepFraction);

      return new LOD[]
      {
        new LOD(Lod0TransitionHeight, lod0Renderers.ToArray()),
        new LOD(Lod1TransitionHeight, lod1Renderers),
        new LOD(Lod2TransitionHeight, lod2Renderers),
        new LOD(Lod3TransitionHeight, lod3Renderers),
      };
    }

    private static Renderer[] FilterRenderersByFraction(
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

    private static float CalculateRelativeHeight(Renderer renderer, Camera camera)
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

    private struct RendererInfo
    {
      public Renderer Renderer;
      public float RelativeHeight;
    }
  }
}
#endif
