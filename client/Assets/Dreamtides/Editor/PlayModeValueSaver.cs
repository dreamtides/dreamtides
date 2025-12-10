#nullable enable

using System.Collections.Generic;
using Dreamtides.Components;
using Dreamtides.Sites;
using UnityEditor;
using UnityEngine;

[InitializeOnLoad]
public static class PlayModeValueSaver
{
  static readonly Dictionary<GlobalObjectId, string> _pendingSaves = new();

  static PlayModeValueSaver()
  {
    EditorApplication.playModeStateChanged += OnPlayModeStateChanged;
  }

  public static void SaveNow(Object runtimeObject)
  {
    if (runtimeObject == null)
    {
      return;
    }
    if (!EditorApplication.isPlaying)
    {
      Debug.LogWarning("PlayModeValueSaver.SaveNow called while not playing.");
      return;
    }
    var id = GlobalObjectId.GetGlobalObjectIdSlow(runtimeObject);
    var json = EditorJsonUtility.ToJson(runtimeObject);
    _pendingSaves[id] = json;
    Debug.Log(
      $"[PlayModeValueSaver] Queued values for '{runtimeObject.name}'. They will be applied when you exit Play mode."
    );
  }

  static void OnPlayModeStateChanged(PlayModeStateChange change)
  {
    if (change != PlayModeStateChange.EnteredEditMode || _pendingSaves.Count == 0)
    {
      return;
    }
    Debug.Log($"[PlayModeValueSaver] Applying {_pendingSaves.Count} queued play-mode save(s).");
    foreach (var kvp in _pendingSaves)
    {
      var id = kvp.Key;
      var json = kvp.Value;
      var originalObj = GlobalObjectId.GlobalObjectIdentifierToObjectSlow(id);
      if (originalObj == null)
      {
        Debug.LogWarning($"[PlayModeValueSaver] Original object for id {id} not found. Skipping.");
        continue;
      }
      Undo.RecordObject(originalObj, "Apply Play Mode Values");
      var characterSite = originalObj as CharacterSite;
      var abstractSite = originalObj as AbstractDreamscapeSite;
      var targetScreenLeft = (Object?)null;
      var targetScreenRight = (Object?)null;
      var targetScreenTop = (Object?)null;
      if (characterSite != null)
      {
        var serialized = new SerializedObject(characterSite);
        targetScreenLeft = serialized.FindProperty("_targetScreenLeftCamera").objectReferenceValue;
        targetScreenRight = serialized
          .FindProperty("_targetScreenRightCamera")
          .objectReferenceValue;
        targetScreenTop = serialized.FindProperty("_targetScreenTopCamera").objectReferenceValue;
      }
      EditorJsonUtility.FromJsonOverwrite(json, originalObj);
      if (characterSite != null)
      {
        var serialized = new SerializedObject(characterSite);
        serialized.FindProperty("_targetScreenLeftCamera").objectReferenceValue = targetScreenLeft;
        serialized.FindProperty("_targetScreenRightCamera").objectReferenceValue =
          targetScreenRight;
        serialized.FindProperty("_targetScreenTopCamera").objectReferenceValue = targetScreenTop;
        var isActiveProperty = serialized.FindProperty("_isActive");
        if (isActiveProperty != null)
        {
          isActiveProperty.boolValue = false;
        }
        serialized.ApplyModifiedPropertiesWithoutUndo();
      }
      else if (abstractSite != null)
      {
        var serialized = new SerializedObject(abstractSite);
        var isActiveProperty = serialized.FindProperty("_isActive");
        if (isActiveProperty != null)
        {
          isActiveProperty.boolValue = false;
        }
        serialized.ApplyModifiedPropertiesWithoutUndo();
      }
      EditorUtility.SetDirty(originalObj);
    }
    _pendingSaves.Clear();
    AssetDatabase.SaveAssets();
  }
}
