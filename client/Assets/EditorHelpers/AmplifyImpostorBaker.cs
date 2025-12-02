#if UNITY_EDITOR

#nullable enable

using AmplifyImpostors;
using UnityEditor;
using UnityEngine;

namespace Dreamtides.EditorHelpers
{
  public static class AmplifyImpostorBaker
  {
    private const string TargetFolder = "Assets/Content/Imposters/SpaceBlackMarket";

    [MenuItem("Tools/Bake Selected Amplify Impostor")]
    public static void BakeSelected()
    {
      var selected = Selection.activeGameObject;
      if (selected == null)
      {
        EditorUtility.DisplayDialog(
          "Bake Amplify Impostor",
          "Select a GameObject to bake an impostor for.",
          "OK"
        );
        return;
      }

      var success = Bake(selected);
      if (!success)
      {
        EditorUtility.DisplayDialog(
          "Bake Amplify Impostor",
          "No renderers were found on the selected object.",
          "OK"
        );
      }
    }

    public static bool Bake(GameObject target)
    {
      var impostor = target.GetComponent<AmplifyImpostor>();
      if (impostor == null)
      {
        impostor = Undo.AddComponent<AmplifyImpostor>(target);
      }

      Undo.RecordObject(impostor, "Configure Amplify Impostor");
      impostor.RootTransform = target.transform;
      impostor.LodGroup = target.GetComponent<LODGroup>();
      if (impostor.Renderers == null || impostor.Renderers.Length == 0)
      {
        AssignRenderers(impostor);
      }

      if (impostor.Renderers == null || impostor.Renderers.Length == 0)
      {
        return false;
      }

      var asset = LoadOrCreateAsset(target.name);
      ConfigureAsset(asset);
      impostor.Data = asset;

      EditorUtility.SetDirty(impostor);
      EditorUtility.SetDirty(asset);
      AssetDatabase.SaveAssets();

      impostor.RenderAllDeferredGroups();
      GroupWithImpostor(impostor);
      return true;
    }

    private static AmplifyImpostorAsset LoadOrCreateAsset(string objectName)
    {
      EnsureFolder(TargetFolder);
      var assetPath = $"{TargetFolder}/{objectName}_Impostor.asset";
      var asset = AssetDatabase.LoadAssetAtPath<AmplifyImpostorAsset>(assetPath);
      if (asset != null)
      {
        return asset;
      }

      asset = ScriptableObject.CreateInstance<AmplifyImpostorAsset>();
      asset.name = $"{objectName}_Impostor";
      AssetDatabase.CreateAsset(asset, assetPath);
      return asset;
    }

    private static void ConfigureAsset(AmplifyImpostorAsset asset)
    {
      asset.ImpostorType = ImpostorType.Spherical;
      asset.SelectedSize = 2048;
      asset.LockedSizes = true;
      asset.TexSize = new Vector2(2048, 2048);
      asset.DecoupleAxisFrames = false;
      asset.HorizontalFrames = 8;
      asset.VerticalFrames = 8;
      asset.PixelPadding = 32;
      if (asset.Preset == null)
      {
        var presetPath = AssetDatabase.GUIDToAssetPath(AmplifyImpostor.DefaultPreset);
        asset.Preset = AssetDatabase.LoadAssetAtPath<AmplifyImpostorBakePreset>(presetPath);
      }
    }

    private static void AssignRenderers(AmplifyImpostor impostor)
    {
      if (impostor.LodGroup == null)
      {
        impostor.Renderers = impostor.RootTransform.GetComponentsInChildren<Renderer>();
        return;
      }

      var lods = impostor.LodGroup.GetLODs();
      if (lods.Length == 0)
      {
        impostor.Renderers = impostor.RootTransform.GetComponentsInChildren<Renderer>();
        return;
      }

      var lastIndex = lods.Length - 1;
      var vertexCount = 0;
      var lastRenderers = lods[lastIndex].renderers;
      for (var i = 0; i < lastRenderers.Length; i++)
      {
        var meshFilter =
          lastRenderers[i] != null ? lastRenderers[i].GetComponent<MeshFilter>() : null;
        if (meshFilter != null && meshFilter.sharedMesh != null)
        {
          vertexCount += meshFilter.sharedMesh.vertexCount;
        }
      }

      if (vertexCount < 8)
      {
        lastIndex--;
      }

      lastIndex = Mathf.Max(lastIndex, 1);
      for (var i = lastIndex - 1; i >= 0; i--)
      {
        var renderers = lods[i].renderers;
        if (renderers != null && renderers.Length > 0)
        {
          impostor.Renderers = renderers;
          break;
        }
      }

      if (impostor.Renderers == null || impostor.Renderers.Length == 0)
      {
        impostor.Renderers = impostor.RootTransform.GetComponentsInChildren<Renderer>();
      }

      impostor.m_insertIndex = lastIndex;
      if (vertexCount < 8)
      {
        impostor.m_lodReplacement = LODReplacement.ReplaceLast;
      }
    }

    private static void GroupWithImpostor(AmplifyImpostor impostor)
    {
      var impostorObject = impostor.m_lastImpostor;
      if (impostorObject == null)
      {
        return;
      }

      var target = impostor.gameObject;
      var parentName = target.name;
      var originalParent = target.transform.parent;
      var group = FindGroup(target.transform, impostorObject.transform, parentName);
      if (group == null)
      {
        var groupObject = new GameObject(parentName);
        Undo.RegisterCreatedObjectUndo(groupObject, "Create Impostor Group");
        var groupTransform = groupObject.transform;
        groupTransform.SetParent(originalParent, false);
        groupTransform.position = target.transform.position;
        groupTransform.rotation = target.transform.rotation;
        groupTransform.localScale = target.transform.lossyScale;
        group = groupTransform;
      }

      Undo.SetTransformParent(target.transform, group, "Parent Original With Impostor");
      Undo.SetTransformParent(impostorObject.transform, group, "Parent Impostor");
      target.SetActive(false);
      impostorObject.SetActive(true);
    }

    private static Transform? FindGroup(
      Transform target,
      Transform impostorTransform,
      string parentName
    )
    {
      if (target.parent != null && target.parent.name == parentName)
      {
        return target.parent;
      }

      if (impostorTransform.parent != null && impostorTransform.parent.name == parentName)
      {
        return impostorTransform.parent;
      }

      return null;
    }

    private static void EnsureFolder(string path)
    {
      if (AssetDatabase.IsValidFolder(path))
      {
        return;
      }

      var segments = path.Split('/');
      var current = segments[0];
      for (var i = 1; i < segments.Length; i++)
      {
        var next = $"{current}/{segments[i]}";
        if (!AssetDatabase.IsValidFolder(next))
        {
          AssetDatabase.CreateFolder(current, segments[i]);
        }

        current = next;
      }
    }
  }
}

#endif
