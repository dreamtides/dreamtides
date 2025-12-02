#nullable enable

using Dreamtides.Components;
using Unity.Cinemachine;
using UnityEditor;
using UnityEngine;

namespace Dreamtides.Editors
{
  [CustomEditor(typeof(DreamscapeMapCamera))]
  public sealed class DreamscapeMapCameraEditor : Editor
  {
    public override void OnInspectorGUI()
    {
      DrawDefaultInspector();
      if (GUILayout.Button("Activate"))
      {
        ActivateCamera();
      }
      using (new EditorGUI.DisabledScope(!EditorApplication.isPlaying))
      {
        if (GUILayout.Button("Save"))
        {
          SaveValues();
        }
      }
    }

    void ActivateCamera()
    {
      var mapCamera = (DreamscapeMapCamera)target;
      var targetCamera = mapCamera.Camera;
      if (targetCamera == null)
      {
        return;
      }
      mapCamera.FrameSites();
      DeactivateSites();
      var mapCameras = FindObjectsByType<DreamscapeMapCamera>(
        FindObjectsInactive.Include,
        FindObjectsSortMode.None
      );
      foreach (var other in mapCameras)
      {
        var otherCamera = other.Camera;
        if (otherCamera == null)
        {
          continue;
        }
        Undo.RecordObject(
          otherCamera,
          other == mapCamera ? "Activate Dreamscape Map Camera" : "Deactivate Dreamscape Map Camera"
        );
        otherCamera.Priority = other == mapCamera ? 10 : 0;
        EditorUtility.SetDirty(otherCamera);
      }
    }

    void SaveValues()
    {
      foreach (var editorTarget in targets)
      {
        var mapCamera = (DreamscapeMapCamera)editorTarget;
        PlayModeValueSaver.SaveNow(mapCamera);
        PlayModeValueSaver.SaveNow(mapCamera.transform);
        if (mapCamera.Camera != null)
        {
          PlayModeValueSaver.SaveNow(mapCamera.Camera);
          PlayModeValueSaver.SaveNow(mapCamera.Camera.transform);
        }
      }
    }

    void DeactivateSites()
    {
      var sites = FindObjectsByType<DreamscapeSite>(
        FindObjectsInactive.Include,
        FindObjectsSortMode.None
      );
      foreach (var site in sites)
      {
        Undo.RecordObject(site, "Deactivate Dreamscape Site");
        site.SetActive(isActive: false);
        EditorUtility.SetDirty(site);
      }
    }
  }
}
