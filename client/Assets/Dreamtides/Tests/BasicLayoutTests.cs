using System.Collections;
using NUnit.Framework;
using UnityEngine.TestTools;
using Dreamtides.Services;
using Dreamtides.Utils;
using Dreamtides.Components;
using Dreamtides.TestUtils;

namespace Dreamtides.Tests
{
  public class BasicLayoutTests : IntegrationTest
  {
    [TearDown]
    public void TearDown()
    {
      Registry.TestConfiguration = null;
    }

    [UnityTest]
    public IEnumerator TestBasicLayout()
    {
      yield return Connect();
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