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
      var targetCamera = ((DreamscapeMapCamera)target).Camera;
      if (targetCamera == null)
      {
        return;
      }
      var allCameras = FindObjectsByType<CinemachineCamera>(
        FindObjectsInactive.Include,
        FindObjectsSortMode.None
      );
      foreach (var camera in allCameras)
      {
        Undo.RecordObject(
          camera,
          camera == targetCamera
            ? "Activate Dreamscape Map Camera"
            : "Deactivate Dreamscape Map Camera"
        );
        camera.Priority = camera == targetCamera ? 10 : 0;
        EditorUtility.SetDirty(camera);
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
  }
}
