#nullable enable

using System.Collections.Generic;
using System.Linq;
using Dreamtides.Components;
using UnityEditor;
using UnityEngine;

namespace Dreamtides.Editors
{
  [CustomEditor(typeof(DreamscapeSite))]
  public sealed class DreamscapeSiteEditor : Editor
  {
    public override void OnInspectorGUI()
    {
      DrawDefaultInspector();
      if (GUILayout.Button("Activate"))
      {
        var selectedSites = new HashSet<DreamscapeSite>(targets.Cast<DreamscapeSite>());
        var allSites = FindObjectsByType<DreamscapeSite>(
          FindObjectsInactive.Include,
          FindObjectsSortMode.None
        );
        foreach (var site in allSites)
        {
          var shouldActivate = selectedSites.Contains(site);
          Undo.RecordObject(
            site,
            shouldActivate ? "Activate Dreamscape Site" : "Deactivate Dreamscape Site"
          );
          site.SetActive(isActive: shouldActivate);
          EditorUtility.SetDirty(site);
        }
      }
      using (new EditorGUI.DisabledScope(!EditorApplication.isPlaying))
      {
        if (GUILayout.Button("Save"))
        {
          foreach (var editorTarget in targets)
          {
            var dreamscapeSite = (DreamscapeSite)editorTarget;
            PlayModeValueSaver.SaveNow(dreamscapeSite);
            PlayModeValueSaver.SaveNow(dreamscapeSite.transform);
          }
        }
      }
    }
  }
}
