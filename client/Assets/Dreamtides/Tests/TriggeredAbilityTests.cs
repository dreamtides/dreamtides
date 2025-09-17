#nullable enable

using System.Collections;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.TestUtils;
using NUnit.Framework;
using UnityEngine.TestTools;

namespace Dreamtides.Tests
{
  public class TriggeredAbilityTests : IntegrationTest
  {
    [TearDown]
    public void TearDown()
    {
      Registry.TestConfiguration = null;
    }

    [UnityTest]
    public IEnumerator TestTriggeredAbility()
    {
      yield return Connect();
      yield return PerformAction(TestBattle.New().RemovePlayerHands().Build());
      yield return PerformAddCardAction(
        TestBattle.New().AddCardToHand(DisplayPlayer.User, TestCards.TestVanillaCharacter).Build()
      );
      var vanillaCharacter = Registry.CardService.GetCard(CurrentCardId);
      yield return PerformAddCardAction(
        TestBattle
          .New()
          .AddCardToHand(
            DisplayPlayer.User,
            TestCards.TestTriggerGainSparkWhenMaterializeAnotherCharacter
          )
          .Build()
      );
      var triggerCharacter = Registry.CardService.GetCard(CurrentCardId);

      yield return TestDragInputProvider.DragTo(
        Registry,
        triggerCharacter,
        Registry.Layout.DefaultStack
      );
      yield return WaitForCount(Registry.Layout.UserBattlefield, 1);

      Assert.That(triggerCharacter._battlefieldSparkText.text, Is.EqualTo("5"));

      Registry.TestHelperService.StartTrackingDisplayables();
      yield return TestDragInputProvider.DragTo(
        Registry,
        vanillaCharacter,
        Registry.Layout.DefaultStack
      );
      yield return WaitForCount(Registry.Layout.UserBattlefield, 2);

      Assert.That(triggerCharacter._battlefieldSparkText.text, Is.EqualTo("6"));

      var createdDisplayables = Registry.TestHelperService.CreatedDisplayables();
      Assert.That(createdDisplayables, Has.Some.Contains("Gain Spark When Materialize Another"));

      yield return EndTest();
    }
  }
}
