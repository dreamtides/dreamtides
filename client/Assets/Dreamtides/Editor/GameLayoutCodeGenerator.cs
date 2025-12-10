#nullable enable

using System;
using System.Collections.Generic;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Services;
using UnityEditor;
using UnityEngine;
using UnityEngine.SceneManagement;
using UnityEngine.UI;

namespace Dreamtides.Editors
{
  public class GameLayoutCodeGenerator : EditorWindow
  {
    BattleLayout? _portraitLayout;
    BattleLayout? _landscapeLayout;
    string _portraitClassName = "GeneratedPortraitGameLayout";
    string _landscapeClassName = "GeneratedLandscapeGameLayout";

    GameObject? _sitesRoot;
    string _sitesClassName = "GeneratedSites";

    Canvas? _canvas;
    string _canvasClassName = "GeneratedCanvas";

    Camera? _mainCamera;
    string _mainCameraClassName = "GeneratedMainCamera";

    Registry? _registry;
    string _registryClassName = "GeneratedRegistry";

    static readonly Dictionary<Type, string?> ServicesToFake = new()
    {
      { typeof(ActionService), "FakeActionService" },
      { typeof(SoundService), "FakeSoundService" },
      { typeof(LoggingService), "FakeLoggingService" },
      { typeof(StudioService), null },
      { typeof(MusicService), null },
      { typeof(IdleReconnectService), null },
      { typeof(DreamscapeService), null },
      { typeof(PrototypeQuest), null },
    };

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

      if (_mainCamera == null)
      {
        _mainCamera = FindRootCamera("MainCamera");
      }

      if (_registry == null)
      {
        _registry = FindRootRegistry("Registry");
      }
    }

    static Registry? FindRootRegistry(string name)
    {
      var go = FindRootGameObject(name);
      return go != null ? go.GetComponent<Registry>() : null;
    }

    static Camera? FindRootCamera(string name)
    {
      var go = FindRootGameObject(name);
      return go != null ? go.GetComponent<Camera>() : null;
    }

    static Canvas? FindRootCanvas(string name)
    {
      var go = FindRootGameObject(name);
      return go != null ? go.GetComponent<Canvas>() : null;
    }

    static BattleLayout? FindRootGameLayout(string name)
    {
      var go = FindRootGameObject(name);
      return go != null ? go.GetComponent<BattleLayout>() : null;
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

      DrawRegistrySection();
      EditorGUILayout.Space(20);
      DrawGameLayoutSection();
      EditorGUILayout.Space(20);
      DrawSitesSection();
      EditorGUILayout.Space(20);
      DrawCanvasSection();
      EditorGUILayout.Space(20);
      DrawMainCameraSection();
      EditorGUILayout.Space(20);
      DrawGenerateButton();
    }

    void DrawRegistrySection()
    {
      EditorGUILayout.LabelField("Registry", EditorStyles.boldLabel);
      EditorGUILayout.Space();

      _registry =
        EditorGUILayout.ObjectField("Registry", _registry, typeof(Registry), true) as Registry;
      _registryClassName = EditorGUILayout.TextField("Registry Class Name", _registryClassName);
    }

    void DrawGameLayoutSection()
    {
      EditorGUILayout.LabelField("Game Layouts", EditorStyles.boldLabel);
      EditorGUILayout.Space();

      _portraitLayout =
        EditorGUILayout.ObjectField(
          "Portrait GameLayout",
          _portraitLayout,
          typeof(BattleLayout),
          true
        ) as BattleLayout;
      _portraitClassName = EditorGUILayout.TextField("Portrait Class Name", _portraitClassName);

      EditorGUILayout.Space();

      _landscapeLayout =
        EditorGUILayout.ObjectField(
          "Landscape GameLayout",
          _landscapeLayout,
          typeof(BattleLayout),
          true
        ) as BattleLayout;
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

    void DrawMainCameraSection()
    {
      EditorGUILayout.LabelField("Main Camera", EditorStyles.boldLabel);
      EditorGUILayout.Space();

      _mainCamera =
        EditorGUILayout.ObjectField("Main Camera", _mainCamera, typeof(Camera), true) as Camera;
      _mainCameraClassName = EditorGUILayout.TextField(
        "Main Camera Class Name",
        _mainCameraClassName
      );
    }

    void DrawGenerateButton()
    {
      var hasLayoutInput = _portraitLayout != null || _landscapeLayout != null;
      var hasSitesInput = _sitesRoot != null;
      var hasCanvasInput = _canvas != null;
      var hasMainCameraInput = _mainCamera != null;
      var hasRegistryInput = _registry != null;

      if (
        !hasLayoutInput
        && !hasSitesInput
        && !hasCanvasInput
        && !hasMainCameraInput
        && !hasRegistryInput
      )
      {
        EditorGUILayout.HelpBox(
          "Select at least one Registry, GameLayout, Sites Root, Canvas, or Main Camera from the scene.",
          MessageType.Info
        );
        return;
      }

      if (_registry != null && string.IsNullOrWhiteSpace(_registryClassName))
      {
        EditorGUILayout.HelpBox("Please enter a registry class name.", MessageType.Warning);
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

      if (_mainCamera != null && string.IsNullOrWhiteSpace(_mainCameraClassName))
      {
        EditorGUILayout.HelpBox("Please enter a main camera class name.", MessageType.Warning);
        return;
      }

      if (GUILayout.Button("Generate Code"))
      {
        var generatedFiles = new List<string>();

        if (_registry != null)
        {
          GenerateRegistryCode(_registry, _registryClassName);
          generatedFiles.Add(_registryClassName);
        }

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

        if (_mainCamera != null)
        {
          GenerateMainCameraCode(_mainCamera, _mainCameraClassName);
          generatedFiles.Add(_mainCameraClassName);
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

      if (type == typeof(BattleLayout))
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

    static bool IsMainCameraSupportedComponent(Component component)
    {
      if (component == null)
      {
        return false;
      }

      var type = component.GetType();

      if (typeof(ObjectLayout).IsAssignableFrom(type))
      {
        return true;
      }

      if (type == typeof(GameCamera))
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

    void GenerateGameLayoutCode(BattleLayout layout, string className)
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

    static void GenerateMainCameraCode(Camera mainCamera, string className)
    {
      var utils = new CodeGeneratorUtils(IsMainCameraSupportedComponent);
      var builder = CodeGeneratorUtils.CreateBuilder(mainCamera.gameObject.name);

      builder.Class(className);
      builder.OpenBrace();

      builder.Line("public GameCamera GameCamera { get; private set; } = null!;");
      builder.BlankLine();

      builder.Method(className, "Create", "List<GameObject> createdObjects", isStatic: true);
      builder.OpenBrace();

      builder.Var("result", $"new {className}()");
      builder.BlankLine();

      var cameraGo = mainCamera.gameObject;
      var cameraVar = utils.GenerateGameObjectAndComponents(
        builder,
        cameraGo,
        "mainCamera",
        isRoot: true
      );

      var gameCamera = cameraGo.GetComponent<GameCamera>();
      if (gameCamera != null)
      {
        builder.Assign("result.GameCamera", $"{cameraVar}Go.GetComponent<GameCamera>()");
      }

      GenerateMainCameraChildren(builder, utils, cameraGo.transform);

      builder.BlankLine();
      builder.Return("result");

      builder.CloseBrace();
      builder.CloseBrace();

      CodeGeneratorUtils.WriteFile(builder, className);
    }

    static void GenerateMainCameraChildren(
      CSharpCodeBuilder builder,
      CodeGeneratorUtils utils,
      Transform parent
    )
    {
      foreach (Transform child in parent)
      {
        builder.BlankLine();
        utils.GenerateGameObjectAndComponents(
          builder,
          child.gameObject,
          CodeGeneratorUtils.SanitizeVarName(child.gameObject.name),
          isRoot: false
        );

        GenerateMainCameraChildren(builder, utils, child);
      }
    }

    static void GenerateRegistryCode(Registry registry, string className)
    {
      var builder = CodeGeneratorUtils.CreateBuilder(registry.gameObject.name);

      builder.Using("Dreamtides.Services");
      builder.Using("Dreamtides.TestFakes");
      builder.BlankLine();

      builder.Class(className);
      builder.OpenBrace();

      builder.Line("public Registry Registry { get; private set; } = null!;");
      builder.Line("public FakeSoundService FakeSoundService { get; private set; } = null!;");
      builder.Line("public FakeActionService FakeActionService { get; private set; } = null!;");
      builder.BlankLine();

      builder.Method(
        className,
        "Create",
        "List<GameObject> createdObjects, GeneratedCanvas canvas, GeneratedMainCamera mainCamera, GameLayout portraitLayout, GameLayout landscapeLayout",
        isStatic: true
      );
      builder.OpenBrace();

      builder.Var("result", $"new {className}()");
      builder.BlankLine();

      builder.Var("registryGo", $"new GameObject(\"{registry.gameObject.name}\")");
      builder.Call("createdObjects", "Add", "registryGo");
      builder.Var("registryComponent", "registryGo.AddComponent<Registry>()");
      builder.Assign("result.Registry", "registryComponent");
      builder.BlankLine();

      builder.Assign("registryComponent._canvas", "canvas.Canvas");
      builder.Assign("registryComponent._cameraAdjuster", "mainCamera.GameCamera");
      builder.Assign("registryComponent._portraitLayout", "portraitLayout");
      builder.Assign("registryComponent._landscapeLayout", "landscapeLayout");
      builder.BlankLine();

      builder.Var("mainAudioSource", "registryGo.AddComponent<AudioSource>()");
      builder.Assign("registryComponent._mainAudioSource", "mainAudioSource");
      builder.Var("musicAudioSource", "registryGo.AddComponent<AudioSource>()");
      builder.Assign("registryComponent._musicAudioSource", "musicAudioSource");
      builder.BlankLine();

      builder.Var("canvasSafeArea", "canvas.Canvas.GetComponentInChildren<RectTransform>()");
      builder.Assign("registryComponent._canvasSafeArea", "canvasSafeArea");
      builder.BlankLine();

      var serialized = new SerializedObject(registry);
      var iterator = serialized.GetIterator();
      var enterChildren = true;

      while (iterator.NextVisible(enterChildren))
      {
        enterChildren = false;

        if (iterator.propertyType != SerializedPropertyType.ObjectReference)
        {
          continue;
        }

        var obj = iterator.objectReferenceValue;
        if (obj == null)
        {
          continue;
        }

        if (obj is not Service service)
        {
          continue;
        }

        var serviceType = service.GetType();
        var fieldName = iterator.name;

        if (ServicesToFake.TryGetValue(serviceType, out var fakeClassName))
        {
          if (fakeClassName == null)
          {
            continue;
          }

          builder.Var(
            CodeGeneratorUtils.SanitizeVarName(fieldName),
            $"registryGo.AddComponent<{fakeClassName}>()"
          );
          builder.Assign(
            $"registryComponent.{fieldName}",
            CodeGeneratorUtils.SanitizeVarName(fieldName)
          );

          if (fakeClassName == "FakeSoundService")
          {
            builder.Assign(
              "result.FakeSoundService",
              CodeGeneratorUtils.SanitizeVarName(fieldName)
            );
          }
          else if (fakeClassName == "FakeActionService")
          {
            builder.Assign(
              "result.FakeActionService",
              CodeGeneratorUtils.SanitizeVarName(fieldName)
            );
          }
        }
        else
        {
          builder.Var(
            CodeGeneratorUtils.SanitizeVarName(fieldName),
            $"registryGo.AddComponent<{serviceType.Name}>()"
          );
          builder.Assign(
            $"registryComponent.{fieldName}",
            CodeGeneratorUtils.SanitizeVarName(fieldName)
          );
        }
      }

      builder.BlankLine();
      builder.Return("result");

      builder.CloseBrace();
      builder.CloseBrace();

      CodeGeneratorUtils.WriteFile(builder, className);
    }
  }
}
