#nullable enable

using System.Collections.Generic;
using Dreamtides.Components;
using Dreamtides.Layout;
using UnityEditor;
using UnityEngine;
using UnityEngine.SceneManagement;

namespace Dreamtides.Editors
{
  public class GameLayoutCodeGenerator : EditorWindow
  {
    GameLayout? _portraitLayout;
    GameLayout? _landscapeLayout;
    string _portraitClassName = "GeneratedPortraitGameLayout";
    string _landscapeClassName = "GeneratedLandscapeGameLayout";

    GameObject? _sitesRoot;
    string _sitesClassName = "GeneratedSites";

    [MenuItem("Tools/Generate Test Code")]
    static void ShowWindow()
    {
      var window = GetWindow<GameLayoutCodeGenerator>("Test Code Generator");
      window.AutoPopulateDefaults();
    }

    void AutoPopulateDefaults()
    {
      if (_portraitLayout == null)
      {
        _portraitLayout = FindRootGameLayout("PortraitLayout");
      }

      if (_landscapeLayout == null)
      {
        _landscapeLayout = FindRootGameLayout("LandscapeLayout");
      }

      if (_sitesRoot == null)
      {
        _sitesRoot = FindRootGameObject("Sites");
      }
    }

    static GameLayout? FindRootGameLayout(string name)
    {
      var go = FindRootGameObject(name);
      return go != null ? go.GetComponent<GameLayout>() : null;
    }

    static GameObject? FindRootGameObject(string name)
    {
      for (var i = 0; i < SceneManager.sceneCount; i++)
      {
        var scene = SceneManager.GetSceneAt(i);
        if (!scene.isLoaded)
        {
          continue;
        }

        foreach (var rootGo in scene.GetRootGameObjects())
        {
          if (rootGo.name == name)
          {
            return rootGo;
          }
        }
      }

      return null;
    }

    void OnGUI()
    {
      EditorGUILayout.LabelField("Test Code Generator", EditorStyles.boldLabel);

      DrawGameLayoutSection();
      EditorGUILayout.Space(20);
      DrawSitesSection();
      EditorGUILayout.Space(20);
      DrawGenerateButton();
    }

    void DrawGameLayoutSection()
    {
      EditorGUILayout.LabelField("Game Layouts", EditorStyles.boldLabel);
      EditorGUILayout.Space();

      _portraitLayout =
        EditorGUILayout.ObjectField(
          "Portrait GameLayout",
          _portraitLayout,
          typeof(GameLayout),
          true
        ) as GameLayout;
      _portraitClassName = EditorGUILayout.TextField("Portrait Class Name", _portraitClassName);

      EditorGUILayout.Space();

      _landscapeLayout =
        EditorGUILayout.ObjectField(
          "Landscape GameLayout",
          _landscapeLayout,
          typeof(GameLayout),
          true
        ) as GameLayout;
      _landscapeClassName = EditorGUILayout.TextField("Landscape Class Name", _landscapeClassName);
    }

    void DrawSitesSection()
    {
      EditorGUILayout.LabelField("Sites", EditorStyles.boldLabel);
      EditorGUILayout.Space();

      _sitesRoot =
        EditorGUILayout.ObjectField("Sites Root GameObject", _sitesRoot, typeof(GameObject), true)
        as GameObject;
      _sitesClassName = EditorGUILayout.TextField("Sites Class Name", _sitesClassName);
    }

    void DrawGenerateButton()
    {
      var hasLayoutInput = _portraitLayout != null || _landscapeLayout != null;
      var hasSitesInput = _sitesRoot != null;

      if (!hasLayoutInput && !hasSitesInput)
      {
        EditorGUILayout.HelpBox(
          "Select at least one GameLayout or Sites Root from the scene.",
          MessageType.Info
        );
        return;
      }

      if (_portraitLayout != null && string.IsNullOrWhiteSpace(_portraitClassName))
      {
        EditorGUILayout.HelpBox("Please enter a portrait class name.", MessageType.Warning);
        return;
      }

      if (_landscapeLayout != null && string.IsNullOrWhiteSpace(_landscapeClassName))
      {
        EditorGUILayout.HelpBox("Please enter a landscape class name.", MessageType.Warning);
        return;
      }

      if (_sitesRoot != null && string.IsNullOrWhiteSpace(_sitesClassName))
      {
        EditorGUILayout.HelpBox("Please enter a sites class name.", MessageType.Warning);
        return;
      }

      if (GUILayout.Button("Generate Code"))
      {
        var generatedFiles = new List<string>();

        if (_portraitLayout != null)
        {
          GenerateGameLayoutCode(_portraitLayout, _portraitClassName);
          generatedFiles.Add(_portraitClassName);
        }

        if (_landscapeLayout != null)
        {
          GenerateGameLayoutCode(_landscapeLayout, _landscapeClassName);
          generatedFiles.Add(_landscapeClassName);
        }

        if (_sitesRoot != null)
        {
          GenerateSitesCode(_sitesRoot, _sitesClassName);
          generatedFiles.Add(_sitesClassName);
        }

        Debug.Log($"Generated {generatedFiles.Count} files: {string.Join(", ", generatedFiles)}");
      }
    }

    static bool IsGameLayoutSupportedComponent(Component component)
    {
      if (component == null)
      {
        return false;
      }

      var type = component.GetType();

      if (type == typeof(GameLayout))
      {
        return true;
      }

      if (typeof(ObjectLayout).IsAssignableFrom(type))
      {
        return true;
      }

      if (type == typeof(StaticGameContext))
      {
        return true;
      }

      if (type == typeof(CardOrderSelector))
      {
        return true;
      }

      return false;
    }

    static bool IsSitesSupportedComponent(Component component)
    {
      if (component == null)
      {
        return false;
      }

      var type = component.GetType();

      if (typeof(DreamscapeSite).IsAssignableFrom(type))
      {
        return true;
      }

      if (typeof(ObjectLayout).IsAssignableFrom(type))
      {
        return true;
      }

      return false;
    }

    static void GenerateGameLayoutCode(GameLayout layout, string className)
    {
      var utils = new CodeGeneratorUtils(IsGameLayoutSupportedComponent);
      var builder = CodeGeneratorUtils.CreateBuilder(layout.gameObject.name);

      builder.Class(className);
      builder.OpenBrace();

      builder.Method("GameLayout", "Create", "List<GameObject> createdObjects", isStatic: true);
      builder.OpenBrace();

      var layoutVar = utils.GenerateGameObjectAndComponents(
        builder,
        layout.gameObject,
        "layout",
        isRoot: true
      );

      utils.GenerateComponentReferences(builder, layout, layoutVar);

      builder.BlankLine();
      builder.Return(layoutVar);

      builder.CloseBrace();
      builder.CloseBrace();

      CodeGeneratorUtils.WriteFile(builder, className);
    }

    static void GenerateSitesCode(GameObject sitesRoot, string className)
    {
      var utils = new CodeGeneratorUtils(IsSitesSupportedComponent);
      var builder = CodeGeneratorUtils.CreateBuilder(sitesRoot.name);

      builder.Class(className);
      builder.OpenBrace();

      builder.Method(
        "List<DreamscapeSite>",
        "Create",
        "List<GameObject> createdObjects",
        isStatic: true
      );
      builder.OpenBrace();

      builder.Var("result", "new List<DreamscapeSite>()");
      builder.BlankLine();

      var siteComponents = sitesRoot.GetComponentsInChildren<DreamscapeSite>(includeInactive: true);

      foreach (var site in siteComponents)
      {
        var siteVar = utils.GenerateGameObjectAndComponents(
          builder,
          site.gameObject,
          CodeGeneratorUtils.SanitizeVarName(site.gameObject.name),
          isRoot: true
        );

        utils.GenerateComponentReferences(builder, site, siteVar);

        builder.Call("result", "Add", siteVar);
        builder.BlankLine();
      }

      builder.Return("result");

      builder.CloseBrace();
      builder.CloseBrace();

      CodeGeneratorUtils.WriteFile(builder, className);
    }
  }
}
