using System.Collections;
using NUnit.Framework;
using UnityEngine.TestTools;
using Dreamtides.Services;
using Dreamtides.Utils;
using Dreamtides.Components;
using Dreamtides.UnityInternal;
using UnityEngine;

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

    [UnityTest]
    public IEnumerator TestBasicLayout([ValueSource("Resolutions")] GameViewResolution resolution)
    {
      Registry registry = null;
      yield return TestUtil.LoadScenario(resolution, "basic", (r) =>
      {
        registry = r;
      });

      ComponentAssertions.AssertBoxColliderIsOnScreen(registry,
          GetBoxCollider(registry.Layout.UserDeck), "User deck is not on screen");
      ComponentAssertions.AssertBoxColliderIsOnScreen(registry,
          GetBoxCollider(registry.Layout.EnemyDeck), "Enemy deck is not on screen");
      ComponentAssertions.AssertBoxColliderIsOnScreen(registry,
          GetBoxCollider(registry.Layout.UserVoid), "User void is not on screen");
      ComponentAssertions.AssertBoxColliderIsOnScreen(registry,
          GetBoxCollider(registry.Layout.EnemyVoid), "Enemy void is not on screen");

      foreach (var displayable in registry.Layout.UserHand.Objects)
      {
        var card = ComponentUtils.Get<Card>(displayable);
        ComponentAssertions.AssertSpriteIsOnScreen(registry, card.CostBackgroundForTests, $"Card {card.Id}");
      }

      yield return TestUtil.TearDownScenario();
    }

    [UnityTest]
    public IEnumerator TestOpenUserVoidBrowser()
    {
      Registry registry = null;
      yield return TestUtil.LoadScenario(GameViewResolution.Resolution16x9, "basic", (r) =>
      {
        registry = r;
      });

      ComponentAssertions.AssertNotEmpty(registry.Layout.UserVoid);
      yield return TestClickInputProvider.ClickOn(registry, registry.Layout.UserVoid.transform);
      ComponentAssertions.AssertEmpty(registry.Layout.UserVoid);
      ComponentAssertions.AssertNotEmpty(registry.Layout.Browser);
      ComponentAssertions.AssertActive(registry.Layout.Browser._closeButton);

      yield return TestUtil.TearDownScenario();
    }

    static BoxCollider GetBoxCollider(Component component)
    {
      return Errors.CheckNotNull(component.GetComponentInChildren<BoxCollider>(),
          $"No BoxCollider found on {component.gameObject}");
    }
  }
}
