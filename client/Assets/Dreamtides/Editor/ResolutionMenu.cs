#if UNITY_EDITOR

#nullable enable

using System;
using System.Reflection;
using UnityEditor;
using UnityEngine;

namespace Dreamtides.Editors
{
  [InitializeOnLoad]
  public static class DeviceMenu
  {
    private const string EditorPrefKey = "DeviceMenu.SelectedDevice";
    private const string MenuRoot = "Tools/Device/";

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

    private static readonly (
      string menu,
      string slug,
      int width,
      int height,
      bool mobile,
      int id
    )[] Devices =
    {
      (Menu16x9, "landscape-16x9", 1920, 1080, false, 0),
      (Menu16x10, "landscape-16x10", 2560, 1600, false, 1),
      (Menu21x9, "landscape-21x9", 3440, 1440, false, 2),
      (Menu3x2, "landscape-3x2", 1470, 956, false, 3),
      (Menu5x4, "landscape-5x4", 1280, 1024, false, 4),
      (Menu32x9, "landscape-32x9", 5120, 1440, false, 5),
      (MenuIPhone12, "iphone-12", 1170, 2532, true, 6),
      (MenuIPhoneSE, "iphone-se", 750, 1334, true, 7),
      (MenuIPadPro12, "ipad-pro-12", 2048, 2732, true, 8),
      (MenuIPodTouch6, "ipod-touch-6", 640, 1136, true, 9),
      (MenuSamsungNote20, "samsung-note-20", 1440, 3088, true, 10),
      (MenuSamsungZFold2, "samsung-z-fold-2", 960, 2658, true, 11),
      (MenuPixel5, "pixel-5", 1080, 2340, true, 12),
    };

    private const BindingFlags AllInstance =
      BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance;

    static DeviceMenu()
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
        ApplyCurrentDevice();
      }
    }

    private static void Select(int id)
    {
      CurrentId = id;
      ApplyCurrentDevice();
      UpdateChecks();
    }

    private static void ApplyCurrentDevice()
    {
      foreach (var (menu, slug, width, height, mobile, id) in Devices)
      {
        if (id == CurrentId)
        {
          ApplyDevice(menu, width, height, mobile);
          Debug.Log($"Set device to {slug}");
          return;
        }
      }
    }

    private static void ApplyDevice(string name, int width, int height, bool mobile)
    {
      if (mobile)
      {
        PlayModeWindow.SetViewType(PlayModeWindow.PlayModeViewTypes.SimulatorView);
        SelectSimulatorDevice(width, height);
      }
      else
      {
        PlayModeWindow.SetViewType(PlayModeWindow.PlayModeViewTypes.GameView);
        PlayModeWindow.SetCustomRenderingResolution((uint)width, (uint)height, name);
      }
    }

    private static void SelectSimulatorDevice(int width, int height)
    {
      try
      {
        Type? windowType = null;
        foreach (var asm in AppDomain.CurrentDomain.GetAssemblies())
        {
          windowType = asm.GetType("UnityEditor.DeviceSimulation.SimulatorWindow");
          if (windowType != null)
            break;
        }

        if (windowType == null)
        {
          Debug.LogWarning("SimulatorWindow type not found in any loaded assembly");
          return;
        }

        var window = EditorWindow.GetWindow(windowType, false);
        if (window == null)
          return;

        var mainField = windowType.GetField("m_Main", AllInstance);
        if (mainField == null)
        {
          Debug.LogWarning("m_Main field not found on SimulatorWindow");
          return;
        }

        var main = mainField.GetValue(window);
        if (main == null)
        {
          Debug.LogWarning("m_Main is null on SimulatorWindow");
          return;
        }

        var mainType = main.GetType();
        var devicesProp = mainType.GetProperty("devices", AllInstance);
        var deviceIndexProp = mainType.GetProperty("deviceIndex", AllInstance);
        if (devicesProp == null || deviceIndexProp == null)
        {
          Debug.LogWarning("devices or deviceIndex property not found on DeviceSimulatorMain");
          return;
        }

        var devices = devicesProp.GetValue(main) as Array;
        if (devices == null || devices.Length == 0)
          return;

        var deviceInfoAssetType = devices.GetType().GetElementType()!;

        for (var i = 0; i < devices.Length; i++)
        {
          var device = devices.GetValue(i);
          if (device == null)
            continue;

          if (DeviceMatchesResolution(device, deviceInfoAssetType, width, height))
          {
            deviceIndexProp.SetValue(main, i);
            return;
          }
        }

        Debug.LogWarning($"No simulator device found matching {width}x{height}");
      }
      catch (Exception e)
      {
        Debug.LogError($"Failed to select simulator device: {e}");
      }
    }

    private static bool DeviceMatchesResolution(
      object device,
      Type assetType,
      int width,
      int height
    )
    {
      var deviceInfoField = assetType.GetField("deviceInfo", AllInstance);
      if (deviceInfoField == null)
        return false;

      var deviceInfo = deviceInfoField.GetValue(device);
      if (deviceInfo == null)
        return false;

      var infoType = deviceInfo.GetType();
      var screensMember =
        (MemberInfo?)infoType.GetField("screens", AllInstance)
        ?? infoType.GetProperty("screens", AllInstance);

      object? screens = screensMember switch
      {
        FieldInfo fi => fi.GetValue(deviceInfo),
        PropertyInfo pi => pi.GetValue(deviceInfo),
        _ => null,
      };

      if (screens is not Array screenArray || screenArray.Length == 0)
        return false;

      var screen = screenArray.GetValue(0);
      if (screen == null)
        return false;

      var screenType = screen.GetType();
      var w = GetIntMember(screenType, screen, "width");
      var h = GetIntMember(screenType, screen, "height");

      return (w == width && h == height) || (w == height && h == width);
    }

    private static int? GetIntMember(Type type, object obj, string name)
    {
      var member =
        (MemberInfo?)type.GetField(name, AllInstance) ?? type.GetProperty(name, AllInstance);
      return member switch
      {
        FieldInfo fi => (int)fi.GetValue(obj)!,
        PropertyInfo pi => (int)pi.GetValue(obj)!,
        _ => null,
      };
    }

    private static void UpdateChecks()
    {
      var current = CurrentId;
      foreach (var (menu, _, _, _, _, id) in Devices)
      {
        Menu.SetChecked(menu, id == current);
      }
    }

    // --- Menu items ---

    [MenuItem(Menu16x9)]
    private static void Set16x9() => Select(0);

    [MenuItem(Menu16x9, true)]
    private static bool Validate16x9()
    {
      UpdateChecks();
      return true;
    }

    [MenuItem(Menu16x10)]
    private static void Set16x10() => Select(1);

    [MenuItem(Menu16x10, true)]
    private static bool Validate16x10()
    {
      UpdateChecks();
      return true;
    }

    [MenuItem(Menu21x9)]
    private static void Set21x9() => Select(2);

    [MenuItem(Menu21x9, true)]
    private static bool Validate21x9()
    {
      UpdateChecks();
      return true;
    }

    [MenuItem(Menu3x2)]
    private static void Set3x2() => Select(3);

    [MenuItem(Menu3x2, true)]
    private static bool Validate3x2()
    {
      UpdateChecks();
      return true;
    }

    [MenuItem(Menu5x4)]
    private static void Set5x4() => Select(4);

    [MenuItem(Menu5x4, true)]
    private static bool Validate5x4()
    {
      UpdateChecks();
      return true;
    }

    [MenuItem(Menu32x9)]
    private static void Set32x9() => Select(5);

    [MenuItem(Menu32x9, true)]
    private static bool Validate32x9()
    {
      UpdateChecks();
      return true;
    }

    [MenuItem(MenuIPhone12)]
    private static void SetIPhone12() => Select(6);

    [MenuItem(MenuIPhone12, true)]
    private static bool ValidateIPhone12()
    {
      UpdateChecks();
      return true;
    }

    [MenuItem(MenuIPhoneSE)]
    private static void SetIPhoneSE() => Select(7);

    [MenuItem(MenuIPhoneSE, true)]
    private static bool ValidateIPhoneSE()
    {
      UpdateChecks();
      return true;
    }

    [MenuItem(MenuIPadPro12)]
    private static void SetIPadPro12() => Select(8);

    [MenuItem(MenuIPadPro12, true)]
    private static bool ValidateIPadPro12()
    {
      UpdateChecks();
      return true;
    }

    [MenuItem(MenuIPodTouch6)]
    private static void SetIPodTouch6() => Select(9);

    [MenuItem(MenuIPodTouch6, true)]
    private static bool ValidateIPodTouch6()
    {
      UpdateChecks();
      return true;
    }

    [MenuItem(MenuSamsungNote20)]
    private static void SetSamsungNote20() => Select(10);

    [MenuItem(MenuSamsungNote20, true)]
    private static bool ValidateSamsungNote20()
    {
      UpdateChecks();
      return true;
    }

    [MenuItem(MenuSamsungZFold2)]
    private static void SetSamsungZFold2() => Select(11);

    [MenuItem(MenuSamsungZFold2, true)]
    private static bool ValidateSamsungZFold2()
    {
      UpdateChecks();
      return true;
    }

    [MenuItem(MenuPixel5)]
    private static void SetPixel5() => Select(12);

    [MenuItem(MenuPixel5, true)]
    private static bool ValidatePixel5()
    {
      UpdateChecks();
      return true;
    }
  }
}
#endif
