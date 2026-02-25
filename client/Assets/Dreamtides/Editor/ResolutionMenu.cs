#if UNITY_EDITOR

#nullable enable

using System;
using System.Reflection;
using Dreamtides.Tests.TestUtils;
using UnityEditor;
using UnityEngine;

namespace Dreamtides.Editors
{
  [InitializeOnLoad]
  public static class ResolutionMenu
  {
    private const string EditorPrefKey = "ResolutionMenu.SelectedResolution";
    private const string MenuRoot = "Tools/Resolution/";

    private const string Menu16x9 = MenuRoot + "Landscape 16:9 (1920x1080)";
    private const string Menu16x10 = MenuRoot + "Landscape 16:10 (2560x1600)";
    private const string Menu21x9 = MenuRoot + "Landscape 21:9 (3440x1440)";
    private const string Menu3x2 = MenuRoot + "Landscape 3:2 (1470x956)";
    private const string Menu5x4 = MenuRoot + "Landscape 5:4 (1280x1024)";
    private const string Menu32x9 = MenuRoot + "Landscape 32:9 (5120x1440)";
    private const string MenuIPhone12 = MenuRoot + "iPhone 12 (1170x2532)";
    private const string MenuIPhoneSE = MenuRoot + "iPhone SE (750x1334)";
    private const string MenuIPadPro12 = MenuRoot + "iPad Pro 12 (2048x2732)";
    private const string MenuIPodTouch6 = MenuRoot + "iPod Touch 6 (640x1136)";
    private const string MenuSamsungNote20 = MenuRoot + "Samsung Note 20 (1440x3088)";
    private const string MenuSamsungZFold2 = MenuRoot + "Samsung Z Fold 2 (960x2658)";
    private const string MenuPixel5 = MenuRoot + "Pixel 5 (1080x2340)";

    static ResolutionMenu()
    {
      EditorApplication.delayCall += UpdateChecks;
      EditorApplication.playModeStateChanged += OnPlayModeStateChanged;
    }

    private static GameViewResolution Current
    {
      get => (GameViewResolution)EditorPrefs.GetInt(EditorPrefKey, (int)GameViewResolution.Resolution16x9);
      set => EditorPrefs.SetInt(EditorPrefKey, (int)value);
    }

    private static void OnPlayModeStateChanged(PlayModeStateChange state)
    {
      if (state == PlayModeStateChange.EnteredPlayMode)
      {
        ApplyResolution(Current);
      }
    }

    private static void Select(GameViewResolution resolution)
    {
      Current = resolution;
      ApplyResolution(resolution);
      UpdateChecks();
    }

    private static void ApplyResolution(GameViewResolution resolution)
    {
      var size = resolution.AsVector();
      var width = (int)size.x;
      var height = (int)size.y;

      try
      {
        var gameViewSizesType = typeof(Editor).Assembly.GetType("UnityEditor.GameViewSizes");
        var singletonProperty =
          gameViewSizesType.GetProperty("instance", BindingFlags.Public | BindingFlags.Static);
        var gameViewSizesInstance = singletonProperty!.GetValue(null);

        var currentGroupType = GetCurrentGroupType(gameViewSizesType, gameViewSizesInstance!);
        var group = GetGroup(gameViewSizesType, gameViewSizesInstance!, currentGroupType);

        var index = FindOrAddCustomSize(group, width, height, resolution.ToString());

        SetGameViewSize(index);
      }
      catch (Exception e)
      {
        Debug.LogError($"Failed to apply resolution {resolution}: {e}");
      }
    }

    private static object GetCurrentGroupType(Type gameViewSizesType, object instance)
    {
      var prop = gameViewSizesType.GetProperty(
        "currentGroupType",
        BindingFlags.Public | BindingFlags.Instance
      );
      return prop!.GetValue(instance)!;
    }

    private static object GetGroup(Type gameViewSizesType, object instance, object groupType)
    {
      var method = gameViewSizesType.GetMethod(
        "GetGroup",
        BindingFlags.Public | BindingFlags.Instance
      );
      return method!.Invoke(instance, new[] { groupType })!;
    }

    private static int FindOrAddCustomSize(object group, int width, int height, string name)
    {
      var groupType = group.GetType();
      var getTotalCount = groupType.GetMethod("GetTotalCount")!;
      var getGameViewSize = groupType.GetMethod("GetGameViewSize")!;
      var totalCount = (int)getTotalCount.Invoke(group, null)!;

      var gameViewSizeType = typeof(Editor).Assembly.GetType("UnityEditor.GameViewSize");
      var widthProp = gameViewSizeType!.GetProperty("width", BindingFlags.Public | BindingFlags.Instance)!;
      var heightProp = gameViewSizeType.GetProperty("height", BindingFlags.Public | BindingFlags.Instance)!;

      for (var i = 0; i < totalCount; i++)
      {
        var size = getGameViewSize.Invoke(group, new object[] { i })!;
        var w = (int)widthProp.GetValue(size)!;
        var h = (int)heightProp.GetValue(size)!;
        if (w == width && h == height)
        {
          return i;
        }
      }

      var sizeTypeEnum = typeof(Editor).Assembly.GetType("UnityEditor.GameViewSizeType")!;
      var fixedResolution = Enum.Parse(sizeTypeEnum, "FixedResolution");
      var ctor = gameViewSizeType.GetConstructor(new[] { sizeTypeEnum, typeof(int), typeof(int), typeof(string) })!;
      var newSize = ctor.Invoke(new[] { fixedResolution, width, height, name });

      var addCustomSize = groupType.GetMethod("AddCustomSize")!;
      addCustomSize.Invoke(group, new[] { newSize });

      totalCount = (int)getTotalCount.Invoke(group, null)!;
      return totalCount - 1;
    }

    private static void SetGameViewSize(int index)
    {
      var gameViewType = typeof(Editor).Assembly.GetType("UnityEditor.GameView")!;
      var getWindow = typeof(EditorWindow).GetMethod(
        "GetWindow",
        BindingFlags.Public | BindingFlags.Static,
        null,
        new[] { typeof(Type), typeof(bool) },
        null
      )!;
      var gameView = getWindow.Invoke(null, new object[] { gameViewType, false }) as EditorWindow;
      if (gameView == null) return;

      var selectedSizeIndex =
        gameViewType.GetProperty("selectedSizeIndex", BindingFlags.Public | BindingFlags.Instance)!;
      selectedSizeIndex.SetValue(gameView, index);
    }

    private static void UpdateChecks()
    {
      Menu.SetChecked(Menu16x9, Current == GameViewResolution.Resolution16x9);
      Menu.SetChecked(Menu16x10, Current == GameViewResolution.Resolution16x10);
      Menu.SetChecked(Menu21x9, Current == GameViewResolution.Resolution21x9);
      Menu.SetChecked(Menu3x2, Current == GameViewResolution.Resolution3x2);
      Menu.SetChecked(Menu5x4, Current == GameViewResolution.Resolution5x4);
      Menu.SetChecked(Menu32x9, Current == GameViewResolution.Resolution32x9);
      Menu.SetChecked(MenuIPhone12, Current == GameViewResolution.ResolutionIPhone12);
      Menu.SetChecked(MenuIPhoneSE, Current == GameViewResolution.ResolutionIPhoneSE);
      Menu.SetChecked(MenuIPadPro12, Current == GameViewResolution.ResolutionIPadPro12);
      Menu.SetChecked(MenuIPodTouch6, Current == GameViewResolution.ResolutionIPodTouch6);
      Menu.SetChecked(MenuSamsungNote20, Current == GameViewResolution.ResolutionSamsungNote20);
      Menu.SetChecked(MenuSamsungZFold2, Current == GameViewResolution.ResolutionSamsungZFold2);
      Menu.SetChecked(MenuPixel5, Current == GameViewResolution.ResolutionPixel5);
    }

    // --- Menu items ---

    [MenuItem(Menu16x9)]
    private static void Set16x9() => Select(GameViewResolution.Resolution16x9);

    [MenuItem(Menu16x9, true)]
    private static bool Validate16x9()
    {
      Menu.SetChecked(Menu16x9, Current == GameViewResolution.Resolution16x9);
      return true;
    }

    [MenuItem(Menu16x10)]
    private static void Set16x10() => Select(GameViewResolution.Resolution16x10);

    [MenuItem(Menu16x10, true)]
    private static bool Validate16x10()
    {
      Menu.SetChecked(Menu16x10, Current == GameViewResolution.Resolution16x10);
      return true;
    }

    [MenuItem(Menu21x9)]
    private static void Set21x9() => Select(GameViewResolution.Resolution21x9);

    [MenuItem(Menu21x9, true)]
    private static bool Validate21x9()
    {
      Menu.SetChecked(Menu21x9, Current == GameViewResolution.Resolution21x9);
      return true;
    }

    [MenuItem(Menu3x2)]
    private static void Set3x2() => Select(GameViewResolution.Resolution3x2);

    [MenuItem(Menu3x2, true)]
    private static bool Validate3x2()
    {
      Menu.SetChecked(Menu3x2, Current == GameViewResolution.Resolution3x2);
      return true;
    }

    [MenuItem(Menu5x4)]
    private static void Set5x4() => Select(GameViewResolution.Resolution5x4);

    [MenuItem(Menu5x4, true)]
    private static bool Validate5x4()
    {
      Menu.SetChecked(Menu5x4, Current == GameViewResolution.Resolution5x4);
      return true;
    }

    [MenuItem(Menu32x9)]
    private static void Set32x9() => Select(GameViewResolution.Resolution32x9);

    [MenuItem(Menu32x9, true)]
    private static bool Validate32x9()
    {
      Menu.SetChecked(Menu32x9, Current == GameViewResolution.Resolution32x9);
      return true;
    }

    [MenuItem(MenuIPhone12)]
    private static void SetIPhone12() => Select(GameViewResolution.ResolutionIPhone12);

    [MenuItem(MenuIPhone12, true)]
    private static bool ValidateIPhone12()
    {
      Menu.SetChecked(MenuIPhone12, Current == GameViewResolution.ResolutionIPhone12);
      return true;
    }

    [MenuItem(MenuIPhoneSE)]
    private static void SetIPhoneSE() => Select(GameViewResolution.ResolutionIPhoneSE);

    [MenuItem(MenuIPhoneSE, true)]
    private static bool ValidateIPhoneSE()
    {
      Menu.SetChecked(MenuIPhoneSE, Current == GameViewResolution.ResolutionIPhoneSE);
      return true;
    }

    [MenuItem(MenuIPadPro12)]
    private static void SetIPadPro12() => Select(GameViewResolution.ResolutionIPadPro12);

    [MenuItem(MenuIPadPro12, true)]
    private static bool ValidateIPadPro12()
    {
      Menu.SetChecked(MenuIPadPro12, Current == GameViewResolution.ResolutionIPadPro12);
      return true;
    }

    [MenuItem(MenuIPodTouch6)]
    private static void SetIPodTouch6() => Select(GameViewResolution.ResolutionIPodTouch6);

    [MenuItem(MenuIPodTouch6, true)]
    private static bool ValidateIPodTouch6()
    {
      Menu.SetChecked(MenuIPodTouch6, Current == GameViewResolution.ResolutionIPodTouch6);
      return true;
    }

    [MenuItem(MenuSamsungNote20)]
    private static void SetSamsungNote20() => Select(GameViewResolution.ResolutionSamsungNote20);

    [MenuItem(MenuSamsungNote20, true)]
    private static bool ValidateSamsungNote20()
    {
      Menu.SetChecked(MenuSamsungNote20, Current == GameViewResolution.ResolutionSamsungNote20);
      return true;
    }

    [MenuItem(MenuSamsungZFold2)]
    private static void SetSamsungZFold2() => Select(GameViewResolution.ResolutionSamsungZFold2);

    [MenuItem(MenuSamsungZFold2, true)]
    private static bool ValidateSamsungZFold2()
    {
      Menu.SetChecked(MenuSamsungZFold2, Current == GameViewResolution.ResolutionSamsungZFold2);
      return true;
    }

    [MenuItem(MenuPixel5)]
    private static void SetPixel5() => Select(GameViewResolution.ResolutionPixel5);

    [MenuItem(MenuPixel5, true)]
    private static bool ValidatePixel5()
    {
      Menu.SetChecked(MenuPixel5, Current == GameViewResolution.ResolutionPixel5);
      return true;
    }
  }
}
#endif
