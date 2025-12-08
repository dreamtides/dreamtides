#nullable enable

using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using Dreamtides.Layout;
using UnityEditor;
using UnityEngine;
using Object = UnityEngine.Object;

namespace Dreamtides.Editors
{
  public class GameLayoutCodeGenerator : EditorWindow
  {
    const string OutputDirectory = "Assets/Dreamtides/Tests/TestUtils/";

    static readonly HashSet<string> SkipFields = new()
    {
      "_registry",
      "_objects",
      "m_Script",
      "m_GameObject",
      "m_Enabled",
      "m_ObjectHideFlags",
    };

    GameLayout? _portraitLayout;
    GameLayout? _landscapeLayout;
    string _portraitClassName = "GeneratedPortraitGameLayout";
    string _landscapeClassName = "GeneratedLandscapeGameLayout";
    readonly Dictionary<GameObject, string> _goVariables = new();
    readonly Dictionary<Component, string> _componentVariables = new();
    readonly HashSet<string> _usedVarNames = new();

    [MenuItem("Tools/Generate GameLayout Test Code")]
    static void ShowWindow()
    {
      GetWindow<GameLayoutCodeGenerator>("GameLayout Code Generator");
    }

    void OnGUI()
    {
      EditorGUILayout.LabelField("GameLayout Code Generator", EditorStyles.boldLabel);
      EditorGUILayout.Space();

      EditorGUILayout.LabelField("Portrait Layout", EditorStyles.boldLabel);
      _portraitLayout =
        EditorGUILayout.ObjectField(
          "Portrait GameLayout",
          _portraitLayout,
          typeof(GameLayout),
          true
        ) as GameLayout;
      _portraitClassName = EditorGUILayout.TextField("Portrait Class Name", _portraitClassName);

      EditorGUILayout.Space();

      EditorGUILayout.LabelField("Landscape Layout", EditorStyles.boldLabel);
      _landscapeLayout =
        EditorGUILayout.ObjectField(
          "Landscape GameLayout",
          _landscapeLayout,
          typeof(GameLayout),
          true
        ) as GameLayout;
      _landscapeClassName = EditorGUILayout.TextField("Landscape Class Name", _landscapeClassName);

      EditorGUILayout.Space();

      if (_portraitLayout == null && _landscapeLayout == null)
      {
        EditorGUILayout.HelpBox("Select at least one GameLayout from the scene.", MessageType.Info);
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

      if (GUILayout.Button("Generate Code"))
      {
        var generatedFiles = new List<string>();

        if (_portraitLayout != null)
        {
          GenerateCode(_portraitLayout, _portraitClassName);
          generatedFiles.Add(_portraitClassName);
        }

        if (_landscapeLayout != null)
        {
          GenerateCode(_landscapeLayout, _landscapeClassName);
          generatedFiles.Add(_landscapeClassName);
        }

        Debug.Log(
          $"Generated {generatedFiles.Count} layout files: {string.Join(", ", generatedFiles)}"
        );
      }
    }

    static bool IsSupportedComponent(Component component)
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

    void GenerateCode(GameLayout layout, string className)
    {
      _goVariables.Clear();
      _componentVariables.Clear();
      _usedVarNames.Clear();

      var builder = new CSharpCodeBuilder();

      builder.Line("// AUTO-GENERATED CODE - DO NOT EDIT");
      builder.Line($"// Generated from: {layout.gameObject.name}");
      builder.Line($"// Generated at: {DateTime.Now:yyyy-MM-dd HH:mm:ss}");
      builder.BlankLine();
      builder.Line("#nullable enable");
      builder.BlankLine();
      builder.Using("System.Collections.Generic");
      builder.Using("Dreamtides.Buttons");
      builder.Using("Dreamtides.Components");
      builder.Using("Dreamtides.Layout");
      builder.Using("Dreamtides.Schema");
      builder.Using("TMPro");
      builder.Using("Unity.Cinemachine");
      builder.Using("UnityEngine");
      builder.Using("UnityEngine.UI");
      builder.BlankLine();
      builder.Namespace("Dreamtides.Tests.TestUtils");
      builder.OpenBrace();

      builder.Class(className);
      builder.OpenBrace();

      GenerateCreateMethod(builder, layout);

      builder.CloseBrace();
      builder.CloseBrace();

      var code = builder.ToString();
      var outputPath = $"{OutputDirectory}{className}.cs";
      File.WriteAllText(outputPath, code);
      AssetDatabase.Refresh();

      Debug.Log($"Generated GameLayout code to {outputPath}");
    }

    void GenerateCreateMethod(CSharpCodeBuilder builder, GameLayout layout)
    {
      builder.Method("GameLayout", "Create", "List<GameObject> createdObjects", isStatic: true);
      builder.OpenBrace();

      var layoutVar = GenerateGameObjectAndComponents(
        builder,
        layout.gameObject,
        "layout",
        isRoot: true
      );

      GenerateChildReferences(builder, layout, layoutVar);

      builder.BlankLine();
      builder.Return(layoutVar);

      builder.CloseBrace();
    }

    string GenerateGameObjectAndComponents(
      CSharpCodeBuilder builder,
      GameObject go,
      string suggestedName,
      bool isRoot = false
    )
    {
      if (_goVariables.TryGetValue(go, out var existingGoVar))
      {
        var primaryComponent = GetPrimaryComponent(go);
        if (
          primaryComponent != null
          && _componentVariables.TryGetValue(primaryComponent, out var existingCompVar)
        )
        {
          return existingCompVar;
        }
        return existingGoVar.Replace("Go", "");
      }

      var baseVarName = GenerateUniqueVarName(suggestedName);
      var goVarName = baseVarName + "Go";

      _goVariables[go] = goVarName;

      builder.CreateGameObject(goVarName, go.name);
      builder.Call("createdObjects", "Add", goVarName);

      GenerateTransform(builder, go.transform, goVarName, isRoot);

      var primaryComponentVar = GenerateComponentsOnGameObject(builder, go, goVarName, baseVarName);

      return primaryComponentVar ?? baseVarName;
    }

    string? GenerateComponentsOnGameObject(
      CSharpCodeBuilder builder,
      GameObject go,
      string goVarName,
      string baseVarName
    )
    {
      var components = go.GetComponents<Component>();
      var supportedComponents = components.Where(IsSupportedComponent).ToList();

      string? primaryVar = null;
      var componentIndex = 0;

      foreach (var component in supportedComponents)
      {
        var componentType = component.GetType();
        string componentVar;

        if (componentIndex == 0)
        {
          componentVar = baseVarName;
        }
        else
        {
          componentVar = GenerateUniqueVarName(baseVarName + componentType.Name);
        }

        _componentVariables[component] = componentVar;

        builder.AddComponent(componentVar, goVarName, componentType.Name);
        GenerateNonDefaultFields(builder, component, componentVar);

        if (primaryVar == null)
        {
          primaryVar = componentVar;
        }

        componentIndex++;
      }

      return primaryVar;
    }

    Component? GetPrimaryComponent(GameObject go)
    {
      var components = go.GetComponents<Component>();
      return components.FirstOrDefault(IsSupportedComponent);
    }

    void GenerateTransform(CSharpCodeBuilder builder, Transform t, string goVar, bool isRoot)
    {
      if (!isRoot && t.parent != null)
      {
        if (_goVariables.TryGetValue(t.parent.gameObject, out var parentGoVar))
        {
          builder.Call($"{goVar}.transform", "SetParent", $"{parentGoVar}.transform", "false");
        }
      }

      if (t.localPosition != Vector3.zero)
      {
        builder.Assign(
          $"{goVar}.transform.localPosition",
          CSharpCodeBuilder.ToVector3(t.localPosition)
        );
      }

      if (t.localRotation != Quaternion.identity)
      {
        builder.Assign(
          $"{goVar}.transform.localRotation",
          CSharpCodeBuilder.ToQuaternion(t.localRotation)
        );
      }

      if (t.localScale != Vector3.one)
      {
        builder.Assign($"{goVar}.transform.localScale", CSharpCodeBuilder.ToVector3(t.localScale));
      }
    }

    void GenerateNonDefaultFields(
      CSharpCodeBuilder builder,
      Component component,
      string componentVar
    )
    {
      var componentType = component.GetType();

      var tempGo = new GameObject("__temp_default_check");
      tempGo.hideFlags = HideFlags.HideAndDontSave;

      try
      {
        var defaultComponent = tempGo.AddComponent(componentType);

        var serializedActual = new SerializedObject(component);
        var serializedDefault = new SerializedObject(defaultComponent);

        var iterator = serializedActual.GetIterator();
        var enterChildren = true;
        while (iterator.NextVisible(enterChildren))
        {
          enterChildren = false;

          if (ShouldSkipProperty(iterator))
          {
            continue;
          }

          var defaultProp = serializedDefault.FindProperty(iterator.propertyPath);
          if (defaultProp == null)
          {
            continue;
          }

          if (!SerializedProperty.DataEquals(iterator, defaultProp))
          {
            GenerateFieldAssignment(builder, iterator, componentVar);
          }
        }
      }
      finally
      {
        DestroyImmediate(tempGo);
      }
    }

    bool ShouldSkipProperty(SerializedProperty prop)
    {
      if (SkipFields.Contains(prop.name))
      {
        return true;
      }

      if (prop.propertyPath.Contains("."))
      {
        return true;
      }

      return false;
    }

    void GenerateFieldAssignment(
      CSharpCodeBuilder builder,
      SerializedProperty prop,
      string componentVar
    )
    {
      var fieldName = prop.name;
      var target = $"{componentVar}.{fieldName}";

      switch (prop.propertyType)
      {
        case SerializedPropertyType.Float:
          builder.Assign(target, CSharpCodeBuilder.ToLiteral(prop.floatValue));
          break;

        case SerializedPropertyType.Integer:
          builder.Assign(target, CSharpCodeBuilder.ToLiteral(prop.intValue));
          break;

        case SerializedPropertyType.Boolean:
          builder.Assign(target, CSharpCodeBuilder.ToLiteral(prop.boolValue));
          break;

        case SerializedPropertyType.String:
          if (!string.IsNullOrEmpty(prop.stringValue))
          {
            builder.Assign(target, CSharpCodeBuilder.ToLiteral(prop.stringValue));
          }
          break;

        case SerializedPropertyType.Enum:
          var enumType = GetEnumType(prop);
          if (
            enumType != null
            && prop.enumValueIndex >= 0
            && prop.enumValueIndex < prop.enumNames.Length
          )
          {
            var enumName = prop.enumNames[prop.enumValueIndex];
            builder.Assign(target, $"{enumType.Name}.{enumName}");
          }
          break;

        case SerializedPropertyType.Vector3:
          builder.Assign(target, CSharpCodeBuilder.ToVector3(prop.vector3Value));
          break;

        case SerializedPropertyType.Color:
          builder.Assign(target, CSharpCodeBuilder.ToColor(prop.colorValue));
          break;

        case SerializedPropertyType.ObjectReference:
          break;
      }
    }

    void GenerateChildReferences(CSharpCodeBuilder builder, GameLayout layout, string layoutVar)
    {
      var serialized = new SerializedObject(layout);
      var iterator = serialized.GetIterator();
      var enterChildren = true;

      var childRefs = new List<(string fieldName, Object objRef)>();

      while (iterator.NextVisible(enterChildren))
      {
        enterChildren = false;

        if (iterator.propertyType != SerializedPropertyType.ObjectReference)
        {
          continue;
        }

        if (ShouldSkipProperty(iterator))
        {
          continue;
        }

        var objRef = iterator.objectReferenceValue;
        if (objRef == null)
        {
          continue;
        }

        childRefs.Add((iterator.name, objRef));
      }

      foreach (var (fieldName, objRef) in childRefs)
      {
        GenerateChildReference(builder, objRef, layoutVar, fieldName);
      }
    }

    void GenerateChildReference(
      CSharpCodeBuilder builder,
      Object objRef,
      string parentVar,
      string fieldName
    )
    {
      GameObject? go = null;
      Component? targetComponent = null;

      if (objRef is Component c)
      {
        go = c.gameObject;
        targetComponent = c;
      }
      else if (objRef is GameObject g)
      {
        go = g;
      }
      else if (objRef is Transform t)
      {
        go = t.gameObject;
        targetComponent = t;
      }
      else
      {
        Debug.LogWarning($"Skipping unsupported asset reference: {fieldName} -> {objRef.name}");
        return;
      }

      if (go == null)
      {
        return;
      }

      if (_goVariables.ContainsKey(go))
      {
        EnsureComponentExists(builder, go, targetComponent);
        AssignExistingReference(builder, parentVar, fieldName, go, targetComponent);
        return;
      }

      builder.BlankLine();

      var childVar = GenerateGameObjectAndComponents(
        builder,
        go,
        SanitizeVarName(fieldName),
        isRoot: false
      );

      EnsureComponentExists(builder, go, targetComponent);

      GenerateNestedChildReferences(builder, go, childVar);

      AssignExistingReference(builder, parentVar, fieldName, go, targetComponent);
    }

    void EnsureComponentExists(CSharpCodeBuilder builder, GameObject go, Component? targetComponent)
    {
      if (targetComponent == null || targetComponent is Transform)
      {
        return;
      }

      if (_componentVariables.ContainsKey(targetComponent))
      {
        return;
      }

      if (!_goVariables.TryGetValue(go, out var goVar))
      {
        return;
      }

      var componentType = targetComponent.GetType();
      var componentVar = GenerateUniqueVarName(SanitizeVarName(componentType.Name));
      _componentVariables[targetComponent] = componentVar;

      builder.AddComponent(componentVar, goVar, componentType.Name);

      if (IsUserDefinedComponent(targetComponent))
      {
        ProcessComponentChildReferences(builder, targetComponent, componentVar);
      }
    }

    static bool IsUserDefinedComponent(Component component)
    {
      var ns = component.GetType().Namespace;
      if (string.IsNullOrEmpty(ns))
      {
        return false;
      }

      return ns.StartsWith("Dreamtides");
    }

    void ProcessComponentChildReferences(
      CSharpCodeBuilder builder,
      Component component,
      string componentVar
    )
    {
      var serialized = new SerializedObject(component);
      var iterator = serialized.GetIterator();
      var enterChildren = true;

      while (iterator.NextVisible(enterChildren))
      {
        enterChildren = false;

        if (iterator.propertyType != SerializedPropertyType.ObjectReference)
        {
          continue;
        }

        if (ShouldSkipProperty(iterator))
        {
          continue;
        }

        var objRef = iterator.objectReferenceValue;
        if (objRef == null)
        {
          continue;
        }

        GameObject? childGo = null;
        Component? childComponent = null;

        if (objRef is Component c)
        {
          childGo = c.gameObject;
          childComponent = c;
        }
        else if (objRef is Transform t)
        {
          childGo = t.gameObject;
          childComponent = t;
        }

        if (childGo == null)
        {
          continue;
        }

        if (!_goVariables.ContainsKey(childGo))
        {
          GenerateChildReference(builder, objRef, componentVar, iterator.name);
        }
        else
        {
          EnsureComponentExists(builder, childGo, childComponent);
          AssignExistingReference(builder, componentVar, iterator.name, childGo, childComponent);
        }
      }
    }

    void GenerateNestedChildReferences(CSharpCodeBuilder builder, GameObject go, string goVar)
    {
      var components = go.GetComponents<Component>();
      foreach (var component in components)
      {
        if (!IsSupportedComponent(component))
        {
          continue;
        }

        var serialized = new SerializedObject(component);
        var iterator = serialized.GetIterator();
        var enterChildren = true;

        while (iterator.NextVisible(enterChildren))
        {
          enterChildren = false;

          if (iterator.propertyType != SerializedPropertyType.ObjectReference)
          {
            continue;
          }

          if (ShouldSkipProperty(iterator))
          {
            continue;
          }

          var objRef = iterator.objectReferenceValue;
          if (objRef == null)
          {
            continue;
          }

          GameObject? childGo = null;
          if (objRef is Component c)
          {
            childGo = c.gameObject;
          }
          else if (objRef is Transform t)
          {
            childGo = t.gameObject;
          }

          if (childGo == null)
          {
            continue;
          }

          if (!_componentVariables.TryGetValue(component, out var compVar))
          {
            compVar = goVar;
          }

          if (!_goVariables.ContainsKey(childGo))
          {
            GenerateChildReference(builder, objRef, compVar, iterator.name);
          }
          else
          {
            AssignExistingReference(builder, compVar, iterator.name, childGo, objRef as Component);
          }
        }
      }
    }

    void AssignExistingReference(
      CSharpCodeBuilder builder,
      string parentVar,
      string fieldName,
      GameObject go,
      Component? targetComponent
    )
    {
      var target = $"{parentVar}.{fieldName}";
      var goVar = _goVariables[go];

      if (targetComponent is Transform)
      {
        builder.Assign(target, $"{goVar}.transform");
      }
      else if (
        targetComponent != null
        && _componentVariables.TryGetValue(targetComponent, out var compVar)
      )
      {
        builder.Assign(target, compVar);
      }
      else if (targetComponent != null)
      {
        var componentTypeName = targetComponent.GetType().Name;
        builder.Assign(target, $"{goVar}.GetComponent<{componentTypeName}>()");
      }
      else
      {
        builder.Assign(target, goVar);
      }
    }

    string GenerateUniqueVarName(string baseName)
    {
      var sanitized = SanitizeVarName(baseName);
      var candidate = sanitized;
      var counter = 1;

      while (_usedVarNames.Contains(candidate))
      {
        candidate = sanitized + counter;
        counter++;
      }

      _usedVarNames.Add(candidate);
      return candidate;
    }

    static string SanitizeVarName(string name)
    {
      var result = name.TrimStart('_');
      if (result.Length > 0)
      {
        result = char.ToLower(result[0]) + result.Substring(1);
      }
      return result;
    }

    static Type? GetEnumType(SerializedProperty prop)
    {
      var targetObject = prop.serializedObject.targetObject;
      var targetType = targetObject.GetType();

      while (targetType != null)
      {
        var field = targetType.GetField(
          prop.name,
          BindingFlags.NonPublic | BindingFlags.Public | BindingFlags.Instance
        );

        if (field != null && field.FieldType.IsEnum)
        {
          return field.FieldType;
        }

        targetType = targetType.BaseType;
      }

      return null;
    }
  }
}
