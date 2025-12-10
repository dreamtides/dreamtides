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
  public class CodeGeneratorUtils
  {
    public const string OutputDirectory = "Assets/Dreamtides/Tests/TestUtils/";

    static readonly HashSet<string> SkipFields = new()
    {
      "_registry",
      "_objects",
      "m_Script",
      "m_GameObject",
      "m_Enabled",
      "m_ObjectHideFlags",
    };

    readonly Dictionary<GameObject, string> _goVariables = new();
    readonly Dictionary<Component, string> _componentVariables = new();
    readonly HashSet<string> _usedVarNames = new();
    readonly Func<Component, bool> _isSupportedComponent;
    readonly HashSet<GameObject> _canvasObjects;

    public CodeGeneratorUtils(Func<Component, bool> isSupportedComponent)
      : this(isSupportedComponent, new HashSet<GameObject>()) { }

    public CodeGeneratorUtils(
      Func<Component, bool> isSupportedComponent,
      HashSet<GameObject> canvasObjects
    )
    {
      _isSupportedComponent = isSupportedComponent;
      _canvasObjects = canvasObjects;
    }

    public void Clear()
    {
      _goVariables.Clear();
      _componentVariables.Clear();
      _usedVarNames.Clear();
    }

    public static CSharpCodeBuilder CreateBuilder(string sourceName)
    {
      var builder = new CSharpCodeBuilder();

      builder.Line("// AUTO-GENERATED CODE - DO NOT EDIT");
      builder.Line($"// Generated from: {sourceName}");
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

      return builder;
    }

    public static void WriteFile(CSharpCodeBuilder builder, string className)
    {
      builder.CloseBrace();
      var code = builder.ToString();
      var outputPath = $"{OutputDirectory}{className}.cs";
      File.WriteAllText(outputPath, code);
      AssetDatabase.Refresh();
      Debug.Log($"Generated code to {outputPath}");
    }

    public string GenerateGameObjectAndComponents(
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

    public void GenerateChildReference(
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

      if (_canvasObjects.Contains(go))
      {
        GenerateCanvasObjectReference(builder, go, parentVar, fieldName, targetComponent);
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

    void GenerateCanvasObjectReference(
      CSharpCodeBuilder builder,
      GameObject go,
      string parentVar,
      string fieldName,
      Component? targetComponent
    )
    {
      builder.BlankLine();
      var goVarName = GenerateUniqueVarName(SanitizeVarName(fieldName)) + "Go";
      var path = GetCanvasObjectPath(go);
      builder.Var(goVarName, $"canvas?.Objects[\"{path}\"]");
      _goVariables[go] = goVarName;

      var target = $"{parentVar}.{fieldName}";

      if (targetComponent == null)
      {
        builder.Line($"if ({goVarName} != null) {target} = {goVarName};");
      }
      else if (targetComponent is RectTransform)
      {
        builder.Line($"if ({goVarName} != null) {target} = (RectTransform){goVarName}.transform;");
      }
      else if (targetComponent is Transform)
      {
        builder.Line($"if ({goVarName} != null) {target} = {goVarName}.transform;");
      }
      else
      {
        var componentTypeName = targetComponent.GetType().Name;
        builder.Line(
          $"if ({goVarName} != null) {target} = {goVarName}.GetComponent<{componentTypeName}>();"
        );
      }
    }

    static string GetCanvasObjectPath(GameObject go)
    {
      var parts = new List<string>();
      var current = go.transform;

      while (current.parent != null && current.parent.GetComponent<Canvas>() == null)
      {
        parts.Insert(0, current.gameObject.name);
        current = current.parent;
      }

      if (current.parent != null)
      {
        parts.Insert(0, current.gameObject.name);
      }

      return string.Join("/", parts);
    }

    public void GenerateComponentReferences(
      CSharpCodeBuilder builder,
      Component component,
      string componentVar
    )
    {
      var serialized = new SerializedObject(component);
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
        GenerateChildReference(builder, objRef, componentVar, fieldName);
      }
    }

    string? GenerateComponentsOnGameObject(
      CSharpCodeBuilder builder,
      GameObject go,
      string goVarName,
      string baseVarName
    )
    {
      var components = go.GetComponents<Component>();
      var supportedComponents = components.Where(_isSupportedComponent).ToList();

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
      return components.FirstOrDefault(_isSupportedComponent);
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

      if (t is RectTransform rt)
      {
        GenerateRectTransform(builder, rt, goVar);
      }
      else
      {
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
          builder.Assign(
            $"{goVar}.transform.localScale",
            CSharpCodeBuilder.ToVector3(t.localScale)
          );
        }
      }
    }

    void GenerateRectTransform(CSharpCodeBuilder builder, RectTransform rt, string goVar)
    {
      var rectVar = goVar + "Rect";
      builder.AddComponent(rectVar, goVar, "RectTransform");

      if (rt.anchorMin != Vector2.zero || rt.anchorMax != Vector2.one)
      {
        builder.Assign($"{rectVar}.anchorMin", CSharpCodeBuilder.ToVector2(rt.anchorMin));
        builder.Assign($"{rectVar}.anchorMax", CSharpCodeBuilder.ToVector2(rt.anchorMax));
      }

      if (rt.pivot != new Vector2(0.5f, 0.5f))
      {
        builder.Assign($"{rectVar}.pivot", CSharpCodeBuilder.ToVector2(rt.pivot));
      }

      if (rt.anchoredPosition != Vector2.zero)
      {
        builder.Assign(
          $"{rectVar}.anchoredPosition",
          CSharpCodeBuilder.ToVector2(rt.anchoredPosition)
        );
      }

      if (rt.sizeDelta != Vector2.zero)
      {
        builder.Assign($"{rectVar}.sizeDelta", CSharpCodeBuilder.ToVector2(rt.sizeDelta));
      }

      if (rt.localRotation != Quaternion.identity)
      {
        builder.Assign(
          $"{rectVar}.localRotation",
          CSharpCodeBuilder.ToQuaternion(rt.localRotation)
        );
      }

      if (rt.localScale != Vector3.one)
      {
        builder.Assign($"{rectVar}.localScale", CSharpCodeBuilder.ToVector3(rt.localScale));
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
        Object.DestroyImmediate(tempGo);
      }
    }

    static bool ShouldSkipProperty(SerializedProperty prop)
    {
      if (SkipFields.Contains(prop.name))
      {
        return true;
      }

      if (prop.name.StartsWith("m_"))
      {
        return true;
      }

      if (prop.propertyPath.Contains("."))
      {
        return true;
      }

      return false;
    }

    static void GenerateFieldAssignment(
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
        if (!_isSupportedComponent(component))
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

      if (targetComponent is RectTransform)
      {
        builder.Assign(target, $"(RectTransform){goVar}.transform");
      }
      else if (targetComponent is Transform)
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

    public static string SanitizeVarName(string name)
    {
      var sb = new System.Text.StringBuilder();
      var capitalizeNext = false;

      foreach (var c in name)
      {
        if (char.IsLetterOrDigit(c))
        {
          if (sb.Length == 0)
          {
            sb.Append(char.ToLower(c));
          }
          else if (capitalizeNext)
          {
            sb.Append(char.ToUpper(c));
            capitalizeNext = false;
          }
          else
          {
            sb.Append(c);
          }
        }
        else if (c == '_')
        {
          if (sb.Length > 0)
          {
            capitalizeNext = true;
          }
        }
        else
        {
          capitalizeNext = true;
        }
      }

      var result = sb.ToString();

      if (result.Length == 0)
      {
        return "item";
      }

      if (char.IsDigit(result[0]))
      {
        result = "n" + result;
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
