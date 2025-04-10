#nullable enable

using System;
using System.Collections;
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
      TweenUtils.TestMode = true;
      GameViewUtils.SetGameViewResolution(resolution);
      SceneManager.LoadScene("Assets/Scenes/Main.unity", LoadSceneMode.Single);
      yield return WaitForSceneLoad();
      var registry = ComponentUtils.Get<Registry>(GameObject.Find("Registry"));
      Assert.IsNotNull(registry);
      Assert.AreEqual(GameViewUtils.GetResolution(resolution), new Vector2(Screen.width, Screen.height),
          $"Resolution {resolution} not set");

      yield return new WaitUntil(() => registry.ActionService.Connected);
      yield return WaitForAnimations();

      action(registry);
    }

    public static IEnumerator TearDownScenario()
    {
      Registry.TestConfiguration = null;
      TweenUtils.TestMode = false;
      yield return WaitForAnimations();
    }

    static IEnumerator WaitForAnimations()
    {
      yield return TweenUtils.WaitForActiveSequences();

      foreach (var system in UnityEngine.Object.FindObjectsByType<ParticleSystem>(FindObjectsSortMode.None))
      {
        if (system.isPlaying && !system.main.loop)
        {
          yield return new WaitUntil(() => !system.isPlaying);
        }
      }

      yield return new WaitForEndOfFrame();
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