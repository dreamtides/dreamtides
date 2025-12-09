#nullable enable

using System.Collections.Generic;
using Dreamtides.Components;
using Dreamtides.Layout;
using UnityEditor;
using UnityEngine;
using UnityEngine.SceneManagement;
using UnityEngine.UI;

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

    Canvas? _canvas;
    string _canvasClassName = "GeneratedCanvas";

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

      if (_canvas == null)
      {
        _canvas = FindRootCanvas("Canvas");
      }
    }

    static Canvas? FindRootCanvas(string name)
    {
      var go = FindRootGameObject(name);
      return go != null ? go.GetComponent<Canvas>() : null;
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
      DrawCanvasSection();
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

    void DrawCanvasSection()
    {
      EditorGUILayout.LabelField("Canvas", EditorStyles.boldLabel);
      EditorGUILayout.Space();

      _canvas = EditorGUILayout.ObjectField("Canvas", _canvas, typeof(Canvas), true) as Canvas;
      _canvasClassName = EditorGUILayout.TextField("Canvas Class Name", _canvasClassName);
    }

    void DrawGenerateButton()
    {
      var hasLayoutInput = _portraitLayout != null || _landscapeLayout != null;
      var hasSitesInput = _sitesRoot != null;
      var hasCanvasInput = _canvas != null;

      if (!hasLayoutInput && !hasSitesInput && !hasCanvasInput)
      {
        EditorGUILayout.HelpBox(
          "Select at least one GameLayout, Sites Root, or Canvas from the scene.",
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

      if (_canvas != null && string.IsNullOrWhiteSpace(_canvasClassName))
      {
        EditorGUILayout.HelpBox("Please enter a canvas class name.", MessageType.Warning);
        return;
      }

      if (GUILayout.Button("Generate Code"))
      {
        var generatedFiles = new List<string>();

        if (_canvas != null)
        {
          GenerateCanvasCode(_canvas, _canvasClassName);
          generatedFiles.Add(_canvasClassName);
        }

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

    static bool IsCanvasSupportedComponent(Component component)
    {
      if (component == null)
      {
        return false;
      }

      var type = component.GetType();

      if (type == typeof(Canvas))
      {
        return true;
      }

      if (type == typeof(CanvasScaler))
      {
        return true;
      }

      if (type == typeof(GraphicRaycaster))
      {
        return true;
      }

      return false;
    }

    static HashSet<GameObject> GetCanvasDescendants(Canvas? canvas)
    {
      var result = new HashSet<GameObject>();
      if (canvas == null)
      {
        return result;
      }

      CollectDescendants(canvas.transform, result);
      return result;
    }

    static void CollectDescendants(Transform parent, HashSet<GameObject> result)
    {
      foreach (Transform child in parent)
      {
        result.Add(child.gameObject);
        CollectDescendants(child, result);
      }
    }

    void GenerateGameLayoutCode(GameLayout layout, string className)
    {
      var canvasObjects = GetCanvasDescendants(_canvas);
      var utils = new CodeGeneratorUtils(IsGameLayoutSupportedComponent, canvasObjects);
      var builder = CodeGeneratorUtils.CreateBuilder(layout.gameObject.name);

      builder.Class(className);
      builder.OpenBrace();

      builder.Method(
        "GameLayout",
        "Create",
        "List<GameObject> createdObjects, GeneratedCanvas? canvas = null",
        isStatic: true
      );
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

    static void GenerateCanvasCode(Canvas canvas, string className)
    {
      var utils = new CodeGeneratorUtils(IsCanvasSupportedComponent);
      var builder = CodeGeneratorUtils.CreateBuilder(canvas.gameObject.name);

      builder.Class(className);
      builder.OpenBrace();

      builder.Line("public Canvas Canvas { get; private set; } = null!;");
      builder.Line(
        "public Dictionary<string, GameObject> Objects { get; } = new Dictionary<string, GameObject>();"
      );
      builder.BlankLine();

      builder.Method(className, "Create", "List<GameObject> createdObjects", isStatic: true);
      builder.OpenBrace();

      builder.Var("result", $"new {className}()");
      builder.BlankLine();

      var canvasGo = canvas.gameObject;
      var canvasVar = utils.GenerateGameObjectAndComponents(
        builder,
        canvasGo,
        "canvas",
        isRoot: true
      );

      builder.Assign("result.Canvas", canvasVar);

      GenerateCanvasChildren(builder, utils, canvasGo.transform, "");

      builder.BlankLine();
      builder.Return("result");

      builder.CloseBrace();
      builder.CloseBrace();

      CodeGeneratorUtils.WriteFile(builder, className);
    }

    static void GenerateCanvasChildren(
      CSharpCodeBuilder builder,
      CodeGeneratorUtils utils,
      Transform parent,
      string parentPath
    )
    {
      foreach (Transform child in parent)
      {
        var childPath = string.IsNullOrEmpty(parentPath)
          ? child.gameObject.name
          : $"{parentPath}/{child.gameObject.name}";

        builder.BlankLine();
        var childVar = utils.GenerateGameObjectAndComponents(
          builder,
          child.gameObject,
          CodeGeneratorUtils.SanitizeVarName(child.gameObject.name),
          isRoot: false
        );

        builder.Call("result.Objects", "Add", $"\"{childPath}\"", $"{childVar}Go");

        GenerateCanvasChildren(builder, utils, child, childPath);
      }
    }
  }
}
