#nullable enable

using System.Collections;
using NUnit.Framework;
using UnityEngine.TestTools;
using Dreamtides.Services;
using Dreamtides.Utils;
using Dreamtides.Components;
using Dreamtides.TestUtils;
using Dreamtides.UnityInternal;

namespace Dreamtides.Tests
{
  public class BasicLayoutTests : IntegrationTest
  {
    public static readonly GameViewResolution[] AllResolutions = new GameViewResolution[]
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
    public IEnumerator TestBasicLayout([ValueSource("AllResolutions")] GameViewResolution resolution)
    {
      yield return Connect(resolution: resolution);
      yield return PerformAction(TestBattle.New().FullLayout().Build());
      AssertBoxColliderIsOnScreen(GetBoxCollider(Registry.Layout.UserDeck), "User deck is not on screen");
      AssertBoxColliderIsOnScreen(GetBoxCollider(Registry.Layout.EnemyDeck), "Enemy deck is not on screen");
      AssertBoxColliderIsOnScreen(GetBoxCollider(Registry.Layout.UserVoid), "User void is not on screen");
      AssertBoxColliderIsOnScreen(GetBoxCollider(Registry.Layout.EnemyVoid), "Enemy void is not on screen");

      foreach (var displayable in Registry.Layout.UserHand.Objects)
      {
        // With 5 cards in hand, all of them should be visible on screen. Beyond
        // this point we switch to scroll bars in some UI configurations.
        var card = ComponentUtils.Get<Card>(displayable);
        AssertSpriteIsOnScreen(card._costBackground, $"Energy Cost of {card.Id}");
      }

      yield return EndTest();
    }
  }
}