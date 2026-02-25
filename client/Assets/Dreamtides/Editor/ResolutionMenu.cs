#if UNITY_EDITOR

#nullable enable

using System;
using System.Reflection;
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

    private static readonly (string menu, int width, int height, int id)[] Resolutions =
    {
      (Menu16x9, 1920, 1080, 0),
      (Menu16x10, 2560, 1600, 1),
      (Menu21x9, 3440, 1440, 2),
      (Menu3x2, 1470, 956, 3),
      (Menu5x4, 1280, 1024, 4),
      (Menu32x9, 5120, 1440, 5),
      (MenuIPhone12, 1170, 2532, 6),
      (MenuIPhoneSE, 750, 1334, 7),
      (MenuIPadPro12, 2048, 2732, 8),
      (MenuIPodTouch6, 640, 1136, 9),
      (MenuSamsungNote20, 1440, 3088, 10),
      (MenuSamsungZFold2, 960, 2658, 11),
      (MenuPixel5, 1080, 2340, 12),
    };

    static ResolutionMenu()
    {
      EditorApplication.delayCall += UpdateChecks;
      EditorApplication.playModeStateChanged += OnPlayModeStateChanged;
    }

    private static int CurrentId
    {
      get => EditorPrefs.GetInt(EditorPrefKey, 0);
      set => EditorPrefs.SetInt(EditorPrefKey, value);
    }

    private static void OnPlayModeStateChanged(PlayModeStateChange state)
    {
      if (state == PlayModeStateChange.EnteredPlayMode)
      {
        ApplyCurrentResolution();
      }
    }

    private static void Select(int id)
    {
      CurrentId = id;
      ApplyCurrentResolution();
      UpdateChecks();
    }

    private static void ApplyCurrentResolution()
    {
      var current = CurrentId;
      foreach (var (_, width, height, id) in Resolutions)
      {
        if (id == current)
        {
          ApplyResolution(width, height);
          return;
        }
      }
    }

    private static void ApplyResolution(int width, int height)
    {
      try
      {
        var gameViewSizesType = typeof(Editor).Assembly.GetType("UnityEditor.GameViewSizes");
        var singletonProperty =
          gameViewSizesType!.GetProperty("instance", BindingFlags.Public | BindingFlags.Static);
        var gameViewSizesInstance = singletonProperty!.GetValue(null);

        var currentGroupType = GetCurrentGroupType(gameViewSizesType!, gameViewSizesInstance!);
        var group = GetGroup(gameViewSizesType!, gameViewSizesInstance!, currentGroupType);

        var index = FindOrAddCustomSize(group, width, height);

        SetGameViewSize(index);
      }
      catch (Exception e)
      {
        Debug.LogError($"Failed to apply resolution {width}x{height}: {e}");
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

    private static int FindOrAddCustomSize(object group, int width, int height)
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
      var ctor = gameViewSizeType.GetConstructor(
        new[] { sizeTypeEnum, typeof(int), typeof(int), typeof(string) }
      )!;
      var newSize = ctor.Invoke(new object[] { fixedResolution, width, height, $"{width}x{height}" });

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
      var current = CurrentId;
      foreach (var (menu, _, _, id) in Resolutions)
      {
        Menu.SetChecked(menu, id == current);
      }
    }

    // --- Menu items ---

    [MenuItem(Menu16x9)]
    private static void Set16x9() => Select(0);

    [MenuItem(Menu16x9, true)]
    private static bool Validate16x9() { UpdateChecks(); return true; }

    [MenuItem(Menu16x10)]
    private static void Set16x10() => Select(1);

    [MenuItem(Menu16x10, true)]
    private static bool Validate16x10() { UpdateChecks(); return true; }

    [MenuItem(Menu21x9)]
    private static void Set21x9() => Select(2);

    [MenuItem(Menu21x9, true)]
    private static bool Validate21x9() { UpdateChecks(); return true; }

    [MenuItem(Menu3x2)]
    private static void Set3x2() => Select(3);

    [MenuItem(Menu3x2, true)]
    private static bool Validate3x2() { UpdateChecks(); return true; }

    [MenuItem(Menu5x4)]
    private static void Set5x4() => Select(4);

    [MenuItem(Menu5x4, true)]
    private static bool Validate5x4() { UpdateChecks(); return true; }

    [MenuItem(Menu32x9)]
    private static void Set32x9() => Select(5);

    [MenuItem(Menu32x9, true)]
    private static bool Validate32x9() { UpdateChecks(); return true; }

    [MenuItem(MenuIPhone12)]
    private static void SetIPhone12() => Select(6);

    [MenuItem(MenuIPhone12, true)]
    private static bool ValidateIPhone12() { UpdateChecks(); return true; }

    [MenuItem(MenuIPhoneSE)]
    private static void SetIPhoneSE() => Select(7);

    [MenuItem(MenuIPhoneSE, true)]
    private static bool ValidateIPhoneSE() { UpdateChecks(); return true; }

    [MenuItem(MenuIPadPro12)]
    private static void SetIPadPro12() => Select(8);

    [MenuItem(MenuIPadPro12, true)]
    private static bool ValidateIPadPro12() { UpdateChecks(); return true; }

    [MenuItem(MenuIPodTouch6)]
    private static void SetIPodTouch6() => Select(9);

    [MenuItem(MenuIPodTouch6, true)]
    private static bool ValidateIPodTouch6() { UpdateChecks(); return true; }

    [MenuItem(MenuSamsungNote20)]
    private static void SetSamsungNote20() => Select(10);

    [MenuItem(MenuSamsungNote20, true)]
    private static bool ValidateSamsungNote20() { UpdateChecks(); return true; }

    [MenuItem(MenuSamsungZFold2)]
    private static void SetSamsungZFold2() => Select(11);

    [MenuItem(MenuSamsungZFold2, true)]
    private static bool ValidateSamsungZFold2() { UpdateChecks(); return true; }

    [MenuItem(MenuPixel5)]
    private static void SetPixel5() => Select(12);

    [MenuItem(MenuPixel5, true)]
    private static bool ValidatePixel5() { UpdateChecks(); return true; }
  }
}
#endif
