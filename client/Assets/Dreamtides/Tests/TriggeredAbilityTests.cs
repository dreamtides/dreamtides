#nullable enable

using System.Collections;
using NUnit.Framework;
using UnityEngine.TestTools;
using Dreamtides.Services;
using Dreamtides.TestUtils;
using Dreamtides.UnityInternal;
using Dreamtides.Schema;
using System.Collections.Generic;

namespace Dreamtides.Tests
{
  public class TriggeredAbilityTests : IntegrationTest
  {
    public static readonly GameViewResolution[] MobileAndDesktop = new GameViewResolution[]
    {
      GameViewResolution.Resolution16x9,
      GameViewResolution.ResolutionIPhone12,
    };

    [TearDown]
    public void TearDown()
    {
      Registry.TestConfiguration = null;
    }

    [UnityTest]
    public IEnumerator TestTriggeredAbility()
    {
      yield return Connect();
      yield return PerformAction(TestBattle.New()
        .RemovePlayerHands()
        .Build()
      );
      yield return PerformAddCardAction(TestBattle.New()
        .AddCardToHand(DisplayPlayer.User, CardName.TestVanillaCharacter)
        .Build()
      );
      var vanillaCharacter = Registry.LayoutService.GetCard(CurrentCardId);
      yield return PerformAddCardAction(TestBattle.New()
        .AddCardToHand(DisplayPlayer.User, CardName.TestTriggerGainSparkWhenMaterializeAnotherCharacter)
        .Build()
      );
      var triggerCharacter = Registry.LayoutService.GetCard(CurrentCardId);

      yield return TestDragInputProvider.DragTo(
        Registry,
        triggerCharacter,
        Registry.Layout.DefaultStack);
      yield return WaitForCount(Registry.Layout.UserBattlefield, 1);

      Assert.That(triggerCharacter._battlefieldSparkText.text, Is.EqualTo("5"));

      Registry.TestHelperService.StartTrackingDisplayables();
      yield return TestDragInputProvider.DragTo(
        Registry,
        vanillaCharacter,
        Registry.Layout.DefaultStack);
      yield return WaitForCount(Registry.Layout.UserBattlefield, 2);

      Assert.That(triggerCharacter._battlefieldSparkText.text, Is.EqualTo("6"));

      var createdDisplayables = Registry.TestHelperService.CreatedDisplayables();
      Assert.That(createdDisplayables, Has.Some.Contains("Materialize Gain Spark"));

      yield return EndTest();
    }
  }
}