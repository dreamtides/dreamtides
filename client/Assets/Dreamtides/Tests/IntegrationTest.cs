using System.Collections;
using NUnit.Framework;
using UnityEngine.TestTools;
using Dreamtides.Services;
using Dreamtides.Utils;
using Dreamtides.Components;
using Dreamtides.UnityInternal;
using UnityEngine;
using System.Linq;
using System;
using UnityEngine.SceneManagement;
using Dreamtides.Layout;

#nullable enable

namespace Dreamtides.Tests
{
  public abstract class IntegrationTest
  {

    Registry? _registry;
    protected Registry Registry => Errors.CheckNotNull(_registry);

    protected IEnumerator Connect(GameViewResolution resolution)
    {
      Registry.TestConfiguration = new TestConfiguration(Guid.NewGuid());
      GameViewUtils.SetGameViewResolution(resolution);
      SceneManager.LoadScene("Assets/Scenes/Main.unity", LoadSceneMode.Single);
      yield return WaitForSceneLoad();
      yield return new WaitForSeconds(0.1f);
      var registry = ComponentUtils.Get<Registry>(GameObject.Find("Registry"));
      Assert.IsNotNull(registry);
      Assert.AreEqual(GameViewUtils.GetResolution(resolution), new Vector2(Screen.width, Screen.height),
          $"Resolution {resolution} not set");
      yield return new WaitUntil(() => registry.ActionService.Connected);
      yield return registry.TestHelperService.WaitForIdle();
      _registry = registry;
    }

    protected IEnumerator EndTest()
    {
      Registry.TestConfiguration = null;
      yield return Registry.TestHelperService.WaitForIdle();
      yield return new WaitForSeconds(0.5f);
    }

    protected IEnumerator WaitForCount(ObjectLayout layout, int count)
    {
      yield return new WaitUntil(() => layout.Objects.Count == count);
      yield return Registry.TestHelperService.WaitForIdle();
    }

    protected IEnumerator WaitForSceneLoad()
    {
      while (SceneManager.GetActiveScene().buildIndex > 0)
      {
        yield return null;
      }
    }
  }
}
