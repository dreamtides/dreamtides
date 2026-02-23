#if UNITY_EDITOR

#nullable enable

using Dreamtides.Services;
using Dreamtides.Utils;
using UnityEditor;
using UnityEngine;

namespace Dreamtides.Editors
{
  internal static class PlayModeSelection
  {
    private const string EditorPrefKey = "PlayModeToggle.CurrentMode";

    public static GameMode Current
    {
      get => (GameMode)EditorPrefs.GetInt(EditorPrefKey, (int)GameMode.Quest);
      set => EditorPrefs.SetInt(EditorPrefKey, (int)value);
    }
  }

  [InitializeOnLoad]
  public static class PlayModeGameModeMenu
  {
    private const string MenuRoot = "Tools/Play Mode/";
    private const string MenuQuest = MenuRoot + "Quest #&q"; // Shift + Alt + Q
    private const string MenuPrototypeQuest = MenuRoot + "Prototype Quest #&p"; // Shift + Alt + P
    private const string MenuBattle = MenuRoot + "Battle #&b"; // Shift + Alt + B

    static PlayModeGameModeMenu()
    {
      EditorApplication.delayCall += UpdateChecks;
      EditorApplication.playModeStateChanged += OnPlayModeStateChanged;
    }

    private static void OnPlayModeStateChanged(PlayModeStateChange state)
    {
      if (state == PlayModeStateChange.ExitingEditMode)
      {
        // About to enter Play: persist selection so runtime can read it.
        PlayerPrefs.SetInt(PlayerPrefKeys.SelectedPlayMode, (int)PlayModeSelection.Current);
        PlayerPrefs.Save();
      }
    }

    [MenuItem(MenuQuest)]
    private static void SetQuest()
    {
      PlayModeSelection.Current = GameMode.Quest;
      Debug.Log($"Set play mode to {PlayModeSelection.Current}");
      UpdateChecks();
    }

    [MenuItem(MenuQuest, true)]
    private static bool ValidateQuest()
    {
      Menu.SetChecked(MenuQuest, PlayModeSelection.Current == GameMode.Quest);
      return true;
    }

    [MenuItem(MenuPrototypeQuest)]
    private static void SetPrototypeQuest()
    {
      PlayModeSelection.Current = GameMode.PrototypeQuest;
      Debug.Log($"Set play mode to {PlayModeSelection.Current}");
      UpdateChecks();
    }

    [MenuItem(MenuPrototypeQuest, true)]
    private static bool ValidatePrototypeQuest()
    {
      Menu.SetChecked(MenuPrototypeQuest, PlayModeSelection.Current == GameMode.PrototypeQuest);
      return true;
    }

    [MenuItem(MenuBattle)]
    private static void SetBattle()
    {
      PlayModeSelection.Current = GameMode.Battle;
      Debug.Log($"Set play mode to {PlayModeSelection.Current}");
      UpdateChecks();
    }

    [MenuItem(MenuBattle, true)]
    private static bool ValidateBattle()
    {
      Menu.SetChecked(MenuBattle, PlayModeSelection.Current == GameMode.Battle);
      return true;
    }

    private static void UpdateChecks()
    {
      Menu.SetChecked(MenuQuest, PlayModeSelection.Current == GameMode.Quest);
      Menu.SetChecked(MenuPrototypeQuest, PlayModeSelection.Current == GameMode.PrototypeQuest);
      Menu.SetChecked(MenuBattle, PlayModeSelection.Current == GameMode.Battle);
    }
  }
}
#endif
