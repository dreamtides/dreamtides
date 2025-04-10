using System.Collections;
using NUnit.Framework;
using UnityEngine;
using UnityEngine.SceneManagement;
using UnityEngine.TestTools;
using Dreamtides.Services;
using Dreamtides.Utils;
using Dreamtides.Components;
using Dreamtides.UnityInternal;

namespace Dreamtides.Tests
{
  public class AllTests
  {
    public static readonly GameViewResolution[] Resolutions = new GameViewResolution[]
    {
      GameViewResolution.Resolution16x9,
      GameViewResolution.Resolution16x10,
      GameViewResolution.Resolution21x9,
      GameViewResolution.Resolution4x3,
      GameViewResolution.Resolution5x4,
      GameViewResolution.Resolution32x9,
      GameViewResolution.ResolutionIPhone12,
      GameViewResolution.ResolutionIPhoneSE,
      GameViewResolution.ResolutionIPadPro12,
      GameViewResolution.ResolutionIPodTouch6,
      GameViewResolution.ResolutionSamsungNote20,
      GameViewResolution.ResolutionSamsungZFold2,
      GameViewResolution.ResolutionPixel5,
    };

    [TearDown]
    public void TearDown()
    {
      Registry.TestConfiguration = null;
    }

    [UnityTest]
    public IEnumerator TestBasicLayout([ValueSource("Resolutions")] GameViewResolution resolution)
    {
      Registry.TestConfiguration = new TestConfiguration("basic");
      GameViewUtils.SetGameViewResolution(resolution);
      SceneManager.LoadScene("Assets/Scenes/Main.unity", LoadSceneMode.Single);
      yield return WaitForSceneLoad();
      var registry = ComponentUtils.Get<Registry>(GameObject.Find("Registry"));
      Assert.IsNotNull(registry);
      Debug.Log($"Running tests at {Screen.width}x{Screen.height}");
      Assert.AreEqual(1 + 1, 2);

      yield return new WaitForSeconds(5f);

      foreach (var displayable in registry.Layout.UserHand.Objects)
      {
        var card = ComponentUtils.Get<Card>(displayable);
        ComponentAssertions.AssertSpriteIsOnScreen(registry, card.CostBackgroundForTests, $"Card {card.Id}");
      }
    }

    private IEnumerator WaitForSceneLoad()
    {
      while (SceneManager.GetActiveScene().buildIndex > 0)
      {
        yield return null;
      }
    }
  }
}
