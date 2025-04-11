#nullable enable

using System;
using System.Collections;
using Dreamtides.Layout;
using Dreamtides.Services;
using Dreamtides.UnityInternal;
using Dreamtides.Utils;
using UnityEngine;
using UnityEngine.Assertions;
using UnityEngine.SceneManagement;

namespace Dreamtides.Tests
{
  public static class TestUtil
  {
    public static IEnumerator LoadScenario(GameViewResolution resolution, string scenario, Action<Registry> action)
    {
      Registry.TestConfiguration = new TestConfiguration(scenario);
      GameViewUtils.SetGameViewResolution(resolution);
      SceneManager.LoadScene("Assets/Scenes/Main.unity", LoadSceneMode.Single);
      yield return WaitForSceneLoad();
      var registry = ComponentUtils.Get<Registry>(GameObject.Find("Registry"));
      Assert.IsNotNull(registry);
      Assert.AreEqual(GameViewUtils.GetResolution(resolution), new Vector2(Screen.width, Screen.height),
          $"Resolution {resolution} not set");

      yield return new WaitUntil(() => registry.ActionService.Connected);
      yield return registry.TestHelperService.WaitForIdle();

      action(registry);
    }

    public static IEnumerator WaitForCount(Registry registry, ObjectLayout layout, int count)
    {
      yield return new WaitUntil(() => layout.Objects.Count == count);
      yield return registry.TestHelperService.WaitForIdle();
    }

    public static IEnumerator TearDownScenario(Registry registry)
    {
      Registry.TestConfiguration = null;
      yield return registry.TestHelperService.WaitForIdle();
      yield return new WaitForSeconds(0.5f);
    }

    static IEnumerator WaitForSceneLoad()
    {
      while (SceneManager.GetActiveScene().buildIndex > 0)
      {
        yield return null;
      }
    }
  }
}